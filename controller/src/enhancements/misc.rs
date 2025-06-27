use std::time::{Duration, Instant};

use anyhow::Context;
use cs2::{
    MouseState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
};
use cs2_schema_generated::cs2::client::C_CSPlayerPawn;
use nalgebra::{Vector3, Vector4};
use overlay::UnicodeTextRenderer;
use rand::{thread_rng, Rng};
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::KeyToggle,
    UpdateContext,
};

/// Anti-flash protection system
pub struct AntiFlash {
    pub enabled: bool,
    pub protection_level: f32, // 0.0 = no protection, 1.0 = full protection
    pub fade_time: Duration,
    pub last_flash: Instant,
}

impl AntiFlash {
    pub fn new() -> Self {
        Self {
            enabled: false,
            protection_level: 0.8,
            fade_time: Duration::from_millis(200),
            last_flash: Instant::now(),
        }
    }
    
    /// Reduces flash effect intensity
    pub fn reduce_flash_effect(&mut self, flash_alpha: f32) -> f32 {
        if !self.enabled {
            return flash_alpha;
        }
        
        let reduction = flash_alpha * self.protection_level;
        self.last_flash = Instant::now();
        
        flash_alpha - reduction
    }
    
    /// Checks if player is currently flashed
    pub fn is_flashed(&self) -> bool {
        self.last_flash.elapsed() < self.fade_time
    }
}

/// Speed hack system for movement enhancement
pub struct SpeedHack {
    pub enabled: bool,
    pub speed_multiplier: f32,
    pub stealth_mode: bool,
    pub random_variation: bool,
    pub last_update: Instant,
    pub variation_seed: u64,
}

impl SpeedHack {
    pub fn new() -> Self {
        Self {
            enabled: false,
            speed_multiplier: 1.2,
            stealth_mode: true,
            random_variation: true,
            last_update: Instant::now(),
            variation_seed: thread_rng().gen(),
        }
    }
    
    /// Gets the current speed multiplier with variations
    pub fn get_speed_multiplier(&mut self) -> f32 {
        if !self.enabled {
            return 1.0;
        }
        
        let mut multiplier = self.speed_multiplier;
        
        if self.random_variation {
            let mut rng = thread_rng();
            let variation = rng.gen_range(-0.1..0.1);
            multiplier += variation;
        }
        
        if self.stealth_mode {
            // Limit speed to avoid detection
            multiplier = multiplier.min(1.3);
        }
        
        multiplier.max(1.0)
    }
    
    /// Applies speed modification (simplified - would modify game memory)
    pub fn apply_speed_modification(&mut self, _current_velocity: &Vector3<f32>) {
        if !self.enabled {
            return;
        }
        
        // In a real implementation, this would modify the player's velocity
        // or movement speed in game memory
        self.last_update = Instant::now();
    }
}

/// No-recoil system (enhanced version of existing aim punch compensation)
pub struct NoRecoil {
    pub enabled: bool,
    pub compensation_factor: f32,
    pub smoothing: bool,
    pub randomization: bool,
    pub last_compensation: Vector4<f32>,
    pub compensation_history: Vec<Vector4<f32>>,
}

impl NoRecoil {
    pub fn new() -> Self {
        Self {
            enabled: false,
            compensation_factor: 1.0,
            smoothing: true,
            randomization: false,
            last_compensation: Vector4::zeros(),
            compensation_history: Vec::new(),
        }
    }
    
    /// Calculates recoil compensation with advanced features
    pub fn calculate_compensation(&mut self, punch_angle: Vector4<f32>) -> Vector4<f32> {
        if !self.enabled {
            return Vector4::zeros();
        }
        
        let mut compensation = punch_angle * self.compensation_factor;
        
        // Apply smoothing
        if self.smoothing && !self.compensation_history.is_empty() {
            let last_comp = self.compensation_history.last().unwrap();
            compensation = last_comp * 0.3 + compensation * 0.7;
        }
        
        // Apply randomization for human-like behavior
        if self.randomization {
            let mut rng = thread_rng();
            let noise = Vector4::new(
                rng.gen_range(-0.02..0.02),
                rng.gen_range(-0.02..0.02),
                0.0,
                0.0,
            );
            compensation += noise;
        }
        
        // Store in history
        self.compensation_history.push(compensation);
        if self.compensation_history.len() > 10 {
            self.compensation_history.remove(0);
        }
        
        self.last_compensation = compensation;
        compensation
    }
}

/// No-spread system for weapon accuracy enhancement
pub struct NoSpread {
    pub enabled: bool,
    pub compensation_factor: f32,
    pub weapon_specific: bool,
    pub current_weapon: String,
    pub spread_patterns: std::collections::HashMap<String, Vec<Vector4<f32>>>,
}

impl NoSpread {
    pub fn new() -> Self {
        let mut spread_patterns = std::collections::HashMap::new();
        
        // Add basic spread patterns for common weapons
        spread_patterns.insert("weapon_ak47".to_string(), Self::ak47_pattern());
        spread_patterns.insert("weapon_m4a4".to_string(), Self::m4a4_pattern());
        
        Self {
            enabled: false,
            compensation_factor: 1.0,
            weapon_specific: true,
            current_weapon: String::new(),
            spread_patterns,
        }
    }
    
    /// AK-47 recoil pattern
    fn ak47_pattern() -> Vec<Vector4<f32>> {
        vec![
            Vector4::new(0.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, -0.89, 0.0, 0.0),
            Vector4::new(-0.2, -0.9, 0.0, 0.0),
            Vector4::new(-0.2, -0.96, 0.0, 0.0),
            Vector4::new(-0.3, -1.0, 0.0, 0.0),
            // Add more pattern data...
        ]
    }
    
    /// M4A4 recoil pattern
    fn m4a4_pattern() -> Vec<Vector4<f32>> {
        vec![
            Vector4::new(0.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, -0.6, 0.0, 0.0),
            Vector4::new(-0.1, -0.7, 0.0, 0.0),
            Vector4::new(-0.15, -0.75, 0.0, 0.0),
            Vector4::new(-0.2, -0.8, 0.0, 0.0),
            // Add more pattern data...
        ]
    }
    
    /// Gets spread compensation for current shot
    pub fn get_spread_compensation(&self, shot_number: usize) -> Vector4<f32> {
        if !self.enabled {
            return Vector4::zeros();
        }
        
        if self.weapon_specific {
            if let Some(pattern) = self.spread_patterns.get(&self.current_weapon) {
                if shot_number < pattern.len() {
                    return pattern[shot_number] * self.compensation_factor;
                }
            }
        }
        
        Vector4::zeros()
    }
}

/// Auto-pistol system for rapid firing
pub struct AutoPistol {
    pub enabled: bool,
    pub fire_rate: f32, // Shots per second
    pub last_shot: Instant,
    pub randomized_timing: bool,
    pub weapon_filter: Vec<String>,
}

impl AutoPistol {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fire_rate: 10.0, // 10 shots per second
            last_shot: Instant::now(),
            randomized_timing: true,
            weapon_filter: vec![
                "weapon_glock".to_string(),
                "weapon_usp_silencer".to_string(),
                "weapon_p250".to_string(),
                "weapon_fiveseven".to_string(),
                "weapon_cz75a".to_string(),
                "weapon_tec9".to_string(),
                "weapon_p2000".to_string(),
            ],
        }
    }
    
    /// Checks if auto-fire should be triggered
    pub fn should_fire(&mut self, current_weapon: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check weapon filter
        if !self.weapon_filter.iter().any(|w| current_weapon.contains(w)) {
            return false;
        }
        
        // Check fire rate timing
        let base_interval = Duration::from_secs_f32(1.0 / self.fire_rate);
        let interval = if self.randomized_timing {
            let mut rng = thread_rng();
            let variation = rng.gen_range(0.9..1.1);
            Duration::from_secs_f32(base_interval.as_secs_f32() * variation)
        } else {
            base_interval
        };
        
        if self.last_shot.elapsed() >= interval {
            self.last_shot = Instant::now();
            return true;
        }
        
        false
    }
}

/// Fake lag system for network manipulation
pub struct FakeLag {
    pub enabled: bool,
    pub lag_amount: u32, // Milliseconds
    pub randomized: bool,
    pub adaptive: bool,
    pub packet_queue: Vec<(Instant, Vec<u8>)>,
}

impl FakeLag {
    pub fn new() -> Self {
        Self {
            enabled: false,
            lag_amount: 50,
            randomized: true,
            adaptive: false,
            packet_queue: Vec::new(),
        }
    }
    
    /// Processes packet queue and determines if packet should be sent
    pub fn should_send_packet(&mut self) -> bool {
        if !self.enabled {
            return true;
        }
        
        let mut lag = self.lag_amount;
        
        if self.randomized {
            let mut rng = thread_rng();
            lag = rng.gen_range((lag / 2)..=(lag * 3 / 2));
        }
        
        if self.adaptive {
            // Adjust lag based on network conditions (simplified)
            lag = (lag as f32 * 1.2) as u32;
        }
        
        // Simplified packet delay logic
        let now = Instant::now();
        self.packet_queue.retain(|(time, _)| {
            now.duration_since(*time) < Duration::from_millis(lag as u64)
        });
        
        // Return true if we should send (simplified)
        self.packet_queue.len() < 3
    }
}

/// Backtrack system for lag compensation exploitation
pub struct Backtrack {
    pub enabled: bool,
    pub max_time: Duration,
    pub player_records: std::collections::HashMap<u32, Vec<PlayerRecord>>,
}

#[derive(Debug, Clone)]
pub struct PlayerRecord {
    pub position: Vector3<f32>,
    pub angles: Vector3<f32>,
    pub timestamp: Instant,
    pub valid: bool,
}

impl Backtrack {
    pub fn new() -> Self {
        Self {
            enabled: false,
            max_time: Duration::from_millis(200),
            player_records: std::collections::HashMap::new(),
        }
    }
    
    /// Records player position for backtracking
    pub fn record_player(&mut self, entity_id: u32, position: Vector3<f32>, angles: Vector3<f32>) {
        if !self.enabled {
            return;
        }
        
        let record = PlayerRecord {
            position,
            angles,
            timestamp: Instant::now(),
            valid: true,
        };
        
        let records = self.player_records.entry(entity_id).or_insert_with(Vec::new);
        records.push(record);
        
        // Clean old records
        let cutoff = Instant::now() - self.max_time;
        records.retain(|r| r.timestamp > cutoff);
        
        // Limit record count
        if records.len() > 64 {
            records.remove(0);
        }
    }
    
    /// Gets the best backtrack record for a target
    pub fn get_best_record(&self, entity_id: u32) -> Option<&PlayerRecord> {
        if !self.enabled {
            return None;
        }
        
        self.player_records.get(&entity_id)?
            .iter()
            .filter(|r| r.valid)
            .min_by_key(|r| r.timestamp)
    }
}

/// Combined miscellaneous features enhancement
pub struct MiscFeatures {
    // Feature toggles
    anti_flash: AntiFlash,
    speed_hack: SpeedHack,
    no_recoil: NoRecoil,
    no_spread: NoSpread,
    auto_pistol: AutoPistol,
    fake_lag: FakeLag,
    backtrack: Backtrack,
    
    // Additional features
    pub auto_defuse: bool,
    pub fast_plant: bool,
    pub instant_defuse: bool,
    pub bomb_timer_prediction: bool,
    
    // Performance tracking
    last_update: Instant,
    feature_usage_stats: std::collections::HashMap<String, u32>,
}

impl MiscFeatures {
    pub fn new() -> Self {
        Self {
            anti_flash: AntiFlash::new(),
            speed_hack: SpeedHack::new(),
            no_recoil: NoRecoil::new(),
            no_spread: NoSpread::new(),
            auto_pistol: AutoPistol::new(),
            fake_lag: FakeLag::new(),
            backtrack: Backtrack::new(),
            
            auto_defuse: false,
            fast_plant: false,
            instant_defuse: false,
            bomb_timer_prediction: true,
            
            last_update: Instant::now(),
            feature_usage_stats: std::collections::HashMap::new(),
        }
    }
    
    /// Updates all miscellaneous features
    fn update_features(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        // Update player state for various features
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        
        if let Some(local_controller) = local_controller.instance.value_reference(memory.view_arc()) {
            let local_pawn_handle = local_controller.m_hPlayerPawn()?;
            let entities = ctx.states.resolve::<StateEntityList>(())?;
            
            if let Ok(local_pawn_entity) = entities.entity_from_handle(&local_pawn_handle) {
                if let Some(local_pawn) = local_pawn_entity.value_reference(memory.view_arc()) {
                    // Update anti-flash
                    if self.anti_flash.enabled {
                        let flash_alpha = 0.0; // Would get from player state
                        self.anti_flash.reduce_flash_effect(flash_alpha);
                        self.increment_usage_stat("anti_flash");
                    }
                    
                    // Update speed hack
                    if self.speed_hack.enabled {
                        let velocity_array = local_pawn.m_vecAbsVelocity()?;
                        let velocity = Vector3::new(velocity_array[0], velocity_array[1], velocity_array[2]);
                        self.speed_hack.apply_speed_modification(&velocity);
                        self.increment_usage_stat("speed_hack");
                    }
                    
                    // Update no-recoil
                    if self.no_recoil.enabled {
                        let punch_angle = Vector4::from_row_slice(&local_pawn.m_aimPunchAngle()?);
                        let compensation = self.no_recoil.calculate_compensation(punch_angle);
                        
                        // Apply compensation (would send mouse input)
                        if compensation.norm() > 0.1 {
                            self.send_recoil_compensation(compensation, ctx)?;
                            self.increment_usage_stat("no_recoil");
                        }
                    }
                    
                    // Update auto-pistol
                    if self.auto_pistol.enabled {
                        let current_weapon = "weapon_glock"; // Would get from weapon state
                        if self.auto_pistol.should_fire(current_weapon) {
                            self.send_auto_fire(ctx)?;
                            self.increment_usage_stat("auto_pistol");
                        }
                    }
                }
            }
        }
        
        // Update backtrack records for all players
        if self.backtrack.enabled {
            self.update_backtrack_records(ctx)?;
        }
        
        self.last_update = Instant::now();
        Ok(())
    }
    
    /// Updates backtrack records for all players
    fn update_backtrack_records(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        
        for entity_identity in entities.entities() {
            let entity_id = entity_identity.handle::<()>()?.get_entity_index();
            
            // Get position and angles (simplified)
            let position = Vector3::new(0.0, 0.0, 0.0); // Would get from entity
            let angles = Vector3::new(0.0, 0.0, 0.0);   // Would get from entity
            
            self.backtrack.record_player(entity_id, position, angles);
        }
        
        Ok(())
    }
    
    /// Sends recoil compensation mouse input
    fn send_recoil_compensation(&self, compensation: Vector4<f32>, ctx: &UpdateContext) -> anyhow::Result<()> {
        let mouse_sensitivity = 1.0; // Would get from game settings
        let mouse_x = (compensation.y / (mouse_sensitivity * 0.022)).round() as i32;
        let mouse_y = (compensation.x / (mouse_sensitivity * 0.022)).round() as i32;
        
        ctx.cs2.send_mouse_state(&[MouseState {
            last_x: -mouse_x,
            last_y: -mouse_y,
            ..Default::default()
        }])?;
        
        Ok(())
    }
    
    /// Sends auto-fire input
    fn send_auto_fire(&self, ctx: &UpdateContext) -> anyhow::Result<()> {
        ctx.cs2.send_mouse_state(&[MouseState {
            left_click: true,
            ..Default::default()
        }])?;
        
        Ok(())
    }
    
    /// Increments usage statistics
    fn increment_usage_stat(&mut self, feature: &str) {
        *self.feature_usage_stats.entry(feature.to_string()).or_insert(0) += 1;
    }
    
    /// Gets usage statistics
    pub fn get_usage_stats(&self) -> &std::collections::HashMap<String, u32> {
        &self.feature_usage_stats
    }
    
    /// Resets usage statistics
    pub fn reset_usage_stats(&mut self) {
        self.feature_usage_stats.clear();
    }
}

impl Enhancement for MiscFeatures {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let _settings = ctx.states.resolve::<AppSettings>(())?;
        
        // Update all miscellaneous features
        self.update_features(ctx)?;
        
        Ok(())
    }
    
    fn render(
        &self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        // Could render feature status indicators
        Ok(())
    }
}

/// Configuration structure for miscellaneous features
#[derive(Debug, Clone)]
pub struct MiscConfig {
    // Anti-flash settings
    pub anti_flash_enabled: bool,
    pub anti_flash_protection: f32,
    
    // Speed hack settings
    pub speed_hack_enabled: bool,
    pub speed_multiplier: f32,
    pub speed_stealth_mode: bool,
    
    // No-recoil settings
    pub no_recoil_enabled: bool,
    pub recoil_compensation_factor: f32,
    pub recoil_smoothing: bool,
    
    // No-spread settings
    pub no_spread_enabled: bool,
    pub spread_compensation: f32,
    
    // Auto-pistol settings
    pub auto_pistol_enabled: bool,
    pub pistol_fire_rate: f32,
    
    // Network features
    pub fake_lag_enabled: bool,
    pub fake_lag_amount: u32,
    pub backtrack_enabled: bool,
    pub backtrack_time: u32,
    
    // Utility features
    pub auto_defuse: bool,
    pub fast_plant: bool,
    pub bomb_timer: bool,
}

impl Default for MiscConfig {
    fn default() -> Self {
        Self {
            anti_flash_enabled: false,
            anti_flash_protection: 0.8,
            
            speed_hack_enabled: false,
            speed_multiplier: 1.2,
            speed_stealth_mode: true,
            
            no_recoil_enabled: false,
            recoil_compensation_factor: 1.0,
            recoil_smoothing: true,
            
            no_spread_enabled: false,
            spread_compensation: 1.0,
            
            auto_pistol_enabled: false,
            pistol_fire_rate: 10.0,
            
            fake_lag_enabled: false,
            fake_lag_amount: 50,
            backtrack_enabled: false,
            backtrack_time: 200,
            
            auto_defuse: false,
            fast_plant: false,
            bomb_timer: true,
        }
    }
}