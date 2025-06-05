use anyhow::Context;
use cs2::{
    BoneMap,
    BodyPart,
    CEntityIdentityEx,
    ClassNameCache,
    CS2Model,
    MouseState,
    PlayerPawnState,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
    StatePawnInfo,
    StatePawnModelInfo,
};
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_CSPlayerPawnBase,
};
use nalgebra::Vector3;
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{KeyToggle, ViewController},
    UpdateContext,
};

#[derive(Clone)]
pub struct AimbotTarget {
    pub entity_id: u32,
    pub position: Vector3<f32>,
    pub distance: f32,
    pub angle_diff: f32,
    pub bone_position: Vector3<f32>,
    pub bone_name: String, // Add bone name for debugging
}

pub struct Aimbot {
    toggle: KeyToggle,
    current_target: Option<AimbotTarget>,
    last_shot_time: std::time::Instant,
}

impl Aimbot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            current_target: None,
            last_shot_time: std::time::Instant::now(),
        }
    }

    fn get_best_target(&self, ctx: &UpdateContext) -> anyhow::Result<Option<AimbotTarget>> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let view = ctx.states.resolve::<ViewController>(())?;
        
        let Some(camera_position) = view.get_camera_world_position() else {
            return Ok(None);
        };

        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
        let Some(local_controller) = local_controller.instance.value_reference(memory.view_arc()) else {
            return Ok(None);
        };
        let local_team = local_controller.m_iPendingTeamNum()?;

                let mut targets = Vec::new();        // Iterate through all players with more lenient checks
        for entity_identity in entities.entities() {
            // More lenient entity class checking
            let is_player = if let Ok(class_name_cache) = ctx.states.resolve::<ClassNameCache>(()) {
                if let Ok(entity_class_info) = entity_identity.entity_class_info() {
                    if let Ok(entity_class) = class_name_cache.lookup(&entity_class_info) {
                        entity_class.map(|name| *name == "C_CSPlayerPawn").unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
            
            if !is_player {
                continue;
            }

            // Check if player is alive (but be more lenient)
            if let Ok(pawn_state) = ctx.states.resolve::<PlayerPawnState>(entity_identity.handle()?) {
                if *pawn_state != PlayerPawnState::Alive {
                    continue;
                }
            } else {
                // If we can't determine state, try to continue anyway
            }

            // Get player info with fallback
            let pawn_info = match ctx.states.resolve::<StatePawnInfo>(entity_identity.handle()?) {
                Ok(info) => info,
                Err(_) => continue, // Skip if we can't get basic info
            };
            
            // More lenient health check
            if pawn_info.player_health <= 0 {
                continue;
            }// Team check with error handling
            if settings.aimbot_team_check {
                if let Ok(player_pawn) = entity_identity
                    .entity_ptr::<dyn C_CSPlayerPawnBase>()
                    .and_then(|ptr| ptr.value_reference(memory.view_arc()).context("player pawn nullptr"))
                {
                    if let Ok(team_num) = player_pawn.m_iTeamNum() {
                        if team_num == local_team {
                            continue;
                        }
                    } else {
                        continue; // Skip if we can't read team
                    }
                } else {
                    continue; // Skip if we can't access player pawn
                }
            }            let pawn_model = ctx.states.resolve::<StatePawnModelInfo>(entity_identity.handle()?)?;
            let entry_model = ctx.states.resolve::<CS2Model>(pawn_model.model_address)?;            // Calculate target bone position with proper bone names and heights            // Create bone map for this model
            let bone_map = BoneMap::new();
            let bone_names: Vec<String> = entry_model.bones.iter().map(|bone| bone.name.clone()).collect();
              let bone_position = match settings.aimbot_target_bone {
                0 => {
                    // Head bone
                    if let Some(head_bone_index) = bone_map.find_bone_index(BodyPart::Head, &bone_names) {
                        if let Some(head_state) = pawn_model.bone_states.get(head_bone_index) {
                            (*head_state).position
                        } else {
                            // Fallback to approximate head position
                            pawn_info.position + Vector3::new(0.0, 0.0, 72.0)                        }
                    } else {
                        // Try neck as fallback
                        if let Some(neck_bone_index) = bone_map.find_bone_index(BodyPart::Neck, &bone_names) {
                            if let Some(neck_state) = pawn_model.bone_states.get(neck_bone_index) {
                                (*neck_state).position + Vector3::new(0.0, 0.0, 8.0) // Slightly above neck
                            } else {
                                pawn_info.position + Vector3::new(0.0, 0.0, 72.0)
                            }
                        } else {
                            pawn_info.position + Vector3::new(0.0, 0.0, 72.0)
                        }
                    }
                }
                1 => {
                    // Chest/Upper body - try upper or middle spine
                    if let Some(upper_spine_index) = bone_map.find_bone_index(BodyPart::UpperSpine, &bone_names) {                        if let Some(spine_state) = pawn_model.bone_states.get(upper_spine_index) {
                            (*spine_state).position
                        } else {
                            // Try middle spine as fallback
                            if let Some(middle_spine_index) = bone_map.find_bone_index(BodyPart::MiddleSpine, &bone_names) {                                if let Some(spine_state) = pawn_model.bone_states.get(middle_spine_index) {
                                    (*spine_state).position
                                } else {
                                    pawn_info.position + Vector3::new(0.0, 0.0, 48.0)
                                }
                            } else {
                                pawn_info.position + Vector3::new(0.0, 0.0, 48.0)
                            }
                        }
                    } else {
                        pawn_info.position + Vector3::new(0.0, 0.0, 48.0)
                    }
                }
                _ => {
                    // Body center - pelvis or lower spine
                    if let Some(pelvis_index) = bone_map.find_bone_index(BodyPart::Pelvis, &bone_names) {                        if let Some(pelvis_state) = pawn_model.bone_states.get(pelvis_index) {
                            (*pelvis_state).position
                        } else {
                            // Try lower spine as fallback
                            if let Some(lower_spine_index) = bone_map.find_bone_index(BodyPart::LowerSpine, &bone_names) {
                                if let Some(spine_state) = pawn_model.bone_states.get(lower_spine_index) {
                                    (*spine_state).position
                                } else {
                                    pawn_info.position + Vector3::new(0.0, 0.0, 36.0)
                                }
                            } else {
                                pawn_info.position + Vector3::new(0.0, 0.0, 36.0)
                            }
                        }
                    } else {
                        pawn_info.position + Vector3::new(0.0, 0.0, 36.0)
                    }
                }
            };
            
            // Get the targeted bone name for debugging
            let targeted_bone_name = match settings.aimbot_target_bone {
                0 => bone_map.get_primary_bone_name(BodyPart::Head).unwrap_or_else(|| "head".to_string()),
                1 => bone_map.get_primary_bone_name(BodyPart::UpperSpine).unwrap_or_else(|| "chest".to_string()),
                _ => bone_map.get_primary_bone_name(BodyPart::Pelvis).unwrap_or_else(|| "pelvis".to_string()),
            };
            
            let distance = (bone_position - camera_position).norm();
            
            // Remove distance check - allow any distance
            // if distance > settings.aimbot_max_distance {
            //     continue;
            // }
            
            // Simple FOV check using screen position
            if let Some(screen_pos) = view.world_to_screen(&bone_position, false) {
                let screen_center_x = view.screen_bounds.x / 2.0;
                let screen_center_y = view.screen_bounds.y / 2.0;
                let screen_distance = ((screen_pos.x - screen_center_x).powf(2.0) + (screen_pos.y - screen_center_y).powf(2.0)).sqrt();
                
                // Fix FOV calculation - use proper screen space radius
                let fov_radius = (settings.aimbot_fov / 90.0) * (view.screen_bounds.x.min(view.screen_bounds.y) / 4.0);
                
                // Safety check for division by zero
                if fov_radius <= 0.0 {
                    continue;
                }
                
                // More lenient FOV check
                if screen_distance > fov_radius && settings.aimbot_fov < 180.0 {
                    continue;
                }
                
                // Use screen distance as angle difference approximation
                let angle_diff = if fov_radius > 0.0 {
                    (screen_distance / fov_radius).min(1.0) * (settings.aimbot_fov / 2.0)
                } else {
                    0.0
                };
                  targets.push(AimbotTarget {
                    entity_id: entity_identity.handle::<()>()?.get_entity_index(),
                    position: pawn_info.position,
                    distance,
                    angle_diff,
                    bone_position,
                    bone_name: targeted_bone_name.clone(), // Include the bone name for debugging
                });
            }
        }

        // Sort by angle difference (closest to crosshair)
        targets.sort_by(|a, b| a.angle_diff.partial_cmp(&b.angle_diff).unwrap_or(std::cmp::Ordering::Equal));

        Ok(targets.into_iter().next())
    }    // Note: These methods were removed as they're not currently used
    // The aimbot now uses a simpler screen-space approach for better performance
}

impl Enhancement for Aimbot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        
        // Update toggle state
        if self.toggle.update(
            &settings.aimbot_mode,
            ctx.input,
            &settings.key_aimbot,
        ) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, settings.aimbot_mode
                ),
            );
        }

        if !self.toggle.enabled {
            self.current_target = None;
            return Ok(());
        }

        // Find best target        self.current_target = self.get_best_target(ctx)?;

        let Some(target) = &self.current_target else {
            return Ok(());
        };

        let view = ctx.states.resolve::<ViewController>(())?;
        
        // Calculate screen position of target
        let Some(target_screen) = view.world_to_screen(&target.bone_position, true) else {
            return Ok(());
        };        // Calculate distance from screen center
        let screen_center_x = view.screen_bounds.x / 2.0;
        let screen_center_y = view.screen_bounds.y / 2.0;
        
        // Le problème semble être dans la gestion du mouvement de souris
        let delta_x = target_screen.x - screen_center_x;
        let delta_y = target_screen.y - screen_center_y;
        
        // Loggons les valeurs pour debug
        log::info!("Target position: ({}, {}), screen center: ({}, {}), delta: ({}, {})",
            target_screen.x, target_screen.y, screen_center_x, screen_center_y, delta_x, delta_y);
        
        // IMPORTANT: L'axe Y est inversé dans les jeux FPS - si la cible est en bas de l'écran,
        // delta_y est positif, mais on doit déplacer la souris vers le bas (valeur négative)
        let delta_y = -delta_y; // Inverser l'axe Y pour correspondre à la logique des FPS
          
        // Apply smoothing - réduire pour un mouvement plus rapide
        let smoothing_factor = settings.aimbot_smoothing.max(1.0); // Assurer un minimum de 1.0
        let smooth_delta_x = delta_x / smoothing_factor;
        let smooth_delta_y = delta_y / smoothing_factor;
          // CS2 uses a specific sensitivity scale that varies with screen resolution
        let cs2_sens = 1.0; // On suppose une sensibilité moyenne dans CS2
        let _screen_width = view.screen_bounds.x;
        let _screen_height = view.screen_bounds.y;
          
        // Calculer un facteur d'échelle basé sur la résolution et le FOV
        let base_sens = 0.15; // Valeur de base pour la sensibilité
        
        // Appliquer un facteur d'amplification pour s'assurer que le mouvement est suffisant
        // L'amplification est plus importante pour les petits mouvements
        let amplification = if delta_x.abs() < 5.0 && delta_y.abs() < 5.0 {
            5.0 // Amplification forte pour les petits mouvements
        } else if delta_x.abs() < 20.0 && delta_y.abs() < 20.0 {
            3.0 // Amplification moyenne pour les mouvements modérés
        } else {
            2.0 // Amplification standard pour les grands mouvements
        };
        
        // Calculer les déplacements finaux de la souris
        let mouse_x = (smooth_delta_x * base_sens * cs2_sens * amplification) as i32;
        let mouse_y = (smooth_delta_y * base_sens * cs2_sens * amplification) as i32;
        
        // Limites dynamiques - plus grandes pour les mouvements importants
        let max_move = if delta_x.abs() > 100.0 || delta_y.abs() > 100.0 {
            200 // Pour les grandes distances
        } else if delta_x.abs() > 50.0 || delta_y.abs() > 50.0 {
            150 // Pour les distances moyennes
        } else {
            100 // Pour les petites distances
        };
        
        // Log pour debug
        log::info!("Mouse move (before clamp): x={}, y={}, amplification={}, max_move={}",
            mouse_x, mouse_y, amplification, max_move);
        
        // Send mouse movement with improved precision
        if mouse_x != 0 || mouse_y != 0 {
            // Appliquer les limites
            let safe_mouse_x = mouse_x.clamp(-max_move, max_move);
            let safe_mouse_y = mouse_y.clamp(-max_move, max_move);
              // Toujours envoyer, même pour de petits mouvements
            log::info!("Sending mouse move: x={}, y={}", safe_mouse_x, safe_mouse_y);
            
            // Utiliser les mouvements relatifs dans MouseState
            ctx.cs2.send_mouse_state(&[MouseState {
                last_x: safe_mouse_x,
                last_y: safe_mouse_y,
                ..Default::default()
            }])?;
            
            // Courte pause pour laisser CS2 traiter le mouvement (évite les mouvements trop rapides)
            std::thread::sleep(std::time::Duration::from_millis(1));
        }// Auto shoot if enabled and target is close to crosshair
        // Augmenter le seuil d'angle_diff pour l'auto-shoot
        if settings.aimbot_auto_shoot && target.angle_diff < 5.0 {
            // Calculer la distance au centre (pixels)
            let center_distance = ((delta_x * delta_x) + (delta_y * delta_y)).sqrt();
            
            // Log pour debug
            log::info!("Auto-shoot check: angle_diff={}, center_distance={}", 
                target.angle_diff, center_distance);
            
            // Tirer seulement si la cible est suffisamment proche du centre de l'écran
            if center_distance < 20.0 {
                let now = std::time::Instant::now();
                if now.duration_since(self.last_shot_time).as_millis() > 100 {
                    // Send click down
                    log::info!("Auto-shooting!");
                    ctx.cs2.send_mouse_state(&[MouseState {
                        buttons: [Some(true), None, None, None, None],
                        ..Default::default()
                    }])?;
                    
                    self.last_shot_time = now;
                }
            }
        }

        // Auto release for auto-shoot (separate from the trigger to avoid blocking)
        if settings.aimbot_auto_shoot {
            let now = std::time::Instant::now();
            if now.duration_since(self.last_shot_time).as_millis() > 50 && now.duration_since(self.last_shot_time).as_millis() < 150 {
                // Release click
                ctx.cs2.send_mouse_state(&[MouseState {
                    buttons: [Some(false), None, None, None, None],
                    ..Default::default()
                }])?;
            }
        }

        Ok(())
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        if !self.toggle.enabled {
            return Ok(());
        }

        let Some(target) = &self.current_target else {
            return Ok(());
        };

        let view = states.resolve::<ViewController>(())?;
        let Some(target_screen) = view.world_to_screen(&target.bone_position, true) else {
            return Ok(());
        };

        let draw = ui.get_window_draw_list();
        
        // Draw target indicator
        draw.add_circle(target_screen, 20.0, [1.0, 0.0, 0.0, 0.8])
            .filled(false)
            .thickness(2.0)
            .build();

        // Draw crosshair lines
        let screen_center = [ui.window_size()[0] / 2.0, ui.window_size()[1] / 2.0];
        draw.add_line(
            screen_center,
            target_screen,
            [1.0, 0.0, 0.0, 0.5]
        )
        .thickness(1.0)
        .build();

        // Draw FOV circle
        let settings = states.resolve::<AppSettings>(())?;
        let fov_radius = (settings.aimbot_fov / 2.0) * 10.0; // Simplified FOV visualization
        draw.add_circle(screen_center, fov_radius, [0.0, 1.0, 0.0, 0.3])
            .filled(false)
            .thickness(1.0)
            .build();

        Ok(())
    }

        fn update_settings(
        &mut self,
        ui: &imgui::Ui,
        settings: &mut AppSettings,
    ) -> anyhow::Result<bool> {
        let mut changed = false;

        if ui.collapsing_header("Aimbot", imgui::TreeNodeFlags::empty()) {            ui.text("Mode:");
            let mut mode_index_usize = match settings.aimbot_mode {
                crate::settings::KeyToggleMode::AlwaysOn => 0,
                crate::settings::KeyToggleMode::Toggle => 1,
                crate::settings::KeyToggleMode::Trigger => 2,
                crate::settings::KeyToggleMode::TriggerInverted => 3,
                crate::settings::KeyToggleMode::Off => 4,
            } as usize;
            
            if ui.combo_simple_string(
                "##aimbot_mode",
                &mut mode_index_usize,
                &["Always On", "Toggle", "Trigger", "Trigger Inverted", "Off"],
                        ) {
                settings.aimbot_mode = match mode_index_usize {
                    0 => crate::settings::KeyToggleMode::AlwaysOn,
                    1 => crate::settings::KeyToggleMode::Toggle,
                    2 => crate::settings::KeyToggleMode::Trigger,
                    3 => crate::settings::KeyToggleMode::TriggerInverted,
                    _ => crate::settings::KeyToggleMode::Off,
                };
                changed = true;
            }

            ui.checkbox("Team Check", &mut settings.aimbot_team_check);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("FOV", 1.0, 180.0, &mut settings.aimbot_fov);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("Smoothing", 0.1, 20.0, &mut settings.aimbot_smoothing);
            if ui.is_item_edited() {
                changed = true;
            }

                        let mut bone_index = settings.aimbot_target_bone as usize;
            if ui.combo_simple_string(
                "Target Bone",
                &mut bone_index,
                &["Head", "Chest", "Closest"],
            ) {
                settings.aimbot_target_bone = bone_index as u32;
                changed = true;
            }

            ui.checkbox("Auto Shoot", &mut settings.aimbot_auto_shoot);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("Max Distance", 50.0, 1000.0, &mut settings.aimbot_max_distance);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.checkbox("Visible Only", &mut settings.aimbot_visible_only);
            if ui.is_item_edited() {
                changed = true;
            }
        }

        Ok(changed)
    }
}
