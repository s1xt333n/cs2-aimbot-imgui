use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    math::smoothing::SmoothingType,
    settings::{HotKey, KeyToggleMode},
};

/// Comprehensive aimbot configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AimbotConfig {
    // Core settings
    #[serde(default = "bool_false")]
    pub enabled: bool,
    
    #[serde(default = "default_toggle_mode")]
    pub toggle_mode: KeyToggleMode,
    
    #[serde(default)]
    pub hotkey: Option<HotKey>,
    
    // Targeting settings
    #[serde(default = "default_fov")]
    pub fov: f32,
    
    #[serde(default = "default_max_distance")]
    pub max_distance: f32,
    
    #[serde(default = "bool_true")]
    pub visible_only: bool,
    
    #[serde(default = "bool_true")]
    pub team_check: bool,
    
    #[serde(default = "bool_true")]
    pub prefer_head: bool,
    
    // Smoothing settings
    #[serde(default = "bool_true")]
    pub smooth_enabled: bool,
    
    #[serde(default = "default_smooth_factor")]
    pub smooth_factor: f32,
    
    #[serde(default = "default_smoothing_type")]
    pub smoothing_type: SmoothingType,
    
    #[serde(default = "default_max_smooth_speed")]
    pub max_smooth_speed: f32,
    
    // Prediction settings
    #[serde(default = "bool_true")]
    pub prediction_enabled: bool,
    
    #[serde(default = "default_prediction_factor")]
    pub prediction_factor: f32,
    
    #[serde(default = "bool_false")]
    pub gravity_compensation: bool,
    
    // Auto-shooting settings
    #[serde(default = "bool_false")]
    pub auto_shoot: bool,
    
    #[serde(default = "default_shot_delay")]
    pub shot_delay: u32,
    
    #[serde(default = "default_shot_duration")]
    pub shot_duration: u32,
    
    #[serde(default = "default_acquisition_delay")]
    pub acquisition_delay: u32,
    
    // Target priority weights
    #[serde(default = "default_fov_weight")]
    pub fov_weight: f32,
    
    #[serde(default = "default_distance_weight")]
    pub distance_weight: f32,
    
    #[serde(default = "default_health_weight")]
    pub health_weight: f32,
    
    #[serde(default = "default_hitbox_weight")]
    pub hitbox_weight: f32,
    
    #[serde(default = "default_visibility_bonus")]
    pub visibility_bonus: f32,
    
    #[serde(default = "default_time_penalty")]
    pub time_penalty: f32,
    
    // Humanization settings
    #[serde(default = "bool_true")]
    pub humanization_enabled: bool,
    
    #[serde(default = "default_reaction_time")]
    pub reaction_time: f32,
    
    #[serde(default = "default_micro_corrections")]
    pub micro_corrections: bool,
    
    #[serde(default = "default_overshoot_chance")]
    pub overshoot_chance: f32,
    
    // Anti-detection settings
    #[serde(default = "bool_false")]
    pub legit_mode: bool,
    
    #[serde(default = "default_miss_chance")]
    pub miss_chance: f32,
    
    #[serde(default = "default_min_shot_delay")]
    pub min_shot_delay: u64,
    
    #[serde(default = "default_max_consecutive_headshots")]
    pub max_consecutive_headshots: u32,
    
    #[serde(default = "bool_false")]
    pub randomize_timing: bool,
    
    // Silent aim settings
    #[serde(default = "bool_false")]
    pub silent_aim: bool,
    
    #[serde(default = "default_silent_fov")]
    pub silent_fov: f32,
    
    // Bone scan settings
    #[serde(default = "bool_false")]
    pub bone_scan: bool,
    
    #[serde(default = "default_bone_scan_delay")]
    pub bone_scan_delay: u32,
    
    // Penetration settings
    #[serde(default = "bool_false")]
    pub auto_penetration: bool,
    
    #[serde(default = "default_min_damage")]
    pub min_damage: f32,
    
    // RCS (Recoil Control System) integration
    #[serde(default = "bool_true")]
    pub rcs_integration: bool,
    
    #[serde(default = "default_rcs_factor")]
    pub rcs_factor: f32,
    
    // Dynamic FOV
    #[serde(default = "bool_false")]
    pub dynamic_fov: bool,
    
    #[serde(default = "default_fov_min")]
    pub fov_min: f32,
    
    #[serde(default = "default_fov_max")]
    pub fov_max: f32,
    
    // Weapon-specific settings
    #[serde(default)]
    pub weapon_configs: std::collections::HashMap<String, WeaponAimbotConfig>,
}

/// Weapon-specific aimbot configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeaponAimbotConfig {
    #[serde(default = "default_smooth_factor")]
    pub smooth_factor: f32,
    
    #[serde(default = "default_fov")]
    pub fov: f32,
    
    #[serde(default = "bool_true")]
    pub prediction_enabled: bool,
    
    #[serde(default = "bool_false")]
    pub auto_shoot: bool,
    
    #[serde(default = "default_shot_delay")]
    pub shot_delay: u32,
    
    #[serde(default = "bool_true")]
    pub prefer_head: bool,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            toggle_mode: default_toggle_mode(),
            hotkey: None,
            
            fov: default_fov(),
            max_distance: default_max_distance(),
            visible_only: true,
            team_check: true,
            prefer_head: true,
            
            smooth_enabled: true,
            smooth_factor: default_smooth_factor(),
            smoothing_type: default_smoothing_type(),
            max_smooth_speed: default_max_smooth_speed(),
            
            prediction_enabled: true,
            prediction_factor: default_prediction_factor(),
            gravity_compensation: false,
            
            auto_shoot: false,
            shot_delay: default_shot_delay(),
            shot_duration: default_shot_duration(),
            acquisition_delay: default_acquisition_delay(),
            
            fov_weight: default_fov_weight(),
            distance_weight: default_distance_weight(),
            health_weight: default_health_weight(),
            hitbox_weight: default_hitbox_weight(),
            visibility_bonus: default_visibility_bonus(),
            time_penalty: default_time_penalty(),
            
            humanization_enabled: true,
            reaction_time: default_reaction_time(),
            micro_corrections: default_micro_corrections(),
            overshoot_chance: default_overshoot_chance(),
            
            legit_mode: false,
            miss_chance: default_miss_chance(),
            min_shot_delay: default_min_shot_delay(),
            max_consecutive_headshots: default_max_consecutive_headshots(),
            randomize_timing: false,
            
            silent_aim: false,
            silent_fov: default_silent_fov(),
            
            bone_scan: false,
            bone_scan_delay: default_bone_scan_delay(),
            
            auto_penetration: false,
            min_damage: default_min_damage(),
            
            rcs_integration: true,
            rcs_factor: default_rcs_factor(),
            
            dynamic_fov: false,
            fov_min: default_fov_min(),
            fov_max: default_fov_max(),
            
            weapon_configs: std::collections::HashMap::new(),
        }
    }
}

impl Default for WeaponAimbotConfig {
    fn default() -> Self {
        Self {
            smooth_factor: default_smooth_factor(),
            fov: default_fov(),
            prediction_enabled: true,
            auto_shoot: false,
            shot_delay: default_shot_delay(),
            prefer_head: true,
        }
    }
}

// Default value functions
fn bool_false() -> bool { false }
fn bool_true() -> bool { true }

fn default_toggle_mode() -> KeyToggleMode { KeyToggleMode::Hold }
fn default_fov() -> f32 { 5.0 }
fn default_max_distance() -> f32 { 5000.0 }

fn default_smooth_factor() -> f32 { 2.0 }
fn default_smoothing_type() -> SmoothingType { SmoothingType::Natural }
fn default_max_smooth_speed() -> f32 { 1000.0 }

fn default_prediction_factor() -> f32 { 1.0 }

fn default_shot_delay() -> u32 { 50 }
fn default_shot_duration() -> u32 { 100 }
fn default_acquisition_delay() -> u32 { 100 }

fn default_fov_weight() -> f32 { 3.0 }
fn default_distance_weight() -> f32 { 1.0 }
fn default_health_weight() -> f32 { 0.5 }
fn default_hitbox_weight() -> f32 { 2.0 }
fn default_visibility_bonus() -> f32 { 5.0 }
fn default_time_penalty() -> f32 { 2.0 }

fn default_reaction_time() -> f32 { 0.1 }
fn default_micro_corrections() -> bool { true }
fn default_overshoot_chance() -> f32 { 0.15 }

fn default_miss_chance() -> f32 { 0.02 }
fn default_min_shot_delay() -> u64 { 50 }
fn default_max_consecutive_headshots() -> u32 { 5 }

fn default_silent_fov() -> f32 { 1.0 }

fn default_bone_scan_delay() -> u32 { 10 }

fn default_min_damage() -> f32 { 30.0 }

fn default_rcs_factor() -> f32 { 0.8 }

fn default_fov_min() -> f32 { 1.0 }
fn default_fov_max() -> f32 { 10.0 }

/// Extended aimbot settings for advanced features
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdvancedAimbotSettings {
    // Multi-point aimbot
    #[serde(default = "bool_false")]
    pub multi_point: bool,
    
    #[serde(default = "default_point_scale")]
    pub point_scale: f32,
    
    // Backtrack
    #[serde(default = "bool_false")]
    pub backtrack: bool,
    
    #[serde(default = "default_backtrack_time")]
    pub backtrack_time: f32,
    
    // Resolver
    #[serde(default = "bool_false")]
    pub resolver: bool,
    
    #[serde(default = "default_resolver_type")]
    pub resolver_type: ResolverType,
    
    // Hitchance
    #[serde(default = "default_hitchance")]
    pub hitchance: f32,
    
    // Minimum damage override
    #[serde(default)]
    pub damage_overrides: std::collections::HashMap<String, f32>,
    
    // Auto scope
    #[serde(default = "bool_false")]
    pub auto_scope: bool,
    
    // Auto stop
    #[serde(default = "bool_false")]
    pub auto_stop: bool,
    
    #[serde(default = "default_stop_type")]
    pub stop_type: AutoStopType,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ResolverType {
    Bruteforce,
    Smart,
    Adaptive,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum AutoStopType {
    Full,
    Minimal,
    Early,
}

impl Default for AdvancedAimbotSettings {
    fn default() -> Self {
        Self {
            multi_point: false,
            point_scale: default_point_scale(),
            backtrack: false,
            backtrack_time: default_backtrack_time(),
            resolver: false,
            resolver_type: default_resolver_type(),
            hitchance: default_hitchance(),
            damage_overrides: std::collections::HashMap::new(),
            auto_scope: false,
            auto_stop: false,
            stop_type: default_stop_type(),
        }
    }
}

fn default_point_scale() -> f32 { 0.7 }
fn default_backtrack_time() -> f32 { 200.0 }
fn default_resolver_type() -> ResolverType { ResolverType::Smart }
fn default_hitchance() -> f32 { 60.0 }
fn default_stop_type() -> AutoStopType { AutoStopType::Full }

/// Aimbot profiles for different scenarios
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AimbotProfile {
    pub name: String,
    pub config: AimbotConfig,
    pub advanced: AdvancedAimbotSettings,
    pub description: String,
}

impl AimbotProfile {
    pub fn legit_profile() -> Self {
        let mut config = AimbotConfig::default();
        config.legit_mode = true;
        config.smooth_factor = 5.0;
        config.fov = 2.0;
        config.humanization_enabled = true;
        config.miss_chance = 0.05;
        config.max_consecutive_headshots = 3;
        config.randomize_timing = true;
        
        Self {
            name: "Legit".to_string(),
            config,
            advanced: AdvancedAimbotSettings::default(),
            description: "Safe settings for matchmaking".to_string(),
        }
    }
    
    pub fn rage_profile() -> Self {
        let mut config = AimbotConfig::default();
        config.legit_mode = false;
        config.smooth_factor = 1.0;
        config.fov = 180.0;
        config.auto_shoot = true;
        config.silent_aim = true;
        config.auto_penetration = true;
        
        let mut advanced = AdvancedAimbotSettings::default();
        advanced.multi_point = true;
        advanced.backtrack = true;
        advanced.resolver = true;
        advanced.auto_stop = true;
        
        Self {
            name: "Rage".to_string(),
            config,
            advanced,
            description: "Aggressive settings for HvH".to_string(),
        }
    }
    
    pub fn semi_rage_profile() -> Self {
        let mut config = AimbotConfig::default();
        config.legit_mode = false;
        config.smooth_factor = 2.5;
        config.fov = 8.0;
        config.auto_shoot = false;
        config.humanization_enabled = true;
        config.miss_chance = 0.02;
        
        let mut advanced = AdvancedAimbotSettings::default();
        advanced.backtrack = true;
        advanced.auto_stop = true;
        
        Self {
            name: "Semi-Rage".to_string(),
            config,
            advanced,
            description: "Balanced settings for semi-rage play".to_string(),
        }
    }
}