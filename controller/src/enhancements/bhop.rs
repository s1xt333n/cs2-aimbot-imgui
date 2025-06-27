use std::time::{Duration, Instant};

use anyhow::Context;
use cs2::{
    MouseState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
};
use cs2_schema_generated::cs2::client::C_CSPlayerPawn;
use nalgebra::Vector3;
use overlay::UnicodeTextRenderer;
use rand::{thread_rng, Rng};
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::KeyToggle,
    UpdateContext,
};

/// Bunny hop state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum BhopState {
    OnGround,
    InAir,
    Landing,
    PreJump,
}

/// Movement synchronization for air strafing
#[derive(Debug, Clone)]
pub struct StrafeSync {
    pub mouse_movement: f32,
    pub key_press: StrafeKey,
    pub sync_percentage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrafeKey {
    None,
    A,
    D,
    W,
    S,
}

/// Advanced bunny hop automation with perfect synchronization
pub struct AdvancedBhop {
    // Core components
    toggle: KeyToggle,
    state: BhopState,
    
    // Movement tracking
    velocity: Vector3<f32>,
    last_velocity: Vector3<f32>,
    ground_time: Duration,
    air_time: Duration,
    
    // Timing optimization
    last_jump_time: Instant,
    last_ground_time: Instant,
    last_air_time: Instant,
    
    // Synchronization
    current_strafe: StrafeSync,
    strafe_history: Vec<StrafeSync>,
    
    // Performance tracking
    jumps_hit: u32,
    jumps_total: u32,
    best_speed: f32,
    current_speed: f32,
    
    // Anti-detection
    humanization_enabled: bool,
    miss_chance: f32,
    timing_variation: f32,
    last_miss: Instant,
    
    // Configuration
    perfect_sync: bool,
    auto_strafe: bool,
    edge_bug_detection: bool,
    max_speed_limit: f32,
}

impl AdvancedBhop {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            state: BhopState::OnGround,
            
            velocity: Vector3::zeros(),
            last_velocity: Vector3::zeros(),
            ground_time: Duration::ZERO,
            air_time: Duration::ZERO,
            
            last_jump_time: Instant::now(),
            last_ground_time: Instant::now(),
            last_air_time: Instant::now(),
            
            current_strafe: StrafeSync {
                mouse_movement: 0.0,
                key_press: StrafeKey::None,
                sync_percentage: 0.0,
            },
            strafe_history: Vec::new(),
            
            jumps_hit: 0,
            jumps_total: 0,
            best_speed: 0.0,
            current_speed: 0.0,
            
            humanization_enabled: true,
            miss_chance: 0.02,
            timing_variation: 0.05,
            last_miss: Instant::now(),
            
            perfect_sync: false,
            auto_strafe: true,
            edge_bug_detection: false,
            max_speed_limit: 3500.0,
        }
    }
    
    /// Updates player velocity and state
    fn update_player_state(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        
        if let Some(local_controller) = local_controller.instance.value_reference(memory.view_arc()) {
            let local_pawn_handle = local_controller.m_hPlayerPawn()?;
            let entities = ctx.states.resolve::<StateEntityList>(())?;
            
            if let Ok(local_pawn_entity) = entities.entity_from_handle(&local_pawn_handle) {
                if let Some(local_pawn) = local_pawn_entity.value_reference(memory.view_arc()) {
                    // Get velocity
                    let velocity_array = local_pawn.m_vecAbsVelocity()?;
                    self.last_velocity = self.velocity;
                    self.velocity = Vector3::new(velocity_array[0], velocity_array[1], velocity_array[2]);
                    
                    // Calculate current speed (horizontal only)
                    self.current_speed = (self.velocity.x * self.velocity.x + self.velocity.y * self.velocity.y).sqrt();
                    
                    // Update best speed
                    if self.current_speed > self.best_speed {
                        self.best_speed = self.current_speed;
                    }
                    
                    // Determine ground state
                    let flags = local_pawn.m_fFlags()?;
                    let is_on_ground = (flags & 1) != 0; // FL_ONGROUND flag
                    
                    let now = Instant::now();
                    
                    // Update state machine
                    match self.state {
                        BhopState::OnGround => {
                            if !is_on_ground {
                                self.state = BhopState::InAir;
                                self.last_air_time = now;
                                self.air_time = Duration::ZERO;
                            } else {
                                self.ground_time = now.duration_since(self.last_ground_time);
                            }
                        }
                        BhopState::InAir => {
                            if is_on_ground {
                                self.state = BhopState::Landing;
                                self.last_ground_time = now;
                                self.ground_time = Duration::ZERO;
                                self.air_time = now.duration_since(self.last_air_time);
                            } else {
                                self.air_time = now.duration_since(self.last_air_time);
                            }
                        }
                        BhopState::Landing => {
                            if is_on_ground {
                                self.state = BhopState::OnGround;
                            } else {
                                self.state = BhopState::InAir;
                                self.last_air_time = now;
                            }
                        }
                        BhopState::PreJump => {
                            if !is_on_ground {
                                self.state = BhopState::InAir;
                                self.last_air_time = now;
                            } else if now.duration_since(self.last_jump_time) > Duration::from_millis(100) {
                                self.state = BhopState::OnGround;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculates optimal strafe synchronization
    fn calculate_optimal_strafe(&self, mouse_delta: f32) -> StrafeSync {
        let velocity_angle = self.velocity.y.atan2(self.velocity.x);
        let velocity_magnitude = (self.velocity.x * self.velocity.x + self.velocity.y * self.velocity.y).sqrt();
        
        // Determine optimal strafe direction based on mouse movement
        let strafe_key = if mouse_delta > 0.5 {
            StrafeKey::D
        } else if mouse_delta < -0.5 {
            StrafeKey::A
        } else {
            StrafeKey::None
        };
        
        // Calculate sync percentage (simplified)
        let ideal_mouse_movement = velocity_magnitude * 0.01; // Simplified calculation
        let sync_percentage = if mouse_delta.abs() > 0.1 {
            (ideal_mouse_movement / mouse_delta.abs()).min(1.0)
        } else {
            0.0
        };
        
        StrafeSync {
            mouse_movement: mouse_delta,
            key_press: strafe_key,
            sync_percentage,
        }
    }
    
    /// Executes perfect bunny hop timing
    fn execute_bhop(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        if !self.should_bhop() {
            return Ok(());
        }
        
        let now = Instant::now();
        
        // Apply timing variation for humanization
        let base_delay = Duration::from_millis(1); // Minimum delay for perfect bhop
        let variation = if self.humanization_enabled {
            let mut rng = thread_rng();
            let variation_ms = rng.gen_range(0..=(self.timing_variation * 10.0) as u64);
            Duration::from_millis(variation_ms)
        } else {
            Duration::ZERO
        };
        
        let total_delay = base_delay + variation;
        
        // Check if enough time has passed since last jump
        if now.duration_since(self.last_jump_time) < total_delay {
            return Ok(());
        }
        
        // Execute jump
        self.send_jump_input(ctx)?;
        self.last_jump_time = now;
        self.jumps_total += 1;
        
        // Track successful jumps (simplified - would need velocity comparison)
        if self.current_speed > self.last_velocity.norm() {
            self.jumps_hit += 1;
        }
        
        self.state = BhopState::PreJump;
        
        Ok(())
    }
    
    /// Determines if bhop should be executed
    fn should_bhop(&self) -> bool {
        // Must be landing or on ground
        if !matches!(self.state, BhopState::Landing | BhopState::OnGround) {
            return false;
        }
        
        // Must have horizontal velocity
        if self.current_speed < 100.0 {
            return false;
        }
        
        // Speed limit check
        if self.current_speed > self.max_speed_limit {
            return false;
        }
        
        // Humanization miss chance
        if self.humanization_enabled {
            let mut rng = thread_rng();
            if rng.gen::<f32>() < self.miss_chance {
                return false;
            }
        }
        
        // Ground time check (too long on ground = walking, not hopping)
        if self.ground_time > Duration::from_millis(50) {
            return false;
        }
        
        true
    }
    
    /// Executes auto-strafing for air acceleration
    fn execute_auto_strafe(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        if !self.auto_strafe || self.state != BhopState::InAir {
            return Ok(());
        }
        
        // Calculate mouse movement for optimal gain
        let mouse_sensitivity = 1.0; // Would get from game settings
        let fps = 60.0; // Assume 60 FPS for calculation
        
        // Optimal mouse movement based on current velocity
        let velocity_yaw = self.velocity.y.atan2(self.velocity.x);
        let optimal_mouse_delta = if self.perfect_sync {
            self.calculate_perfect_mouse_delta(velocity_yaw, fps)
        } else {
            self.calculate_natural_mouse_delta(velocity_yaw)
        };
        
        // Apply humanization
        let final_mouse_delta = if self.humanization_enabled {
            let mut rng = thread_rng();
            let noise = rng.gen_range(-0.1..0.1);
            optimal_mouse_delta + noise
        } else {
            optimal_mouse_delta
        };
        
        // Send mouse input
        if final_mouse_delta.abs() > 0.1 {
            self.send_mouse_input(final_mouse_delta, ctx)?;
        }
        
        // Calculate and store strafe sync
        self.current_strafe = self.calculate_optimal_strafe(final_mouse_delta);
        
        // Store in history for analysis
        self.strafe_history.push(self.current_strafe.clone());
        if self.strafe_history.len() > 100 {
            self.strafe_history.remove(0);
        }
        
        Ok(())
    }
    
    /// Calculates perfect mouse movement for maximum acceleration
    fn calculate_perfect_mouse_delta(&self, velocity_yaw: f32, fps: f32) -> f32 {
        // Perfect sync calculation based on CS2 air acceleration mechanics
        let air_accelerate = 12.0; // CS2 air acceleration value
        let max_air_speed = 30.0;  // CS2 maximum air speed
        
        // Optimal turn rate for perfect sync
        let frame_time = 1.0 / fps;
        let optimal_turn_rate = air_accelerate * frame_time;
        
        // Mouse sensitivity conversion (simplified)
        let mouse_delta = optimal_turn_rate * 0.022 * 1.0; // Assuming sens = 1
        
        // Determine direction based on current velocity
        if self.velocity.x > 0.0 {
            mouse_delta // Turn right
        } else {
            -mouse_delta // Turn left
        }
    }
    
    /// Calculates natural-looking mouse movement
    fn calculate_natural_mouse_delta(&self, velocity_yaw: f32) -> f32 {
        // More natural mouse movement with some imperfection
        let base_delta = self.calculate_perfect_mouse_delta(velocity_yaw, 60.0);
        
        let mut rng = thread_rng();
        let efficiency = rng.gen_range(0.85..0.98); // 85-98% efficiency
        
        base_delta * efficiency
    }
    
    /// Sends jump input to the game
    fn send_jump_input(&self, ctx: &UpdateContext) -> anyhow::Result<()> {
        // Send space key press (simplified - would use proper key input)
        ctx.cs2.send_mouse_state(&[MouseState {
            // Would implement proper key input here
            ..Default::default()
        }])?;
        
        Ok(())
    }
    
    /// Sends mouse input for strafing
    fn send_mouse_input(&self, delta: f32, ctx: &UpdateContext) -> anyhow::Result<()> {
        ctx.cs2.send_mouse_state(&[MouseState {
            last_x: (delta * 100.0) as i32, // Scale for mouse input
            last_y: 0,
            ..Default::default()
        }])?;
        
        Ok(())
    }
    
    /// Detects edge bugs and walls for advanced movement
    fn detect_edge_bugs(&self) -> bool {
        if !self.edge_bug_detection {
            return false;
        }
        
        // Simplified edge bug detection
        // In a real implementation, this would check for specific map geometry
        // and velocity patterns that indicate edge bug opportunities
        
        // Check for sudden velocity changes that might indicate wall contact
        let velocity_change = (self.velocity - self.last_velocity).norm();
        velocity_change > 500.0 && self.state == BhopState::InAir
    }
    
    /// Gets performance statistics
    pub fn get_statistics(&self) -> BhopStatistics {
        let hit_rate = if self.jumps_total > 0 {
            self.jumps_hit as f32 / self.jumps_total as f32
        } else {
            0.0
        };
        
        let average_sync = if !self.strafe_history.is_empty() {
            self.strafe_history.iter()
                .map(|s| s.sync_percentage)
                .sum::<f32>() / self.strafe_history.len() as f32
        } else {
            0.0
        };
        
        BhopStatistics {
            jumps_hit: self.jumps_hit,
            jumps_total: self.jumps_total,
            hit_rate,
            current_speed: self.current_speed,
            best_speed: self.best_speed,
            average_sync,
            state: self.state.clone(),
        }
    }
    
    /// Resets statistics
    pub fn reset_statistics(&mut self) {
        self.jumps_hit = 0;
        self.jumps_total = 0;
        self.best_speed = 0.0;
        self.strafe_history.clear();
    }
}

impl Enhancement for AdvancedBhop {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        
        // Update toggle (would need bhop settings in AppSettings)
        // For now, assume it's always enabled for demonstration
        self.toggle.enabled = true;
        
        if !self.toggle.enabled {
            return Ok(());
        }
        
        // Update player state and velocity
        self.update_player_state(ctx)?;
        
        // Execute bunny hop if conditions are met
        self.execute_bhop(ctx)?;
        
        // Execute auto-strafe for air movement
        self.execute_auto_strafe(ctx)?;
        
        // Check for edge bugs
        if self.detect_edge_bugs() {
            // Would implement edge bug exploitation here
        }
        
        Ok(())
    }
    
    fn render(
        &self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        // Could render speed/sync info on screen
        Ok(())
    }
}

/// Statistics structure for bhop performance
#[derive(Debug, Clone)]
pub struct BhopStatistics {
    pub jumps_hit: u32,
    pub jumps_total: u32,
    pub hit_rate: f32,
    pub current_speed: f32,
    pub best_speed: f32,
    pub average_sync: f32,
    pub state: BhopState,
}

/// Configuration for bunny hop behavior
#[derive(Debug, Clone)]
pub struct BhopConfig {
    pub enabled: bool,
    pub perfect_sync: bool,
    pub auto_strafe: bool,
    pub edge_bug_detection: bool,
    pub humanization_enabled: bool,
    pub miss_chance: f32,
    pub timing_variation: f32,
    pub max_speed_limit: f32,
    pub auto_turn: bool,
    pub scroll_bhop: bool,
}

impl Default for BhopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            perfect_sync: false,
            auto_strafe: true,
            edge_bug_detection: false,
            humanization_enabled: true,
            miss_chance: 0.02,
            timing_variation: 0.05,
            max_speed_limit: 3500.0,
            auto_turn: false,
            scroll_bhop: false,
        }
    }
}