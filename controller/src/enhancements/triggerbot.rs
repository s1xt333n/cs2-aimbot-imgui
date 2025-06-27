use std::time::{Duration, Instant};
use std::collections::HashMap;

use anyhow::Context;
use cs2::{
    MouseState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
    ClassNameCache,
    PlayerPawnState,
    StatePawnInfo,
};
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_CSPlayerPawn,
};
use nalgebra::Vector3;
use overlay::UnicodeTextRenderer;
use rand::{thread_rng, Rng};
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{KeyToggle, StateLocalCrosshair},
    UpdateContext,
};

/// Enhanced triggerbot states
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerState {
    Idle,
    Pending { 
        delay: Duration, 
        timestamp: Instant,
        target: EntityHandle<()>,
    },
    Shooting { 
        duration: Duration, 
        timestamp: Instant,
        target: EntityHandle<()>,
    },
    Cooldown { 
        delay: Duration, 
        timestamp: Instant 
    },
}

/// Target validation information
#[derive(Debug, Clone)]
pub struct TriggerTarget {
    pub entity_handle: EntityHandle<()>,
    pub health: i32,
    pub team_id: u8,
    pub position: Vector3<f32>,
    pub is_visible: bool,
    pub hitbox_hit: String,
    pub distance: f32,
    pub last_seen: Instant,
}

/// Humanization settings for triggerbot
#[derive(Debug, Clone)]
pub struct TriggerHumanization {
    pub reaction_time_min: Duration,
    pub reaction_time_max: Duration,
    pub shot_duration_min: Duration,
    pub shot_duration_max: Duration,
    pub miss_chance: f32,
    pub fatigue_factor: f32,
    pub consistency_factor: f32,
}

impl Default for TriggerHumanization {
    fn default() -> Self {
        Self {
            reaction_time_min: Duration::from_millis(80),
            reaction_time_max: Duration::from_millis(200),
            shot_duration_min: Duration::from_millis(50),
            shot_duration_max: Duration::from_millis(150),
            miss_chance: 0.02,
            fatigue_factor: 1.0,
            consistency_factor: 0.85,
        }
    }
}

/// Enhanced triggerbot with intelligent targeting and humanization
pub struct EnhancedTriggerBot {
    // Core state
    toggle: KeyToggle,
    state: TriggerState,
    
    // Target management
    current_target: Option<TriggerTarget>,
    target_history: HashMap<u32, Vec<Instant>>,
    
    // Humanization
    humanization: TriggerHumanization,
    shot_count: u32,
    consecutive_hits: u32,
    last_shot_time: Instant,
    
    // Performance tracking
    total_shots: u32,
    total_hits: u32,
    session_start: Instant,
    
    // Anti-detection
    randomization_seed: u64,
    last_randomization: Instant,
    
    // Configuration cache
    local_team_id: u8,
    local_position: Vector3<f32>,
}

impl EnhancedTriggerBot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            state: TriggerState::Idle,
            
            current_target: None,
            target_history: HashMap::new(),
            
            humanization: TriggerHumanization::default(),
            shot_count: 0,
            consecutive_hits: 0,
            last_shot_time: Instant::now(),
            
            total_shots: 0,
            total_hits: 0,
            session_start: Instant::now(),
            
            randomization_seed: thread_rng().gen(),
            last_randomization: Instant::now(),
            
            local_team_id: 0,
            local_position: Vector3::zeros(),
        }
    }
    
    /// Updates humanization settings based on user configuration
    pub fn update_humanization(&mut self, settings: &AppSettings) {
        self.humanization.reaction_time_min = Duration::from_millis(settings.trigger_bot_delay_min as u64);
        self.humanization.reaction_time_max = Duration::from_millis(settings.trigger_bot_delay_max as u64);
        self.humanization.shot_duration_min = Duration::from_millis(settings.trigger_bot_shot_duration as u64 / 2);
        self.humanization.shot_duration_max = Duration::from_millis(settings.trigger_bot_shot_duration as u64);
        
        // Apply fatigue based on session length and shot count
        let session_time = self.session_start.elapsed().as_secs_f32() / 3600.0; // Hours
        self.humanization.fatigue_factor = (1.0 + session_time * 0.1 + self.shot_count as f32 * 0.001).min(2.0);
        
        // Apply consistency degradation over time
        if self.consecutive_hits > 10 {
            self.humanization.consistency_factor = (0.85 - (self.consecutive_hits as f32 - 10.0) * 0.01).max(0.6);
        }
    }
    
    /// Generates humanized delay with fatigue and consistency factors
    fn generate_humanized_delay(&self, base_min: Duration, base_max: Duration) -> Duration {
        let mut rng = thread_rng();
        
        // Apply fatigue to increase delay
        let fatigue_multiplier = self.humanization.fatigue_factor;
        let consistency_variation = 1.0 + (1.0 - self.humanization.consistency_factor) * 0.5;
        
        let min_ms = (base_min.as_millis() as f32 * fatigue_multiplier) as u64;
        let max_ms = (base_max.as_millis() as f32 * fatigue_multiplier * consistency_variation) as u64;
        
        Duration::from_millis(rng.gen_range(min_ms..=max_ms))
    }
    
    /// Checks if target is valid for triggering
    fn is_valid_target(&self, target: &TriggerTarget, settings: &AppSettings) -> bool {
        // Team check
        if settings.trigger_bot_team_check && target.team_id == self.local_team_id {
            return false;
        }
        
        // Health check
        if target.health <= 0 {
            return false;
        }
        
        // Distance check (optional)
        if target.distance > 5000.0 {
            return false;
        }
        
        // Visibility check
        if !target.is_visible {
            return false;
        }
        
        // Rate limiting per target
        if let Some(history) = self.target_history.get(&target.entity_handle.get_entity_index()) {
            let recent_shots = history.iter()
                .filter(|&time| time.elapsed() < Duration::from_secs(5))
                .count();
            
            if recent_shots > 3 {
                return false; // Too many recent shots on this target
            }
        }
        
        true
    }
    
    /// Simulates human-like miss chance
    fn should_miss_shot(&self) -> bool {
        let mut rng = thread_rng();
        
        // Base miss chance
        let mut miss_chance = self.humanization.miss_chance;
        
        // Increase miss chance if too many consecutive hits
        if self.consecutive_hits > 5 {
            miss_chance += (self.consecutive_hits as f32 - 5.0) * 0.01;
        }
        
        // Decrease miss chance for closer targets
        if let Some(target) = &self.current_target {
            if target.distance < 1000.0 {
                miss_chance *= 0.5;
            }
        }
        
        // Apply fatigue
        miss_chance *= self.humanization.fatigue_factor.min(1.5);
        
        rng.gen::<f32>() < miss_chance.min(0.15)
    }
    
    /// Performs crosshair target detection
    fn detect_crosshair_target(&self, ctx: &UpdateContext) -> anyhow::Result<Option<TriggerTarget>> {
        let crosshair = ctx.states.resolve::<StateLocalCrosshair>(())?;
        
        if crosshair.entity_id == 0 {
            return Ok(None);
        }
        
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(())?;
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        
        // Find the entity under crosshair
        for entity_identity in entities.entities() {
            if entity_identity.handle::<()>()?.get_entity_index() != crosshair.entity_id {
                continue;
            }
            
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
            
            let distance = (pawn_info.position - self.local_position).norm();
            
            let target = TriggerTarget {
                entity_handle: entity_identity.handle()?,
                health: pawn_info.player_health,
                team_id: pawn_info.team_id,
                position: pawn_info.position,
                is_visible: true, // Simplified - in crosshair means visible
                hitbox_hit: "unknown".to_string(), // Would be determined by ray trace
                distance,
                last_seen: Instant::now(),
            };
            
            return Ok(Some(target));
        }
        
        Ok(None)
    }
    
    /// Executes the shot with mouse input
    fn execute_shot(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        // Record shot statistics
        self.total_shots += 1;
        self.shot_count += 1;
        
        // Simulate mouse click
        ctx.cs2.send_mouse_state(&[MouseState {
            left_click: true,
            ..Default::default()
        }])?;
        
        // Record target history
        if let Some(ref target) = self.current_target {
            let entity_id = target.entity_handle.get_entity_index();
            self.target_history
                .entry(entity_id)
                .or_insert_with(Vec::new)
                .push(Instant::now());
        }
        
        self.last_shot_time = Instant::now();
        Ok(())
    }
    
    /// Updates local player information
    fn update_local_player_info(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        
        if let Some(local_controller) = local_controller.instance.value_reference(memory.view_arc()) {
            self.local_team_id = local_controller.m_iPendingTeamNum()?;
            
            // Get local pawn position
            let local_pawn_handle = local_controller.m_hPlayerPawn()?;
            let entities = ctx.states.resolve::<StateEntityList>(())?;
            
            if let Ok(local_pawn_entity) = entities.entity_from_handle(&local_pawn_handle) {
                if let Some(local_pawn) = local_pawn_entity.value_reference(memory.view_arc()) {
                    let origin = local_pawn.m_vOldOrigin()?;
                    self.local_position = Vector3::new(origin[0], origin[1], origin[2]);
                }
            }
        }
        
        Ok(())
    }
    
    /// Performs intelligent target validation
    fn validate_target_intelligence(&self, target: &TriggerTarget, settings: &AppSettings) -> bool {
        // Check if target moved significantly (may be evading)
        if let Some(history) = self.target_history.get(&target.entity_handle.get_entity_index()) {
            if let Some(&last_shot_time) = history.last() {
                if last_shot_time.elapsed() < Duration::from_millis(500) {
                    // Target was shot recently, check if they're still in similar position
                    // This would require position history tracking
                    return true; // Simplified
                }
            }
        }
        
        // Check target's movement pattern (would require velocity tracking)
        // For now, just validate basic conditions
        self.is_valid_target(target, settings)
    }
    
    /// Cleanup old data to prevent memory leaks
    fn cleanup_old_data(&mut self) {
        let cutoff_time = Instant::now() - Duration::from_secs(30);
        
        // Clean up target history
        for (_, history) in self.target_history.iter_mut() {
            history.retain(|&time| time > cutoff_time);
        }
        
        // Remove empty entries
        self.target_history.retain(|_, history| !history.is_empty());
        
        // Reset randomization seed periodically
        if self.last_randomization.elapsed() > Duration::from_secs(300) {
            self.randomization_seed = thread_rng().gen();
            self.last_randomization = Instant::now();
        }
    }
}

impl Enhancement for EnhancedTriggerBot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        
        // Update toggle state
        if self.toggle.update(
            &settings.trigger_bot_mode,
            ctx.input,
            &settings.key_trigger_bot,
        ) {
            ctx.cs2.add_metrics_record(
                "triggerbot-toggle",
                &format!("enabled: {}", self.toggle.enabled),
            );
        }
        
        if !self.toggle.enabled {
            self.state = TriggerState::Idle;
            return Ok(());
        }
        
        // Update local player info and humanization
        self.update_local_player_info(ctx)?;
        self.update_humanization(&settings);
        
        // State machine logic
        match &self.state.clone() {
            TriggerState::Idle => {
                // Look for target under crosshair
                if let Some(target) = self.detect_crosshair_target(ctx)? {
                    if self.is_valid_target(&target, &settings) {
                        // Generate humanized reaction delay
                        let delay = self.generate_humanized_delay(
                            self.humanization.reaction_time_min,
                            self.humanization.reaction_time_max,
                        );
                        
                        self.current_target = Some(target.clone());
                        self.state = TriggerState::Pending {
                            delay,
                            timestamp: Instant::now(),
                            target: target.entity_handle,
                        };
                    }
                }
            }
            
            TriggerState::Pending { delay, timestamp, target } => {
                // Check if delay has elapsed
                if timestamp.elapsed() >= *delay {
                    // Re-validate target if configured
                    let should_shoot = if settings.trigger_bot_check_target_after_delay {
                        if let Some(current_target) = self.detect_crosshair_target(ctx)? {
                            current_target.entity_handle == *target &&
                            self.validate_target_intelligence(&current_target, &settings)
                        } else {
                            false
                        }
                    } else {
                        true
                    };
                    
                    if should_shoot && !self.should_miss_shot() {
                        // Execute shot
                        self.execute_shot(ctx)?;
                        
                        // Generate shot duration
                        let duration = self.generate_humanized_delay(
                            self.humanization.shot_duration_min,
                            self.humanization.shot_duration_max,
                        );
                        
                        self.state = TriggerState::Shooting {
                            duration,
                            timestamp: Instant::now(),
                            target: *target,
                        };
                        
                        self.consecutive_hits += 1;
                        self.total_hits += 1;
                    } else {
                        // Miss or target lost
                        if self.should_miss_shot() {
                            self.consecutive_hits = 0;
                        }
                        
                        self.state = TriggerState::Idle;
                    }
                } else {
                    // Check if target is still valid during delay
                    if settings.trigger_bot_check_target_after_delay {
                        if let Some(current_target) = self.detect_crosshair_target(ctx)? {
                            if current_target.entity_handle != *target {
                                self.state = TriggerState::Idle;
                            }
                        } else {
                            self.state = TriggerState::Idle;
                        }
                    }
                }
            }
            
            TriggerState::Shooting { duration, timestamp, target: _ } => {
                if timestamp.elapsed() >= *duration {
                    // Stop shooting and enter cooldown
                    ctx.cs2.send_mouse_state(&[MouseState {
                        left_click: false,
                        ..Default::default()
                    }])?;
                    
                    // Generate cooldown period
                    let cooldown = Duration::from_millis(
                        thread_rng().gen_range(50..150)
                    );
                    
                    self.state = TriggerState::Cooldown {
                        delay: cooldown,
                        timestamp: Instant::now(),
                    };
                }
            }
            
            TriggerState::Cooldown { delay, timestamp } => {
                if timestamp.elapsed() >= *delay {
                    self.state = TriggerState::Idle;
                }
            }
        }
        
        // Periodic cleanup
        if self.last_shot_time.elapsed() > Duration::from_secs(10) {
            self.cleanup_old_data();
        }
        
        Ok(())
    }
    
    fn render(
        &self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        // Render debug info or statistics if needed
        Ok(())
    }
}

impl EnhancedTriggerBot {
    /// Gets current performance statistics
    pub fn get_statistics(&self) -> TriggerBotStats {
        let accuracy = if self.total_shots > 0 {
            self.total_hits as f32 / self.total_shots as f32
        } else {
            0.0
        };
        
        TriggerBotStats {
            total_shots: self.total_shots,
            total_hits: self.total_hits,
            accuracy,
            consecutive_hits: self.consecutive_hits,
            session_time: self.session_start.elapsed(),
            last_shot: self.last_shot_time.elapsed(),
        }
    }
    
    /// Resets statistics
    pub fn reset_statistics(&mut self) {
        self.total_shots = 0;
        self.total_hits = 0;
        self.consecutive_hits = 0;
        self.session_start = Instant::now();
        self.shot_count = 0;
    }
}

/// Statistics structure for triggerbot performance
#[derive(Debug, Clone)]
pub struct TriggerBotStats {
    pub total_shots: u32,
    pub total_hits: u32,
    pub accuracy: f32,
    pub consecutive_hits: u32,
    pub session_time: Duration,
    pub last_shot: Duration,
}