use std::time::{Duration, Instant};
use std::collections::HashMap;

use anyhow::Context;
use cs2::{
    MouseState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
    BoneFlags,
    CEntityIdentityEx,
    CS2Model,
    ClassNameCache,
    LocalCameraControllerTarget,
    PlayerPawnState,
    StatePawnInfo,
    StatePawnModelInfo,
};
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_CSPlayerPawn,
};
use nalgebra::{Vector2, Vector3, Vector4};
use overlay::UnicodeTextRenderer;
use rand::{thread_rng, Rng};
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    math::{
        prediction::{
            VelocityTracker, 
            predict_position_with_physics, 
            calculate_intercept_time,
            calculate_optimal_prediction_time,
            WeaponType,
        },
        smoothing::{
            SmoothController, 
            SmoothingType, 
            HumanizationSettings,
        },
    },
    settings::AppSettings,
    view::{KeyToggle, ViewController},
    UpdateContext,
};

/// Hitbox priority for targeting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hitbox {
    Head = 8,
    Neck = 7,
    UpperChest = 6,
    Chest = 5,
    LowerChest = 4,
    Stomach = 3,
    Pelvis = 2,
}

impl Hitbox {
    pub fn priority(&self) -> u8 {
        *self as u8
    }
    
    pub fn get_bone_name(&self) -> &'static str {
        match self {
            Hitbox::Head => "head_0",
            Hitbox::Neck => "neck_0",
            Hitbox::UpperChest => "spine_2",
            Hitbox::Chest => "spine_1",
            Hitbox::LowerChest => "spine_0",
            Hitbox::Stomach => "spine_0",
            Hitbox::Pelvis => "pelvis",
        }
    }
}

/// Target information for aimbot
#[derive(Debug, Clone)]
pub struct AimbotTarget {
    pub entity_handle: EntityHandle<()>,
    pub position: Vector3<f32>,
    pub head_position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub distance: f32,
    pub fov_distance: f32,
    pub is_visible: bool,
    pub hitbox: Hitbox,
    pub health: i32,
    pub is_enemy: bool,
    pub weapon_type: WeaponType,
    pub last_seen: Instant,
}

/// Aimbot state machine
#[derive(Debug, Clone, PartialEq)]
pub enum AimbotState {
    Idle,
    Acquiring { target: EntityHandle<()>, start_time: Instant },
    Tracking { target: EntityHandle<()> },
    Shooting { target: EntityHandle<()>, duration: Duration, start_time: Instant },
}

/// Advanced aimbot implementation
pub struct AdvancedAimbot {
    // Core components
    toggle: KeyToggle,
    smooth_controller: SmoothController,
    humanization: HumanizationSettings,
    
    // State management
    state: AimbotState,
    velocity_trackers: HashMap<u32, VelocityTracker>,
    
    // Target management
    current_target: Option<AimbotTarget>,
    targets: Vec<AimbotTarget>,
    target_selection_cooldown: Instant,
    
    // Performance tracking
    last_update: Instant,
    shots_fired_count: u32,
    
    // Configuration cache
    local_team_id: u8,
    local_eye_position: Vector3<f32>,
    view_angles: Vector2<f32>,
    
    // Anti-detection
    last_shot_time: Instant,
    consecutive_headshots: u32,
    randomization_seed: u64,
}

impl AdvancedAimbot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            smooth_controller: SmoothController::new(SmoothingType::Natural, 2.0),
            humanization: HumanizationSettings::default(),
            
            state: AimbotState::Idle,
            velocity_trackers: HashMap::new(),
            
            current_target: None,
            targets: Vec::new(),
            target_selection_cooldown: Instant::now(),
            
            last_update: Instant::now(),
            shots_fired_count: 0,
            
            local_team_id: 0,
            local_eye_position: Vector3::zeros(),
            view_angles: Vector2::zeros(),
            
            last_shot_time: Instant::now(),
            consecutive_headshots: 0,
            randomization_seed: thread_rng().gen(),
        }
    }
    
    /// Updates velocity tracking for a player
    fn update_velocity_tracking(&mut self, entity_id: u32, position: Vector3<f32>) {
        let tracker = self.velocity_trackers
            .entry(entity_id)
            .or_insert_with(|| VelocityTracker::new(10));
        
        tracker.add_position(position);
    }
    
    /// Calculates the best hitbox to target based on visibility and settings
    fn calculate_best_hitbox(
        &self,
        target_pos: &Vector3<f32>,
        model: &CS2Model,
        bone_states: &[cs2::BoneState],
        settings: &crate::settings::AimbotConfig,
    ) -> Option<(Hitbox, Vector3<f32>)> {
        let hitboxes = if settings.prefer_head {
            vec![Hitbox::Head, Hitbox::Neck, Hitbox::UpperChest, Hitbox::Chest]
        } else {
            vec![Hitbox::Chest, Hitbox::UpperChest, Hitbox::Head, Hitbox::Neck]
        };
        
        for hitbox in hitboxes {
            if let Some(bone_index) = model.bones.iter()
                .position(|bone| bone.name == hitbox.get_bone_name()) {
                
                if let Some(bone_state) = bone_states.get(bone_index) {
                    // Check if hitbox is visible (simplified ray check)
                    if self.is_hitbox_visible(&bone_state.position, settings) {
                        return Some((hitbox, bone_state.position));
                    }
                }
            }
        }
        
        None
    }
    
    /// Simplified visibility check for hitboxes
    fn is_hitbox_visible(&self, position: &Vector3<f32>, settings: &crate::settings::AimbotConfig) -> bool {
        // In a real implementation, this would do proper ray tracing
        // For now, we'll use a simplified distance and angle check
        let distance = (position - &self.local_eye_position).norm();
        distance <= settings.max_distance
    }
    
    /// Calculates FOV distance from crosshair to target
    fn calculate_fov_distance(&self, target_pos: &Vector3<f32>) -> f32 {
        let direction = (target_pos - &self.local_eye_position).normalize();
        let view_forward = Vector3::new(
            self.view_angles.y.to_radians().cos() * self.view_angles.x.to_radians().cos(),
            self.view_angles.y.to_radians().sin() * self.view_angles.x.to_radians().cos(),
            -self.view_angles.x.to_radians().sin(),
        );
        
        let dot_product = direction.dot(&view_forward);
        let angle = dot_product.acos();
        angle.to_degrees()
    }
    
    /// Selects the best target based on various factors
    fn select_best_target(&self, settings: &crate::settings::AimbotConfig) -> Option<AimbotTarget> {
        if self.targets.is_empty() {
            return None;
        }
        
        let mut scored_targets: Vec<(f32, &AimbotTarget)> = self.targets
            .iter()
            .filter(|target| {
                target.is_enemy &&
                target.distance <= settings.max_distance &&
                target.fov_distance <= settings.fov &&
                (!settings.visible_only || target.is_visible)
            })
            .map(|target| {
                let mut score = 0.0f32;
                
                // FOV distance (closer to crosshair = higher score)
                score += (settings.fov - target.fov_distance) * settings.fov_weight;
                
                // Distance (closer = higher score)
                score += (settings.max_distance - target.distance) * settings.distance_weight;
                
                // Health (lower health = higher score for finishing)
                score += (100.0 - target.health as f32) * settings.health_weight;
                
                // Hitbox priority
                score += target.hitbox.priority() as f32 * settings.hitbox_weight;
                
                // Visibility bonus
                if target.is_visible {
                    score += settings.visibility_bonus;
                }
                
                // Time since last seen penalty
                let time_penalty = target.last_seen.elapsed().as_secs_f32() * settings.time_penalty;
                score -= time_penalty;
                
                (score, target)
            })
            .collect();
        
        // Sort by score (highest first)
        scored_targets.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        scored_targets.first().map(|(_, target)| (*target).clone())
    }
    
    /// Calculates the aim position with prediction and smoothing
    fn calculate_aim_position(
        &self,
        target: &AimbotTarget,
        settings: &crate::settings::AimbotConfig,
    ) -> Option<Vector3<f32>> {
        let entity_id = target.entity_handle.get_entity_index();
        
        // Get velocity prediction if available
        let predicted_position = if settings.prediction_enabled {
            if let Some(tracker) = self.velocity_trackers.get(&entity_id) {
                let prediction_time = calculate_optimal_prediction_time(
                    target.distance,
                    target.weapon_type,
                );
                
                tracker.predict_future_position(prediction_time)
                    .unwrap_or(target.position)
            } else {
                target.position
            }
        } else {
            target.position
        };
        
        // Apply humanization if enabled
        let screen_pos = self.world_to_screen_position(&predicted_position)?;
        let humanized_pos = if settings.humanization_enabled {
            self.humanization.humanize_target(screen_pos, target.distance)
        } else {
            screen_pos
        };
        
        self.screen_to_world_position(&humanized_pos)
    }
    
    /// Converts world position to screen coordinates (simplified)
    fn world_to_screen_position(&self, world_pos: &Vector3<f32>) -> Option<Vector2<f32>> {
        // This would use the actual view matrix in a real implementation
        // For now, return a placeholder
        Some(Vector2::new(960.0, 540.0))
    }
    
    /// Converts screen coordinates to world position (simplified)
    fn screen_to_world_position(&self, screen_pos: &Vector2<f32>) -> Option<Vector3<f32>> {
        // This would use the inverse view matrix in a real implementation
        // For now, return a placeholder
        Some(Vector3::zeros())
    }
    
    /// Executes the aim movement
    fn execute_aim(&mut self, target_pos: Vector3<f32>, settings: &crate::settings::AimbotConfig, ctx: &UpdateContext) -> anyhow::Result<()> {
        // Calculate required mouse movement
        let current_angles = self.view_angles;
        let target_angles = self.calculate_angles_to_target(&target_pos);
        let angle_delta = target_angles - current_angles;
        
        // Convert angles to mouse movement
        let mouse_sensitivity = 1.0; // Would get from game settings
        let mouse_delta = Vector2::new(
            angle_delta.y / (mouse_sensitivity * 0.022),
            -angle_delta.x / (mouse_sensitivity * 0.022),
        );
        
        // Apply smoothing if enabled
        if settings.smooth_enabled && settings.smooth_factor > 0.0 {
            let current_screen_pos = Vector2::new(960.0, 540.0); // Current crosshair position
            let target_screen_pos = current_screen_pos + mouse_delta;
            
            if !self.smooth_controller.is_active() {
                self.smooth_controller.move_to(current_screen_pos, target_screen_pos);
            }
            
            if let Some(smooth_pos) = self.smooth_controller.update() {
                let smooth_delta = smooth_pos - current_screen_pos;
                self.send_mouse_input(smooth_delta, ctx)?;
            }
        } else {
            // Direct aim without smoothing
            self.send_mouse_input(mouse_delta, ctx)?;
        }
        
        Ok(())
    }
    
    /// Calculates angles needed to aim at target
    fn calculate_angles_to_target(&self, target_pos: &Vector3<f32>) -> Vector2<f32> {
        let delta = target_pos - &self.local_eye_position;
        let distance = delta.norm();
        
        if distance == 0.0 {
            return Vector2::zeros();
        }
        
        let pitch = (-delta.z / distance).asin().to_degrees();
        let yaw = delta.y.atan2(delta.x).to_degrees();
        
        Vector2::new(pitch, yaw)
    }
    
    /// Sends mouse input to the game
    fn send_mouse_input(&self, delta: Vector2<f32>, ctx: &UpdateContext) -> anyhow::Result<()> {
        if delta.norm() < 0.1 {
            return Ok(());
        }
        
        ctx.cs2.send_mouse_state(&[MouseState {
            last_x: delta.x as i32,
            last_y: delta.y as i32,
            ..Default::default()
        }])?;
        
        Ok(())
    }
    
    /// Anti-detection logic
    fn should_suppress_shot(&mut self, settings: &crate::settings::AimbotConfig) -> bool {
        let now = Instant::now();
        
        // Rate limiting
        if now.duration_since(self.last_shot_time) < Duration::from_millis(settings.min_shot_delay) {
            return true;
        }
        
        // Consecutive headshot limiting
        if self.consecutive_headshots >= settings.max_consecutive_headshots {
            // Reset with some randomization
            let mut rng = thread_rng();
            if rng.gen::<f32>() < 0.3 {
                self.consecutive_headshots = 0;
            } else {
                return true;
            }
        }
        
        // Random suppression for legit mode
        if settings.legit_mode {
            let mut rng = thread_rng();
            if rng.gen::<f32>() < settings.miss_chance {
                return true;
            }
        }
        
        false
    }
}

impl Enhancement for AdvancedAimbot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        let aimbot_settings = &settings.aimbot_config;
        
        // Check if aimbot is enabled
        if self.toggle.update(
            &aimbot_settings.toggle_mode,
            ctx.input,
            &aimbot_settings.hotkey,
        ) {
            ctx.cs2.add_metrics_record(
                "aimbot-toggle",
                &format!("enabled: {}", self.toggle.enabled),
            );
        }
        
        if !self.toggle.enabled {
            self.state = AimbotState::Idle;
            self.smooth_controller.stop();
            return Ok(());
        }
        
        // Update local player information
        self.update_local_player_info(ctx)?;
        
        // Find and evaluate targets
        self.find_targets(ctx)?;
        
        // Update state machine
        match &self.state.clone() {
            AimbotState::Idle => {
                if let Some(target) = self.select_best_target(aimbot_settings) {
                    self.current_target = Some(target.clone());
                    self.state = AimbotState::Acquiring {
                        target: target.entity_handle,
                        start_time: Instant::now(),
                    };
                }
            }
            AimbotState::Acquiring { target, start_time } => {
                let acquisition_time = Duration::from_millis(aimbot_settings.acquisition_delay);
                
                if start_time.elapsed() >= acquisition_time {
                    self.state = AimbotState::Tracking { target: *target };
                } else if self.current_target.as_ref()
                    .map(|t| t.entity_handle != *target)
                    .unwrap_or(true) {
                    // Target lost or changed
                    self.state = AimbotState::Idle;
                }
            }
            AimbotState::Tracking { target } => {
                if let Some(current_target) = &self.current_target {
                    if current_target.entity_handle == *target {
                        // Calculate and execute aim
                        if let Some(aim_pos) = self.calculate_aim_position(current_target, aimbot_settings) {
                            self.execute_aim(aim_pos, aimbot_settings, ctx)?;
                            
                            // Check if we should start shooting
                            if aimbot_settings.auto_shoot && !self.should_suppress_shot(aimbot_settings) {
                                self.state = AimbotState::Shooting {
                                    target: *target,
                                    duration: Duration::from_millis(aimbot_settings.shot_duration),
                                    start_time: Instant::now(),
                                };
                            }
                        }
                    } else {
                        self.state = AimbotState::Idle;
                    }
                } else {
                    self.state = AimbotState::Idle;
                }
            }
            AimbotState::Shooting { target, duration, start_time } => {
                if start_time.elapsed() >= *duration {
                    self.last_shot_time = Instant::now();
                    self.shots_fired_count += 1;
                    
                    // Update headshot counter if targeting head
                    if let Some(ref current_target) = self.current_target {
                        if current_target.hitbox == Hitbox::Head {
                            self.consecutive_headshots += 1;
                        } else {
                            self.consecutive_headshots = 0;
                        }
                    }
                    
                    self.state = AimbotState::Idle;
                } else {
                    // Continue aiming while shooting
                    if let Some(current_target) = &self.current_target {
                        if let Some(aim_pos) = self.calculate_aim_position(current_target, aimbot_settings) {
                            self.execute_aim(aim_pos, aimbot_settings, ctx)?;
                        }
                    }
                }
            }
        }
        
        // Cleanup old velocity trackers
        let cutoff_time = Instant::now() - Duration::from_secs(5);
        self.velocity_trackers.retain(|_, tracker| {
            tracker.positions.last()
                .map(|(_, time)| *time > cutoff_time)
                .unwrap_or(false)
        });
        
        self.last_update = Instant::now();
        Ok(())
    }
    
    fn render(
        &self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        // Render debug information or crosshair modifications if needed
        Ok(())
    }
}

// Implementation methods continued...
impl AdvancedAimbot {
    fn update_local_player_info(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        
        if let Some(local_controller) = local_controller.instance.value_reference(memory.view_arc()) {
            self.local_team_id = local_controller.m_iPendingTeamNum()?;
            
            // Get local pawn for eye position
            let local_pawn_handle = local_controller.m_hPlayerPawn()?;
            let entities = ctx.states.resolve::<StateEntityList>(())?;
            
            if let Ok(local_pawn_entity) = entities.entity_from_handle(&local_pawn_handle) {
                if let Some(local_pawn) = local_pawn_entity.value_reference(memory.view_arc()) {
                    // Get eye position (simplified)
                    let origin = local_pawn.m_vOldOrigin()?;
                    let view_offset = local_pawn.m_vecViewOffset()?;
                    self.local_eye_position = Vector3::new(
                        origin[0] + view_offset[0],
                        origin[1] + view_offset[1],
                        origin[2] + view_offset[2],
                    );
                    
                    // Get view angles (would need proper implementation)
                    self.view_angles = Vector2::zeros(); // Placeholder
                }
            }
        }
        
        Ok(())
    }
    
    fn find_targets(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        self.targets.clear();
        
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(())?;
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        
        for entity_identity in entities.entities() {
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false) {
                continue;
            }
            
            let pawn_state = ctx.states.resolve::<PlayerPawnState>(entity_identity.handle()?)?;
            if *pawn_state != PlayerPawnState::Alive {
                continue;
            }
            
            let pawn_info = ctx.states.resolve::<StatePawnInfo>(entity_identity.handle()?)?;
            if pawn_info.player_health <= 0 {
                continue;
            }
            
            let pawn_model = ctx.states.resolve::<StatePawnModelInfo>(entity_identity.handle()?)?;
            let model = ctx.states.resolve::<CS2Model>(pawn_model.model_address)?;
            
            // Update velocity tracking
            let entity_id = entity_identity.handle::<()>()?.get_entity_index();
            self.update_velocity_tracking(entity_id, pawn_info.position);
            
            // Calculate target metrics
            let distance = (pawn_info.position - self.local_eye_position).norm();
            let fov_distance = self.calculate_fov_distance(&pawn_info.position);
            let is_enemy = pawn_info.team_id != self.local_team_id;
            
            // Find best hitbox
            if let Some((hitbox, hitbox_pos)) = self.calculate_best_hitbox(
                &pawn_info.position,
                &model,
                &pawn_model.bone_states,
                &ctx.states.resolve::<AppSettings>()?.aimbot_config,
            ) {
                let target = AimbotTarget {
                    entity_handle: entity_identity.handle()?,
                    position: pawn_info.position,
                    head_position: hitbox_pos,
                    velocity: self.velocity_trackers
                        .get(&entity_id)
                        .and_then(|t| t.get_velocity())
                        .unwrap_or_else(Vector3::zeros),
                    distance,
                    fov_distance,
                    is_visible: true, // Simplified
                    hitbox,
                    health: pawn_info.player_health,
                    is_enemy,
                    weapon_type: WeaponType::from_weapon_name(&pawn_info.weapon.display_name()),
                    last_seen: Instant::now(),
                };
                
                self.targets.push(target);
            }
        }
        
        Ok(())
    }
}