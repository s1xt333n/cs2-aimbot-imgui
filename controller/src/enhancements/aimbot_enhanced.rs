use std::time::{Instant, Duration};
use nalgebra::Vector3;
use obfstr::obfstr;
use rand::{Rng, thread_rng};
use utils_state::StateRegistry;
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

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{KeyToggle, ViewController},
    utils::ImGuiKey,
    UpdateContext,
};

// Structure de cible améliorée avec plus d'informations
#[derive(Clone, Debug)]
pub struct AimbotTargetEnhanced {
    pub entity_id: u32,
    pub position: Vector3<f32>,
    pub distance: f32,
    pub angle_diff: f32,
    pub bone_position: Vector3<f32>,
    pub bone_name: String,
    pub is_visible: bool,
    pub last_seen: Instant,
    pub address: u64,
}

// Structure pour stocker l'état de verrouillage sur une cible
struct TargetLock {
    pub entity_id: u32,
    pub locked_since: Instant,
    pub last_updated: Instant,
}

pub struct AimbotEnhanced {
    toggle: KeyToggle,
    current_target: Option<AimbotTargetEnhanced>,
    target_lock: Option<TargetLock>,
    last_shot_time: Instant,
    last_move_time: Instant,
    rng: rand::rngs::ThreadRng,
}

impl AimbotEnhanced {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            current_target: None,
            target_lock: None,
            last_shot_time: Instant::now(),
            last_move_time: Instant::now(),
            rng: thread_rng(),
        }
    }

    // Sélectionne la meilleure cible basée sur l'angle et la distance
    fn get_best_target(&self, ctx: &UpdateContext) -> anyhow::Result<Option<AimbotTargetEnhanced>> {
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

        let bone_map = BoneMap::new();
        
        let mut targets = Vec::new();
        
        // Parcourir toutes les entités pour trouver des cibles potentielles
        for entity_identity in entities.entities() {
            // Vérifier si c'est un joueur
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

            // Vérifier si le joueur est vivant
            if let Ok(pawn_state) = ctx.states.resolve::<PlayerPawnState>(entity_identity.handle()?) {
                if *pawn_state != PlayerPawnState::Alive {
                    continue;
                }
            } else {
                continue;
            }

            // Récupérer les informations du joueur
            let pawn_info = match ctx.states.resolve::<StatePawnInfo>(entity_identity.handle()?) {
                Ok(info) => info,
                Err(_) => continue,
            };
            
            // Vérifier que le joueur a de la vie
            if pawn_info.player_health <= 0 {
                continue;
            }            // Vérifier si c'est un ennemi
            if settings.aimbot_enhanced_team_check {
                if pawn_info.team_id == local_team {
                    continue;
                }
            }

            // Récupérer les informations du modèle et des os
            let pawn_model = match ctx.states.resolve::<StatePawnModelInfo>(entity_identity.handle()?) {
                Ok(model) => model,
                Err(_) => continue,
            };
            
            let entry_model = match ctx.states.resolve::<CS2Model>(pawn_model.model_address) {
                Ok(model) => model,
                Err(_) => continue,
            };

            // Extraire les noms des os
            let bone_names: Vec<String> = entry_model.bones.iter().map(|bone| bone.name.clone()).collect();
            
            // Définir les os prioritaires selon la configuration
            let body_parts = match settings.aimbot_target_bone {
                0 => vec![BodyPart::Head, BodyPart::Neck],
                1 => vec![BodyPart::UpperSpine, BodyPart::MiddleSpine],
                _ => vec![BodyPart::Pelvis, BodyPart::LowerSpine],
            };

            // Analyser chaque partie du corps configurée
            for body_part in body_parts {
                if let Some(bone_index) = bone_map.find_bone_index(body_part, &bone_names) {
                    if bone_index >= pawn_model.bone_states.len() {
                        continue;
                    }
                    
                    if let Some(bone_state) = pawn_model.bone_states.get(bone_index) {
                        let bone_position = bone_state.position;
                        
                        // Calculer la distance et l'angle par rapport au joueur
                        let distance = (bone_position - camera_position).norm();
                        
                        // Vérifier si la distance est dans les limites configurées
                        if settings.aimbot_max_distance > 0.0 && distance > settings.aimbot_max_distance {
                            continue;
                        }
                        
                        // Calculer l'angle par rapport au centre de l'écran
                        let Some(bone_screen_pos) = view.world_to_screen(&bone_position, false) else {
                            continue;
                        };
                        
                        let screen_center_x = view.screen_bounds.x / 2.0;
                        let screen_center_y = view.screen_bounds.y / 2.0;
                        
                        let delta_x = bone_screen_pos.x - screen_center_x;
                        let delta_y = bone_screen_pos.y - screen_center_y;
                        
                        let screen_distance = ((delta_x * delta_x) + (delta_y * delta_y)).sqrt();
                        
                        // Convertir la distance écran en angle FOV approximatif
                        let screen_radius = (view.screen_bounds.x.min(view.screen_bounds.y) / 2.0) as f32;
                        let angle_diff = (screen_distance / screen_radius) * (settings.aimbot_fov / 2.0);
                        
                        // Vérifier si la cible est dans le FOV configuré
                        if angle_diff < settings.aimbot_fov {
                            // Vérifier si la cible est visible (pas de check de visibilité exact)
                            let is_visible = true; // TODO: Implémenter un vrai check de visibilité
                            
                            targets.push(AimbotTargetEnhanced {
                                entity_id: entity_identity.handle::<()>()?.get_entity_index(),
                                position: pawn_info.position,
                                distance,
                                angle_diff,
                                bone_position,
                                bone_name: bone_names[bone_index].clone(),
                                is_visible,
                                last_seen: Instant::now(),
                                address: entity_identity.handle::<()>()?.get_entity_index() as u64,
                            });
                            
                            // Une fois qu'on a trouvé un os utilisable pour cette entité, on passe à la suivante
                            break;
                        }
                    }
                }
            }
        }

        // Si on est déjà verrouillé sur une cible, privilégier cette cible si elle est toujours valide
        if let Some(target_lock) = &self.target_lock {
            if let Some(locked_target) = targets.iter().find(|t| t.entity_id == target_lock.entity_id) {
                // La cible verrouillée est toujours valide, la renvoyer en priorité
                return Ok(Some(locked_target.clone()));
            }
        }

        // Trier les cibles par angle (le plus proche du centre de l'écran)
        targets.sort_by(|a, b| a.angle_diff.partial_cmp(&b.angle_diff).unwrap_or(std::cmp::Ordering::Equal));
        
        // Retourner la meilleure cible
        Ok(targets.into_iter().next())
    }

    // Fonction pour appliquer le mouvement de souris vers la cible
    fn apply_mouse_movement(&mut self, ctx: &UpdateContext, target: &AimbotTargetEnhanced, settings: &AppSettings, view: &ViewController) -> anyhow::Result<()> {
        // Calcul de la position de la cible sur l'écran
        let Some(target_screen) = view.world_to_screen(&target.bone_position, true) else {
            return Ok(());
        };

        // Coordonnées du centre de l'écran
        let screen_center_x = view.screen_bounds.x / 2.0;
        let screen_center_y = view.screen_bounds.y / 2.0;
        
        // Différence entre la position de la cible et le centre de l'écran
        let delta_x = target_screen.x - screen_center_x;
        let delta_y = target_screen.y - screen_center_y;
        
        // Inverser l'axe Y pour la logique FPS
        let delta_y = -delta_y;

        // Log pour diagnostiquer
        log::debug!("Target position: ({}, {}), delta: ({}, {})", 
            target_screen.x, target_screen.y, delta_x, delta_y);
          // Calculer le facteur de lissage (smoothing) avec une légère randomisation
        let base_smoothing = settings.aimbot_enhanced_smoothing.max(1.0);
        let randomize_factor = settings.aimbot_enhanced_randomize_factor;
        let smoothing_randomness = self.rng.gen_range(-randomize_factor..randomize_factor);
        let smoothing_factor = base_smoothing + smoothing_randomness;
        
        // Appliquer le lissage aux mouvements
        let mut smooth_x = delta_x / smoothing_factor;
        let mut smooth_y = delta_y / smoothing_factor;
        
        // Calculer un facteur d'amplification dynamique
        // Plus la cible est proche du réticule, plus le mouvement est précis
        let amplification_factor = if target.angle_diff < 3.0 {
            // Très près du réticule, mouvement plus précis
            1.0 + (1.0 - (target.angle_diff / 3.0)) * 0.5
        } else if target.angle_diff < 10.0 {
            // Relativement proche, mouvement standard
            1.0
        } else {            // Loin du réticule, mouvement plus rapide
            1.0 + (target.angle_diff / settings.aimbot_enhanced_fov) * 1.5
        };
        
        // Appliquer l'amplification
        smooth_x *= amplification_factor;
        smooth_y *= amplification_factor;
        
        // Convertir en entiers pour le mouvement de souris
        let mouse_x = (smooth_x) as i32;
        let mouse_y = (smooth_y) as i32;
        
        // Limiter les mouvements trop grands ou trop petits
        let max_movement = 80;
        let safe_mouse_x = mouse_x.clamp(-max_movement, max_movement);
        let safe_mouse_y = mouse_y.clamp(-max_movement, max_movement);
        
        // Vérifier si un mouvement est nécessaire
        if safe_mouse_x != 0 || safe_mouse_y != 0 {
            // Appliquer un délai entre les mouvements
            if self.last_move_time.elapsed() > Duration::from_millis(5) {
                log::debug!("Moving mouse: x={}, y={}", safe_mouse_x, safe_mouse_y);
                
                // Envoyer le mouvement de souris au jeu
                ctx.cs2.send_mouse_state(&[MouseState {
                    last_x: safe_mouse_x,
                    last_y: safe_mouse_y,
                    ..Default::default()
                }])?;
                
                self.last_move_time = Instant::now();
            }        }
        
        Ok(())
    }
    
    // Vérifier si auto-shoot doit être déclenché
    fn check_auto_shoot(&mut self, ctx: &UpdateContext, target: &AimbotTargetEnhanced, settings: &AppSettings, view: &ViewController) -> anyhow::Result<()> {
        if !settings.aimbot_enhanced_auto_shoot {
            return Ok(());
        }
        
        // Calculer la position à l'écran
        let Some(target_screen) = view.world_to_screen(&target.bone_position, true) else {
            return Ok(());
        };
        
        // Distance au centre de l'écran
        let screen_center_x = view.screen_bounds.x / 2.0;
        let screen_center_y = view.screen_bounds.y / 2.0;
        let delta_x = target_screen.x - screen_center_x;
        let delta_y = target_screen.y - screen_center_y;
        
        let center_distance = ((delta_x * delta_x) + (delta_y * delta_y)).sqrt();
        
        // Ne tirer que si la cible est suffisamment proche du réticule
        // et qu'on attend assez longtemps depuis le dernier tir
        let shoot_threshold = 10.0; // Pixels
        if center_distance < shoot_threshold && self.last_shot_time.elapsed() > Duration::from_millis(100) {
            log::debug!("Auto-shooting at distance {}", center_distance);
            
            // Envoyer un clic gauche
            ctx.cs2.send_mouse_state(&[MouseState {
                buttons: [Some(true), None, None, None, None],
                ..Default::default()
            }])?;
            
            // Stocker le moment du tir
            self.last_shot_time = Instant::now();
            
            // Relâcher le clic après un court délai
            std::thread::sleep(Duration::from_millis(20));
              ctx.cs2.send_mouse_state(&[MouseState {
                buttons: [Some(false), None, None, None, None],
                ..Default::default()
            }])?;
        }
        
        Ok(())
    }
    
    // Gérer le système de verrouillage de cible
    fn manage_target_locking(&mut self, new_target: &Option<AimbotTargetEnhanced>, settings: &AppSettings) {
        // Si le verrouillage de cible est désactivé, ne pas utiliser cette fonctionnalité
        if !settings.aimbot_enhanced_target_lock {
            self.target_lock = None;
            return;
        }
        
        match (new_target, &self.target_lock) {
            // Nouvelle cible trouvée
            (Some(target), None) => {
                // Créer un nouveau verrouillage
                self.target_lock = Some(TargetLock {
                    entity_id: target.entity_id,
                    locked_since: Instant::now(),
                    last_updated: Instant::now(),
                });
            },
            
            // Mise à jour d'une cible existante
            (Some(target), Some(lock)) if target.entity_id == lock.entity_id => {
                // Mettre à jour le temps de dernière visibilité
                if let Some(ref mut lock) = self.target_lock {
                    lock.last_updated = Instant::now();
                }
            },
            
            // Changement de cible
            (Some(target), Some(_)) => {
                // Créer un nouveau verrouillage pour la nouvelle cible
                self.target_lock = Some(TargetLock {
                    entity_id: target.entity_id,
                    locked_since: Instant::now(),
                    last_updated: Instant::now(),
                });
            },
            
            // Perte de cible
            (None, Some(lock)) => {                // Vérifier si le verrouillage doit être conservé
                let lock_timeout = Duration::from_millis(settings.aimbot_enhanced_lock_duration.into());
                if lock.last_updated.elapsed() > lock_timeout {
                    // Timeout dépassé, supprimer le verrouillage
                    self.target_lock = None;
                }
                // Sinon, conserver le verrouillage jusqu'au timeout
            },
              // Pas de cible, pas de verrouillage
            (None, None) => {},
        }
    }
}

impl AimbotEnhanced {
    // Méthode pour dessiner le cercle FOV à l'écran
    fn render_fov_circle(&self, ui: &imgui::Ui, view: &ViewController, settings: &AppSettings) {
        // Vérifier si l'affichage du cercle FOV est activé
        if !settings.aimbot_enhanced_draw_fov {
            return;
        }
        
        // Calculer le rayon du cercle FOV en fonction du FOV et de la taille de l'écran
        let screen_size = view.screen_bounds;
        let _aspect_ratio = screen_size.x / screen_size.y;
        let screen_center = [screen_size.x / 2.0, screen_size.y / 2.0];
        
        // Convertir le FOV en pixels en utilisant la hauteur de l'écran comme référence
        // Cette formule approchée donne des résultats cohérents avec l'implémentation du jeu
        let y_scale = screen_size.y / 2.0;
        let fov_scale = (settings.aimbot_enhanced_fov * 3.14159 / 180.0).tan();
        let fov_radius = y_scale * fov_scale;
        
        // Dessiner le cercle
        let draw = ui.get_window_draw_list();
        draw.add_circle(
            screen_center, 
            fov_radius, 
            [0.9, 0.4, 0.1, 0.5]  // Couleur orange semi-transparente
        )
        .filled(false)
        .thickness(1.5)
        .num_segments(64)  // Plus de segments pour un cercle plus lisse
        .build();
    }
}

impl Enhancement for AimbotEnhanced {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;

        // Mettre à jour l'état du toggle
        if self.toggle.update(
            &settings.aimbot_enhanced_mode,
            ctx.input,
            &settings.key_aimbot_enhanced,
        ) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-enhanced-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, settings.aimbot_enhanced_mode
                ),
            );
        }

        // Si l'aimbot est désactivé, ne rien faire
        if !self.toggle.enabled {
            self.current_target = None;
            self.target_lock = None;
            return Ok(());
        }        // Trouver la meilleure cible
        let new_target = self.get_best_target(ctx)?;
        
        // Gérer le système de verrouillage de cible
        self.manage_target_locking(&new_target, &settings);
        
        // Stocker la cible actuelle
        self.current_target = new_target;
          // Si on a une cible, appliquer les mouvements de souris
        if let Some(target) = self.current_target.clone() {
            let view = ctx.states.resolve::<ViewController>(())?;
            
            // Appliquer le mouvement de souris vers la cible
            self.apply_mouse_movement(ctx, &target, &settings, &view)?;
            
            // Vérifier si on doit tirer automatiquement
            self.check_auto_shoot(ctx, &target, &settings, &view)?;
        }
        
        Ok(())
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        _unicode_text: &overlay::UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        if !self.toggle.enabled {
            return Ok(());
        }

        let view = states.resolve::<ViewController>(())?;
        let settings = states.resolve::<AppSettings>(())?;
        
        // Dessiner le cercle FOV
        self.render_fov_circle(ui, &view, &settings);
        
        // Dessiner des informations sur la cible si nécessaire
        if let Some(target) = &self.current_target {
            let Some(target_screen) = view.world_to_screen(&target.bone_position, true) else {
                return Ok(());
            };
            
            // Dessiner un indicateur de cible
            let draw = ui.get_window_draw_list();
            draw.add_circle(target_screen, 5.0, [1.0, 0.0, 0.0, 0.8])
                .filled(false)
                .thickness(2.0)
                .build();
                
            // Dessiner une ligne du centre vers la cible
            let screen_center = [ui.window_size()[0] / 2.0, ui.window_size()[1] / 2.0];
            draw.add_line(
                screen_center,
                target_screen,
                [1.0, 0.0, 0.0, 0.5]            )
            .thickness(1.0)
            .build();
        }
        
        Ok(())
    }
    
    fn update_settings(
        &mut self,
        ui: &imgui::Ui,
        settings: &mut AppSettings,
    ) -> anyhow::Result<bool> {
        let mut changed = false;

        if ui.collapsing_header("Aimbot Amélioré", imgui::TreeNodeFlags::empty()) {
            ui.text("Mode:");
            let mut mode_index_usize = match settings.aimbot_enhanced_mode {
                crate::settings::KeyToggleMode::AlwaysOn => 0,
                crate::settings::KeyToggleMode::Toggle => 1,
                crate::settings::KeyToggleMode::Trigger => 2,
                crate::settings::KeyToggleMode::TriggerInverted => 3,
                crate::settings::KeyToggleMode::Off => 4,
            } as usize;
            
            if ui.combo_simple_string(
                "##aimbot_enhanced_mode",
                &mut mode_index_usize,
                &["Always On", "Toggle", "Trigger", "Trigger Inverted", "Off"],
                        ) {
                settings.aimbot_enhanced_mode = match mode_index_usize {
                    0 => crate::settings::KeyToggleMode::AlwaysOn,
                    1 => crate::settings::KeyToggleMode::Toggle,
                    2 => crate::settings::KeyToggleMode::Trigger,
                    3 => crate::settings::KeyToggleMode::TriggerInverted,
                    _ => crate::settings::KeyToggleMode::Off,
                };                changed = true;
            }
            
            ui.separator();
            ui.text("Paramètres de touche:");
              if ui.button_key_optional("Touche Aimbot", &mut settings.key_aimbot_enhanced, [200.0, 0.0]) {
                changed = true;
            }
            
            ui.separator();
            ui.text("Paramètres généraux:");

            ui.checkbox("Vérification de l'équipe", &mut settings.aimbot_enhanced_team_check);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("FOV", 1.0, 180.0, &mut settings.aimbot_enhanced_fov);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.checkbox("Afficher le cercle FOV", &mut settings.aimbot_enhanced_draw_fov);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("Lissage", 0.1, 20.0, &mut settings.aimbot_enhanced_smoothing);
            if ui.is_item_edited() {
                changed = true;
            }
            
            ui.slider("Facteur aléatoire", 0.0, 1.0, &mut settings.aimbot_enhanced_randomize_factor);
            if ui.is_item_edited() {
                changed = true;
            }

            let mut bone_index = settings.aimbot_enhanced_target_bone as usize;
            if ui.combo_simple_string(
                "Point de visée",
                &mut bone_index,
                &["Tête", "Torse", "Bassin"],
            ) {
                settings.aimbot_enhanced_target_bone = bone_index as u32;
                changed = true;
            }
            
            ui.separator();
            ui.text("Paramètres avancés:");

            ui.checkbox("Verrouillage de cible", &mut settings.aimbot_enhanced_target_lock);
            if ui.is_item_edited() {
                changed = true;
            }
            
            if settings.aimbot_enhanced_target_lock {
                ui.slider_config("Durée de verrouillage (ms)", 100, 2000)
                    .display_format("%.0f ms")
                    .build(&mut settings.aimbot_enhanced_lock_duration);
                if ui.is_item_edited() {
                    changed = true;
                }
            }

            ui.checkbox("Tir automatique", &mut settings.aimbot_enhanced_auto_shoot);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.slider("Distance max", 50.0, 1000.0, &mut settings.aimbot_enhanced_max_distance);
            if ui.is_item_edited() {
                changed = true;
            }

            ui.checkbox("Viser seulement les cibles visibles", &mut settings.aimbot_enhanced_visible_only);
            if ui.is_item_edited() {
                changed = true;
            }
        }

        Ok(changed)
    }
}
