use nalgebra::{Vector2, Vector3};
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

/// Bezier curve implementation for smooth aimbot movement
#[derive(Debug, Clone)]
pub struct BezierCurve {
    pub control_points: Vec<Vector2<f32>>,
    pub duration: Duration,
    pub start_time: Instant,
}

impl BezierCurve {
    /// Creates a cubic Bezier curve for smooth mouse movement
    pub fn new_cubic(
        start: Vector2<f32>,
        end: Vector2<f32>,
        duration: Duration,
        smoothness: f32,
    ) -> Self {
        let direction = end - start;
        let distance = direction.norm();
        
        // Control points based on smoothness setting
        let control_factor = smoothness.clamp(0.1, 1.0) * 0.4;
        let perpendicular = Vector2::new(-direction.y, direction.x).normalize();
        
        // Add some random variation for more natural movement
        let mut rng = thread_rng();
        let random_offset = rng.gen_range(-0.2..0.2) * distance * control_factor;
        let random_perpendicular = perpendicular * random_offset;
        
        let control1 = start + direction * control_factor + random_perpendicular;
        let control2 = end - direction * control_factor - random_perpendicular;
        
        Self {
            control_points: vec![start, control1, control2, end],
            duration,
            start_time: Instant::now(),
        }
    }
    
    /// Creates a more natural curve with multiple control points
    pub fn new_natural(
        start: Vector2<f32>,
        end: Vector2<f32>,
        duration: Duration,
        natural_variation: f32,
    ) -> Self {
        let mut rng = thread_rng();
        let direction = end - start;
        let distance = direction.norm();
        
        if distance < 10.0 {
            // For small movements, use simple linear interpolation
            return Self {
                control_points: vec![start, end],
                duration,
                start_time: Instant::now(),
            };
        }
        
        let mut points = vec![start];
        
        // Add intermediate control points for more natural movement
        let num_controls = ((distance / 100.0) as usize).min(5).max(2);
        
        for i in 1..num_controls {
            let t = i as f32 / num_controls as f32;
            let base_point = start + direction * t;
            
            // Add natural variation
            let perpendicular = Vector2::new(-direction.y, direction.x).normalize();
            let variation_amount = natural_variation * distance * 0.1;
            let random_offset = rng.gen_range(-variation_amount..variation_amount);
            
            let control_point = base_point + perpendicular * random_offset;
            points.push(control_point);
        }
        
        points.push(end);
        
        Self {
            control_points: points,
            duration,
            start_time: Instant::now(),
        }
    }
    
    /// Gets the current position on the curve based on time elapsed
    pub fn get_current_position(&self) -> Option<Vector2<f32>> {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            return Some(*self.control_points.last()?);
        }
        
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        Some(self.evaluate_at(t))
    }
    
    /// Evaluates the curve at parameter t (0.0 to 1.0)
    pub fn evaluate_at(&self, t: f32) -> Vector2<f32> {
        let t = t.clamp(0.0, 1.0);
        
        match self.control_points.len() {
            2 => {
                // Linear interpolation
                self.control_points[0] * (1.0 - t) + self.control_points[1] * t
            }
            3 => {
                // Quadratic Bezier
                let p0 = self.control_points[0];
                let p1 = self.control_points[1];
                let p2 = self.control_points[2];
                
                p0 * (1.0 - t).powi(2) + p1 * 2.0 * (1.0 - t) * t + p2 * t.powi(2)
            }
            4 => {
                // Cubic Bezier
                let p0 = self.control_points[0];
                let p1 = self.control_points[1];
                let p2 = self.control_points[2];
                let p3 = self.control_points[3];
                
                p0 * (1.0 - t).powi(3) 
                    + p1 * 3.0 * (1.0 - t).powi(2) * t 
                    + p2 * 3.0 * (1.0 - t) * t.powi(2) 
                    + p3 * t.powi(3)
            }
            _ => {
                // Multi-point Bezier using De Casteljau's algorithm
                self.de_casteljau(t)
            }
        }
    }
    
    /// De Casteljau's algorithm for multi-point Bezier curves
    fn de_casteljau(&self, t: f32) -> Vector2<f32> {
        let mut points = self.control_points.clone();
        
        while points.len() > 1 {
            for i in 0..points.len() - 1 {
                points[i] = points[i] * (1.0 - t) + points[i + 1] * t;
            }
            points.pop();
        }
        
        points[0]
    }
    
    /// Checks if the curve animation is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
    
    /// Gets the remaining time for the curve
    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.start_time.elapsed())
    }
}

/// Smooth movement controller with different smoothing algorithms
#[derive(Debug, Clone)]
pub struct SmoothController {
    pub smoothing_type: SmoothingType,
    pub current_curve: Option<BezierCurve>,
    pub target_position: Option<Vector2<f32>>,
    pub current_position: Vector2<f32>,
    pub smoothness_factor: f32,
    pub max_speed: f32,
    pub humanization_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmoothingType {
    Linear,
    Exponential,
    Bezier,
    Natural,
    Humanized,
}

impl SmoothController {
    pub fn new(smoothing_type: SmoothingType, smoothness_factor: f32) -> Self {
        Self {
            smoothing_type,
            current_curve: None,
            target_position: None,
            current_position: Vector2::zeros(),
            smoothness_factor: smoothness_factor.clamp(0.1, 10.0),
            max_speed: 1000.0, // pixels per second
            humanization_enabled: true,
        }
    }
    
    /// Starts smooth movement to a target position
    pub fn move_to(&mut self, start: Vector2<f32>, target: Vector2<f32>) {
        self.current_position = start;
        self.target_position = Some(target);
        
        let distance = (target - start).norm();
        if distance < 1.0 {
            return; // Too close, no need to smooth
        }
        
        // Calculate movement duration based on distance and smoothness
        let base_duration = distance / self.max_speed;
        let smooth_duration = base_duration * self.smoothness_factor;
        let duration = Duration::from_secs_f32(smooth_duration.clamp(0.01, 2.0));
        
        match self.smoothing_type {
            SmoothingType::Bezier => {
                self.current_curve = Some(BezierCurve::new_cubic(
                    start,
                    target,
                    duration,
                    self.smoothness_factor,
                ));
            }
            SmoothingType::Natural => {
                let variation = if self.humanization_enabled { 0.5 } else { 0.1 };
                self.current_curve = Some(BezierCurve::new_natural(
                    start,
                    target,
                    duration,
                    variation,
                ));
            }
            SmoothingType::Humanized => {
                let variation = 0.8; // Higher variation for human-like movement
                self.current_curve = Some(BezierCurve::new_natural(
                    start,
                    target,
                    duration,
                    variation,
                ));
            }
            _ => {
                // For Linear and Exponential, we'll handle in update
                self.current_curve = None;
            }
        }
    }
    
    /// Updates the smooth movement and returns the next position
    pub fn update(&mut self) -> Option<Vector2<f32>> {
        match self.smoothing_type {
            SmoothingType::Bezier | SmoothingType::Natural | SmoothingType::Humanized => {
                if let Some(ref curve) = self.current_curve {
                    if let Some(position) = curve.get_current_position() {
                        self.current_position = position;
                        
                        if curve.is_complete() {
                            self.current_curve = None;
                            self.target_position = None;
                        }
                        
                        return Some(position);
                    }
                }
            }
            SmoothingType::Linear => {
                if let Some(target) = self.target_position {
                    let direction = target - self.current_position;
                    let distance = direction.norm();
                    
                    if distance < 1.0 {
                        self.current_position = target;
                        self.target_position = None;
                        return Some(target);
                    }
                    
                    let speed = self.max_speed / self.smoothness_factor;
                    let movement = direction.normalize() * speed * 0.016; // Assume 60 FPS
                    self.current_position += movement;
                    
                    return Some(self.current_position);
                }
            }
            SmoothingType::Exponential => {
                if let Some(target) = self.target_position {
                    let diff = target - self.current_position;
                    if diff.norm() < 1.0 {
                        self.current_position = target;
                        self.target_position = None;
                        return Some(target);
                    }
                    
                    // Exponential smoothing
                    let alpha = 1.0 / self.smoothness_factor;
                    self.current_position += diff * alpha.clamp(0.01, 1.0);
                    
                    return Some(self.current_position);
                }
            }
        }
        
        None
    }
    
    /// Checks if smooth movement is active
    pub fn is_active(&self) -> bool {
        self.current_curve.is_some() || self.target_position.is_some()
    }
    
    /// Stops the current smooth movement
    pub fn stop(&mut self) {
        self.current_curve = None;
        self.target_position = None;
    }
    
    /// Gets the estimated time to reach the target
    pub fn estimated_completion_time(&self) -> Option<Duration> {
        if let Some(ref curve) = self.current_curve {
            return Some(curve.remaining_time());
        }
        
        if let Some(target) = self.target_position {
            let distance = (target - self.current_position).norm();
            let speed = self.max_speed / self.smoothness_factor;
            if speed > 0.0 {
                return Some(Duration::from_secs_f32(distance / speed));
            }
        }
        
        None
    }
}

/// Humanization features for natural mouse movement
pub struct HumanizationSettings {
    pub micro_movements: bool,
    pub reaction_time_variation: f32,
    pub overshoot_chance: f32,
    pub correction_delay: Duration,
    pub fatigue_simulation: bool,
}

impl Default for HumanizationSettings {
    fn default() -> Self {
        Self {
            micro_movements: true,
            reaction_time_variation: 0.1, // ±10% variation
            overshoot_chance: 0.15,       // 15% chance to overshoot
            correction_delay: Duration::from_millis(50),
            fatigue_simulation: false,
        }
    }
}

impl HumanizationSettings {
    /// Applies humanization to a target position
    pub fn humanize_target(&self, target: Vector2<f32>, distance: f32) -> Vector2<f32> {
        let mut rng = thread_rng();
        let mut result = target;
        
        // Add micro-movements for close targets
        if self.micro_movements && distance < 100.0 {
            let micro_offset = Vector2::new(
                rng.gen_range(-0.5..0.5),
                rng.gen_range(-0.5..0.5),
            );
            result += micro_offset;
        }
        
        // Simulate overshoot
        if rng.gen::<f32>() < self.overshoot_chance {
            let overshoot_factor = rng.gen_range(1.02..1.08);
            let center = Vector2::new(960.0, 540.0); // Assume 1920x1080 screen
            let direction = (target - center).normalize();
            result = center + direction * distance * overshoot_factor;
        }
        
        result
    }
    
    /// Calculates humanized reaction time
    pub fn get_reaction_time(&self, base_time: Duration) -> Duration {
        if self.reaction_time_variation <= 0.0 {
            return base_time;
        }
        
        let mut rng = thread_rng();
        let variation = rng.gen_range(
            -self.reaction_time_variation..self.reaction_time_variation
        );
        
        let multiplier = 1.0 + variation;
        Duration::from_secs_f32(base_time.as_secs_f32() * multiplier.max(0.1))
    }
}