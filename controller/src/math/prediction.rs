use nalgebra::Vector3;
use std::time::{Duration, Instant};

/// Calculates the predicted position of a target based on velocity and time
pub fn predict_position(
    current_position: &Vector3<f32>,
    velocity: &Vector3<f32>,
    prediction_time: f32,
) -> Vector3<f32> {
    current_position + velocity * prediction_time
}

/// Advanced prediction considering acceleration and gravity
pub fn predict_position_with_physics(
    current_position: &Vector3<f32>,
    velocity: &Vector3<f32>,
    acceleration: &Vector3<f32>,
    prediction_time: f32,
    apply_gravity: bool,
) -> Vector3<f32> {
    let gravity = if apply_gravity {
        Vector3::new(0.0, 0.0, -9.81 * 52.5) // CS2 gravity units
    } else {
        Vector3::zeros()
    };

    let total_acceleration = acceleration + gravity;
    
    // s = ut + 0.5at²
    current_position 
        + velocity * prediction_time 
        + 0.5 * total_acceleration * prediction_time * prediction_time
}

/// Calculates the time for a projectile to reach a target
pub fn calculate_intercept_time(
    shooter_pos: &Vector3<f32>,
    target_pos: &Vector3<f32>,
    target_velocity: &Vector3<f32>,
    projectile_speed: f32,
) -> Option<f32> {
    let relative_pos = target_pos - shooter_pos;
    
    // Quadratic equation coefficients for projectile interception
    let a = target_velocity.dot(target_velocity) - projectile_speed * projectile_speed;
    let b = 2.0 * relative_pos.dot(target_velocity);
    let c = relative_pos.dot(&relative_pos);
    
    let discriminant = b * b - 4.0 * a * c;
    
    if discriminant < 0.0 {
        return None; // No solution
    }
    
    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b + sqrt_discriminant) / (2.0 * a);
    let t2 = (-b - sqrt_discriminant) / (2.0 * a);
    
    // Return the smallest positive time
    let times = [t1, t2];
    times.iter()
        .filter(|&&t| t > 0.0)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
}

/// Velocity history tracker for better prediction accuracy
#[derive(Debug, Clone)]
pub struct VelocityTracker {
    positions: Vec<(Vector3<f32>, Instant)>,
    max_samples: usize,
}

impl VelocityTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            positions: Vec::with_capacity(max_samples),
            max_samples,
        }
    }
    
    pub fn add_position(&mut self, position: Vector3<f32>) {
        let now = Instant::now();
        self.positions.push((position, now));
        
        if self.positions.len() > self.max_samples {
            self.positions.remove(0);
        }
        
        // Remove old samples (older than 1 second)
        let cutoff = now - Duration::from_secs(1);
        self.positions.retain(|(_, time)| *time > cutoff);
    }
    
    pub fn get_velocity(&self) -> Option<Vector3<f32>> {
        if self.positions.len() < 2 {
            return None;
        }
        
        let (newest_pos, newest_time) = self.positions.last()?;
        let (oldest_pos, oldest_time) = self.positions.first()?;
        
        let delta_time = newest_time.duration_since(*oldest_time).as_secs_f32();
        if delta_time <= 0.0 {
            return None;
        }
        
        let delta_pos = newest_pos - oldest_pos;
        Some(delta_pos / delta_time)
    }
    
    pub fn get_acceleration(&self) -> Option<Vector3<f32>> {
        if self.positions.len() < 3 {
            return None;
        }
        
        let len = self.positions.len();
        let mid_idx = len / 2;
        
        let (end_pos, end_time) = &self.positions[len - 1];
        let (mid_pos, mid_time) = &self.positions[mid_idx];
        let (start_pos, start_time) = &self.positions[0];
        
        let dt1 = mid_time.duration_since(*start_time).as_secs_f32();
        let dt2 = end_time.duration_since(*mid_time).as_secs_f32();
        
        if dt1 <= 0.0 || dt2 <= 0.0 {
            return None;
        }
        
        let v1 = (mid_pos - start_pos) / dt1;
        let v2 = (end_pos - mid_pos) / dt2;
        
        Some((v2 - v1) / ((dt1 + dt2) / 2.0))
    }
    
    pub fn predict_future_position(&self, time_ahead: f32) -> Option<Vector3<f32>> {
        if let Some((current_pos, _)) = self.positions.last() {
            if let Some(velocity) = self.get_velocity() {
                let acceleration = self.get_acceleration().unwrap_or_else(Vector3::zeros);
                return Some(predict_position_with_physics(
                    current_pos,
                    &velocity,
                    &acceleration,
                    time_ahead,
                    false, // Usually don't apply gravity for player movement
                ));
            }
        }
        None
    }
}

/// Calculates optimal prediction time based on distance and weapon
pub fn calculate_optimal_prediction_time(
    distance: f32,
    weapon_type: WeaponType,
) -> f32 {
    let base_speed = match weapon_type {
        WeaponType::Rifle => 2700.0,      // Average rifle bullet speed
        WeaponType::Pistol => 1800.0,     // Average pistol bullet speed  
        WeaponType::Sniper => 3500.0,     // Sniper rifle bullet speed
        WeaponType::SMG => 2200.0,        // SMG bullet speed
        WeaponType::Shotgun => 1200.0,    // Shotgun pellet speed
        WeaponType::Unknown => 2500.0,    // Default speed
    };
    
    // Add some buffer time for human reaction and game lag
    let travel_time = distance / base_speed;
    let buffer_time = 0.016; // One frame at 60fps minimum
    
    travel_time + buffer_time
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    Rifle,
    Pistol,
    Sniper,
    SMG,
    Shotgun,
    Unknown,
}

impl WeaponType {
    pub fn from_weapon_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("ak47") || name_lower.contains("m4a4") || name_lower.contains("m4a1") {
            WeaponType::Rifle
        } else if name_lower.contains("awp") || name_lower.contains("ssg08") {
            WeaponType::Sniper
        } else if name_lower.contains("glock") || name_lower.contains("usp") || name_lower.contains("p250") {
            WeaponType::Pistol
        } else if name_lower.contains("mp5") || name_lower.contains("p90") || name_lower.contains("ump") {
            WeaponType::SMG
        } else if name_lower.contains("nova") || name_lower.contains("xm1014") {
            WeaponType::Shotgun
        } else {
            WeaponType::Unknown
        }
    }
}