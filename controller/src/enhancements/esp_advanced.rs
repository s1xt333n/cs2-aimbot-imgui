use std::time::Instant;
use std::collections::HashMap;

use anyhow::Context;
use cs2::{
    BoneFlags,
    CEntityIdentityEx,
    CS2Model,
    ClassNameCache,
    LocalCameraControllerTarget,
    PlayerPawnState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
    StatePawnInfo,
    StatePawnModelInfo,
};
use imgui::{DrawListMut, ImColor32};
use nalgebra::{Vector2, Vector3};
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::{
        AppSettings,
        EspBoxType,
        EspConfig,
        EspHeadDot,
        EspHealthBar,
        EspPlayerSettings,
        EspSelector,
        EspTracePosition,
    },
    view::{KeyToggle, ViewController},
    UpdateContext,
};

/// Advanced ESP information structure
#[derive(Debug, Clone)]
pub struct AdvancedESPInfo {
    pub pawn_info: StatePawnInfo,
    pub pawn_model: StatePawnModelInfo,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub is_scoped: bool,
    pub is_defusing: bool,
    pub is_planting: bool,
    pub weapon_ammo: i32,
    pub armor: i32,
    pub money: i32,
    pub rank: String,
    pub last_seen: Instant,
    pub visibility_percentage: f32,
    pub predicted_position: Vector3<f32>,
}

/// Glow effect settings
#[derive(Debug, Clone)]
pub struct GlowSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub color: [f32; 4],
    pub through_walls: bool,
    pub style: GlowStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlowStyle {
    Outline,
    Solid,
    Pulsing,
    Gradient,
}

/// Chams (colored models) settings
#[derive(Debug, Clone)]
pub struct ChamsSettings {
    pub enabled: bool,
    pub visible_color: [f32; 4],
    pub hidden_color: [f32; 4],
    pub material_type: ChamsMaterial,
    pub brightness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChamsMaterial {
    Flat,
    Shiny,
    Glass,
    Chrome,
    Glow,
}

/// 2D Radar settings
#[derive(Debug, Clone)]
pub struct RadarSettings {
    pub enabled: bool,
    pub position: Vector2<f32>,
    pub size: f32,
    pub zoom: f32,
    pub show_enemies: bool,
    pub show_teammates: bool,
    pub show_weapons: bool,
    pub show_grenades: bool,
    pub rotation: bool,
}

/// Enhanced ESP with advanced features
pub struct AdvancedESP {
    // Core components
    toggle: KeyToggle,
    players: Vec<AdvancedESPInfo>,
    local_team_id: u8,
    
    // Advanced features
    glow_settings: GlowSettings,
    chams_settings: ChamsSettings,
    radar_settings: RadarSettings,
    
    // Performance tracking
    last_update: Instant,
    frame_count: u32,
    
    // Visibility tracking
    visibility_history: HashMap<u32, Vec<(bool, Instant)>>,
    
    // Prediction tracking
    velocity_history: HashMap<u32, Vec<(Vector3<f32>, Instant)>>,
}

impl AdvancedESP {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            players: Vec::new(),
            local_team_id: 0,
            
            glow_settings: GlowSettings {
                enabled: false,
                intensity: 1.0,
                color: [1.0, 0.0, 0.0, 0.8],
                through_walls: true,
                style: GlowStyle::Outline,
            },
            
            chams_settings: ChamsSettings {
                enabled: false,
                visible_color: [0.0, 1.0, 0.0, 0.7],
                hidden_color: [1.0, 0.0, 0.0, 0.5],
                material_type: ChamsMaterial::Flat,
                brightness: 1.0,
            },
            
            radar_settings: RadarSettings {
                enabled: false,
                position: Vector2::new(100.0, 100.0),
                size: 200.0,
                zoom: 1.0,
                show_enemies: true,
                show_teammates: true,
                show_weapons: false,
                show_grenades: false,
                rotation: true,
            },
            
            last_update: Instant::now(),
            frame_count: 0,
            
            visibility_history: HashMap::new(),
            velocity_history: HashMap::new(),
        }
    }
    
    /// Updates visibility tracking for a player
    fn update_visibility_tracking(&mut self, entity_id: u32, is_visible: bool) {
        let now = Instant::now();
        let history = self.visibility_history
            .entry(entity_id)
            .or_insert_with(Vec::new);
        
        history.push((is_visible, now));
        
        // Keep only last 100 entries or 30 seconds
        let cutoff = now - std::time::Duration::from_secs(30);
        history.retain(|(_, time)| *time > cutoff);
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }
    
    /// Updates velocity tracking for prediction
    fn update_velocity_tracking(&mut self, entity_id: u32, position: Vector3<f32>) {
        let now = Instant::now();
        let history = self.velocity_history
            .entry(entity_id)
            .or_insert_with(Vec::new);
        
        history.push((position, now));
        
        // Keep only last 20 entries or 5 seconds
        let cutoff = now - std::time::Duration::from_secs(5);
        history.retain(|(_, time)| *time > cutoff);
        if history.len() > 20 {
            history.drain(0..history.len() - 20);
        }
    }
    
    /// Calculates velocity from position history
    fn calculate_velocity(&self, entity_id: u32) -> Vector3<f32> {
        if let Some(history) = self.velocity_history.get(&entity_id) {
            if history.len() >= 2 {
                let (newest_pos, newest_time) = history.last().unwrap();
                let (oldest_pos, oldest_time) = history.first().unwrap();
                
                let delta_time = newest_time.duration_since(*oldest_time).as_secs_f32();
                if delta_time > 0.0 {
                    return (newest_pos - oldest_pos) / delta_time;
                }
            }
        }
        
        Vector3::zeros()
    }
    
    /// Calculates acceleration from velocity history
    fn calculate_acceleration(&self, entity_id: u32) -> Vector3<f32> {
        if let Some(history) = self.velocity_history.get(&entity_id) {
            if history.len() >= 3 {
                let len = history.len();
                let mid_idx = len / 2;
                
                let (end_pos, end_time) = &history[len - 1];
                let (mid_pos, mid_time) = &history[mid_idx];
                let (start_pos, start_time) = &history[0];
                
                let dt1 = mid_time.duration_since(*start_time).as_secs_f32();
                let dt2 = end_time.duration_since(*mid_time).as_secs_f32();
                
                if dt1 > 0.0 && dt2 > 0.0 {
                    let v1 = (mid_pos - start_pos) / dt1;
                    let v2 = (end_pos - mid_pos) / dt2;
                    return (v2 - v1) / ((dt1 + dt2) / 2.0);
                }
            }
        }
        
        Vector3::zeros()
    }
    
    /// Calculates visibility percentage over time
    fn calculate_visibility_percentage(&self, entity_id: u32) -> f32 {
        if let Some(history) = self.visibility_history.get(&entity_id) {
            if history.is_empty() {
                return 0.0;
            }
            
            let visible_count = history.iter()
                .filter(|(visible, _)| *visible)
                .count();
            
            visible_count as f32 / history.len() as f32
        } else {
            0.0
        }
    }
    
    /// Predicts future position based on velocity and acceleration
    fn predict_position(&self, current_pos: Vector3<f32>, velocity: Vector3<f32>, acceleration: Vector3<f32>, time_ahead: f32) -> Vector3<f32> {
        // s = ut + 0.5at²
        current_pos + velocity * time_ahead + 0.5 * acceleration * time_ahead * time_ahead
    }
    
    /// Renders enhanced player box with additional information
    fn render_enhanced_player_box(
        &self,
        draw: &DrawListMut,
        player: &AdvancedESPInfo,
        view: &ViewController,
        settings: &EspPlayerSettings,
        distance: f32,
    ) {
        let player_rel_health = (player.pawn_info.player_health as f32 / 100.0).clamp(0.0, 1.0);
        
        // Enhanced box with thickness variation based on health
        if let Some((vmin, vmax)) = view.calculate_box_2d(
            &(Vector3::new(-16.0, -16.0, 0.0) + player.pawn_info.position),
            &(Vector3::new(16.0, 16.0, 72.0) + player.pawn_info.position),
        ) {
            let box_color = settings.box_color.calculate_color(player_rel_health, distance);
            let thickness = settings.box_width * (0.5 + player_rel_health * 0.5);
            
            // Main box
            draw.add_rect([vmin.x, vmin.y], [vmax.x, vmax.y], box_color)
                .thickness(thickness)
                .build();
            
            // Corner highlights for better visibility
            let corner_size = 10.0;
            let corner_thickness = thickness + 1.0;
            
            // Top-left
            draw.add_line([vmin.x, vmin.y], [vmin.x + corner_size, vmin.y], box_color)
                .thickness(corner_thickness).build();
            draw.add_line([vmin.x, vmin.y], [vmin.x, vmin.y + corner_size], box_color)
                .thickness(corner_thickness).build();
            
            // Top-right
            draw.add_line([vmax.x, vmin.y], [vmax.x - corner_size, vmin.y], box_color)
                .thickness(corner_thickness).build();
            draw.add_line([vmax.x, vmin.y], [vmax.x, vmin.y + corner_size], box_color)
                .thickness(corner_thickness).build();
            
            // Bottom-left
            draw.add_line([vmin.x, vmax.y], [vmin.x + corner_size, vmax.y], box_color)
                .thickness(corner_thickness).build();
            draw.add_line([vmin.x, vmax.y], [vmin.x, vmax.y - corner_size], box_color)
                .thickness(corner_thickness).build();
            
            // Bottom-right
            draw.add_line([vmax.x, vmax.y], [vmax.x - corner_size, vmax.y], box_color)
                .thickness(corner_thickness).build();
            draw.add_line([vmax.x, vmax.y], [vmax.x, vmax.y - corner_size], box_color)
                .thickness(corner_thickness).build();
        }
    }
    
    /// Renders velocity prediction line
    fn render_velocity_prediction(
        &self,
        draw: &DrawListMut,
        player: &AdvancedESPInfo,
        view: &ViewController,
    ) {
        if player.velocity.norm() > 5.0 {
            let prediction_time = 1.0; // 1 second ahead
            let predicted_pos = self.predict_position(
                player.pawn_info.position,
                player.velocity,
                player.acceleration,
                prediction_time,
            );
            
            if let (Some(current_screen), Some(predicted_screen)) = (
                view.world_to_screen(&player.pawn_info.position, false),
                view.world_to_screen(&predicted_pos, false),
            ) {
                // Velocity line
                draw.add_line(
                    current_screen,
                    predicted_screen,
                    [1.0, 1.0, 0.0, 0.8], // Yellow
                ).thickness(2.0).build();
                
                // Prediction point
                draw.add_circle(predicted_screen, 3.0, [1.0, 0.5, 0.0, 1.0])
                    .filled(true).build();
            }
        }
    }
    
    /// Renders 2D radar
    fn render_2d_radar(
        &self,
        draw: &DrawListMut,
        view: &ViewController,
        local_pos: Vector3<f32>,
    ) {
        if !self.radar_settings.enabled {
            return;
        }
        
        let radar_center = self.radar_settings.position;
        let radar_size = self.radar_settings.size;
        let radar_radius = radar_size / 2.0;
        
        // Radar background
        draw.add_circle(radar_center, radar_radius, [0.0, 0.0, 0.0, 0.7])
            .filled(true).build();
        
        // Radar border
        draw.add_circle(radar_center, radar_radius, [1.0, 1.0, 1.0, 0.9])
            .filled(false).thickness(2.0).build();
        
        // Center point (local player)
        draw.add_circle(radar_center, 3.0, [0.0, 1.0, 0.0, 1.0])
            .filled(true).build();
        
        // Render players on radar
        for player in &self.players {
            let relative_pos = player.pawn_info.position - local_pos;
            let distance_2d = (relative_pos.x * relative_pos.x + relative_pos.y * relative_pos.y).sqrt();
            
            // Scale position to radar
            let radar_distance = (distance_2d / (2000.0 * self.radar_settings.zoom)).min(1.0) * radar_radius;
            let angle = relative_pos.y.atan2(relative_pos.x);
            
            let radar_pos = Vector2::new(
                radar_center.x + radar_distance * angle.cos(),
                radar_center.y + radar_distance * angle.sin(),
            );
            
            // Player color based on team
            let color = if player.pawn_info.team_id == self.local_team_id {
                [0.0, 0.0, 1.0, 1.0] // Blue for teammates
            } else {
                [1.0, 0.0, 0.0, 1.0] // Red for enemies
            };
            
            // Player dot
            draw.add_circle(radar_pos, 4.0, color)
                .filled(true).build();
            
            // Direction indicator
            if player.velocity.norm() > 50.0 {
                let vel_angle = player.velocity.y.atan2(player.velocity.x);
                let indicator_end = radar_pos + Vector2::new(
                    8.0 * vel_angle.cos(),
                    8.0 * vel_angle.sin(),
                );
                
                draw.add_line(radar_pos, indicator_end, color)
                    .thickness(2.0).build();
            }
        }
        
        // Radar grid
        let grid_color = [0.5, 0.5, 0.5, 0.3];
        for i in 1..=3 {
            let ring_radius = radar_radius * (i as f32 / 4.0);
            draw.add_circle(radar_center, ring_radius, grid_color)
                .filled(false).thickness(1.0).build();
        }
        
        // Cardinal directions
        let line_length = radar_radius * 0.9;
        draw.add_line(
            [radar_center.x - line_length, radar_center.y],
            [radar_center.x + line_length, radar_center.y],
            grid_color,
        ).thickness(1.0).build();
        
        draw.add_line(
            [radar_center.x, radar_center.y - line_length],
            [radar_center.x, radar_center.y + line_length],
            grid_color,
        ).thickness(1.0).build();
    }
    
    /// Renders glow effects (simplified - would need game memory modification)
    fn apply_glow_effects(&self, _player: &AdvancedESPInfo) {
        if !self.glow_settings.enabled {
            return;
        }
        
        // In a real implementation, this would modify game memory
        // to apply glow effects to player models
        // This is just a placeholder for the API
    }
    
    /// Renders chams effects (simplified - would need game memory modification)
    fn apply_chams_effects(&self, _player: &AdvancedESPInfo) {
        if !self.chams_settings.enabled {
            return;
        }
        
        // In a real implementation, this would modify game materials
        // to apply colored overlays to player models
        // This is just a placeholder for the API
    }
}

impl Enhancement for AdvancedESP {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(())?;
        let settings = ctx.states.resolve::<AppSettings>(())?;
        
        if self.toggle.update(&settings.esp_mode, ctx.input, &settings.esp_toogle) {
            ctx.cs2.add_metrics_record(
                "advanced-esp-toggle",
                &format!("enabled: {}", self.toggle.enabled),
            );
        }
        
        self.players.clear();
        if !self.toggle.enabled {
            return Ok(());
        }
        
        self.players.reserve(16);
        
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let local_player_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        let Some(local_player_controller) = local_player_controller
            .instance
            .value_reference(memory.view_arc())
        else {
            return Ok(());
        };
        
        self.local_team_id = local_player_controller.m_iPendingTeamNum()?;
        
        let view_target = ctx.states.resolve::<LocalCameraControllerTarget>(())?;
        let view_target_entity_id = match &view_target.target_entity_id {
            Some(value) => *value,
            None => return Ok(()),
        };
        
        for entity_identity in entities.entities() {
            if entity_identity.handle::<()>()?.get_entity_index() == view_target_entity_id {
                continue;
            }
            
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                continue;
            }
            
            let pawn_state = ctx
                .states
                .resolve::<PlayerPawnState>(entity_identity.handle()?)?;
            if *pawn_state != PlayerPawnState::Alive {
                continue;
            }
            
            let pawn_info = ctx
                .states
                .resolve::<StatePawnInfo>(entity_identity.handle()?)?;
            
            if pawn_info.player_health <= 0 || pawn_info.player_name.is_none() {
                continue;
            }
            
            let pawn_model = ctx
                .states
                .resolve::<StatePawnModelInfo>(entity_identity.handle()?)?;
            
            let entity_id = entity_identity.handle::<()>()?.get_entity_index();
            
            // Update tracking data
            self.update_velocity_tracking(entity_id, pawn_info.position);
            self.update_visibility_tracking(entity_id, true); // Simplified visibility
            
            // Calculate advanced info
            let velocity = self.calculate_velocity(entity_id);
            let acceleration = self.calculate_acceleration(entity_id);
            let visibility_percentage = self.calculate_visibility_percentage(entity_id);
            let predicted_position = self.predict_position(
                pawn_info.position,
                velocity,
                acceleration,
                0.5, // 0.5 seconds ahead
            );
            
            let advanced_info = AdvancedESPInfo {
                pawn_info: pawn_info.clone(),
                pawn_model: pawn_model.clone(),
                velocity,
                acceleration,
                is_scoped: false, // Would need to check weapon state
                is_defusing: false, // Would need to check player state
                is_planting: false, // Would need to check player state
                weapon_ammo: 30, // Would need to check weapon info
                armor: 100, // Would need to check player state
                money: 16000, // Would need to check player state
                rank: "Unknown".to_string(), // Would need to check player info
                last_seen: Instant::now(),
                visibility_percentage,
                predicted_position,
            };
            
            // Apply effects
            self.apply_glow_effects(&advanced_info);
            self.apply_chams_effects(&advanced_info);
            
            self.players.push(advanced_info);
        }
        
        self.frame_count += 1;
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        let view = states.resolve::<ViewController>(())?;
        
        let draw = ui.get_window_draw_list();
        
        let view_world_position = match view.get_camera_world_position() {
            Some(view_world_position) => view_world_position,
            _ => return Ok(()),
        };
        
        // Render 2D radar first (background element)
        self.render_2d_radar(&draw, &view, view_world_position);
        
        // Render enhanced ESP for each player
        for player in &self.players {
            let distance = (player.pawn_info.position - view_world_position).norm() * 0.01905; // Convert to meters
            
            // Get ESP settings for this player type
            let esp_target = EspSelector::PlayerTeam {
                enemy: player.pawn_info.team_id != self.local_team_id,
            };
            
            let config_key = esp_target.config_key();
            
            if !settings
                .esp_settings_enabled
                .get(&config_key)
                .cloned()
                .unwrap_or_default()
            {
                continue;
            }
            
            let esp_settings = match settings.esp_settings.get(&config_key) {
                Some(EspConfig::Player(settings)) => settings,
                _ => continue,
            };
            
            // Render enhanced player box
            self.render_enhanced_player_box(&draw, player, &view, esp_settings, distance);
            
            // Render velocity prediction
            self.render_velocity_prediction(&draw, player, &view);
            
            // Render additional info with enhanced details
            if let Some((vmin, vmax)) = view.calculate_box_2d(
                &(Vector3::new(-16.0, -16.0, 0.0) + player.pawn_info.position),
                &(Vector3::new(16.0, 16.0, 72.0) + player.pawn_info.position),
            ) {
                let mut info_y = vmin.y - 5.0;
                let info_color = [1.0, 1.0, 1.0, 1.0];
                
                // Enhanced name display with rank
                if esp_settings.info_name {
                    let display_name = format!(
                        "{} [{}]",
                        player.pawn_info.player_name.as_ref().unwrap_or(&"Unknown".to_string()),
                        player.rank
                    );
                    
                    // Would render text here - simplified for this example
                    info_y -= 15.0;
                }
                
                // Velocity display
                if player.velocity.norm() > 10.0 {
                    let speed = player.velocity.norm() * 0.01905; // Convert to units/sec
                    // Would render speed text here
                    info_y -= 15.0;
                }
                
                // Weapon and ammo info
                if esp_settings.info_weapon {
                    let weapon_info = format!("{} [{}/{}]", 
                        player.pawn_info.weapon.display_name(),
                        player.weapon_ammo,
                        100 // Max ammo placeholder
                    );
                    // Would render weapon info here
                    info_y -= 15.0;
                }
                
                // Special states
                let mut states = Vec::new();
                if player.is_scoped { states.push("SCOPED"); }
                if player.is_defusing { states.push("DEFUSING"); }
                if player.is_planting { states.push("PLANTING"); }
                
                if !states.is_empty() {
                    let state_text = states.join(" | ");
                    // Would render state text here
                    info_y -= 15.0;
                }
                
                // Visibility percentage
                if player.visibility_percentage < 1.0 {
                    let visibility_text = format!("VIS: {:.0}%", player.visibility_percentage * 100.0);
                    // Would render visibility text here
                }
            }
        }
        
        Ok(())
    }
}