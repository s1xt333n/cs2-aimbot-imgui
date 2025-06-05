use std::{
    borrow::Cow,
    collections::{
        btree_map::Entry,
        HashMap,
    },
    fs::File,
    io::{
        BufReader,
        Write,
    },
    path::PathBuf,
    sync::{
        atomic::Ordering,
        Arc,
        Mutex,
    },
    thread,
    time::Instant,
};

use anyhow::Context;
use cs2::{
    StateBuildInfo,
    StateCurrentMap,
};
use imgui::{
    Condition,
    ImColor32,
    SelectableFlags,
    StyleColor,
    StyleVar,
    TableColumnFlags,
    TableColumnSetup,
    TableFlags,
    TreeNodeFlags,
};
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use super::{
    Color,
    EspColor,
    EspColorType,
    EspConfig,
    EspSelector,
    GrenadeSettings,
    GrenadeSortOrder,
    GrenadeSpotInfo,
    GrenadeType,
    KeyToggleMode,
};

use crate::utils::neuomorphic::NeuomorphicUi;
use crate::{
    enhancements::StateGrenadeHelperPlayerLocation,
    settings::{
        AppSettings,
        EspBoxType,
        EspHeadDot,
        EspHealthBar,
        EspPlayerSettings,
        EspTracePosition,
    },
    utils::{
        ImGuiKey,
        ImguiComboEnum,
    },
    Application,
};

enum EspPlayerActiveHeader {
    Features,
    Style,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum GrenadeSettingsTarget {
    General,
    MapType(String),
    Map {
        map_name: String,
        display_name: String,
    },
}

impl GrenadeSettingsTarget {
    pub fn ui_token(&self) -> Cow<'static, str> {
        match self {
            Self::General => "_settings".into(),
            Self::MapType(value) => format!("map_type_{}", value).into(),
            Self::Map { map_name: name, .. } => format!("map_{}", name).into(),
        }
    }

    pub fn ident_level(&self) -> usize {
        match self {
            Self::General => 0,
            Self::MapType(_) => 0,
            Self::Map { .. } => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum GrenadeHelperTransferDirection {
    Export,
    Import,
}

enum GrenadeHelperTransferState {
    /// Currently no transfer in progress
    Idle,    /// A new transfer should be initiated.
    Pending {
        #[allow(dead_code)]
        direction: GrenadeHelperTransferDirection,
    },    /// A transfer has been initiated.
    /// This might be either an export or import.
    Active {
        #[allow(dead_code)]
        direction: GrenadeHelperTransferDirection,
    },
    /// The current transfer failed.
    Failed {
        direction: GrenadeHelperTransferDirection,
        message: String,
    },
    /// The source file has been loaded.
    /// Prompting the user, if he wants to replace or add the new items.
    ImportPending {
        elements: HashMap<String, Vec<GrenadeSpotInfo>>,
    },    ImportSuccess {
        count: usize,
        #[allow(dead_code)]
        replacing: bool,
    },
    ExportSuccess {
        target_path: PathBuf,
    },
}

pub struct SettingsUI {
    discord_link_copied: Option<Instant>,

    esp_selected_target: EspSelector,
    esp_pending_target: Option<EspSelector>,
    esp_player_active_header: EspPlayerActiveHeader,

    grenade_helper_target: GrenadeSettingsTarget,
    grenade_helper_selected_id: usize,
    grenade_helper_skip_confirmation_dialog: bool,
    grenade_helper_new_item: Option<GrenadeSpotInfo>,
    grenade_helper_transfer_state: Arc<Mutex<GrenadeHelperTransferState>>,

    grenade_helper_pending_target: Option<GrenadeSettingsTarget>,
    grenade_helper_pending_selected_id: Option<usize>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl SettingsUI {
    pub fn new() -> Self {
        Self {
            discord_link_copied: None,

            esp_selected_target: EspSelector::None,
            esp_pending_target: None,
            esp_player_active_header: EspPlayerActiveHeader::Features,

            grenade_helper_target: GrenadeSettingsTarget::General,
            grenade_helper_selected_id: 0,
            grenade_helper_new_item: None,
            grenade_helper_skip_confirmation_dialog: false,
            grenade_helper_transfer_state: Arc::new(Mutex::new(GrenadeHelperTransferState::Idle)),            grenade_helper_pending_target: None,
            grenade_helper_pending_selected_id: None,
        }
    }

    pub fn render(
        &mut self,
        app: &Application,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) {
        let content_font = ui.current_font().id();
        let _title_font = if let Some(font_id) = app.fonts.valthrun.font_id() {
            ui.push_font(font_id)
        } else {
            return;
        };
        use crate::utils::ragnarek::RagnarekUi;
        ui.window(obfstr!("Valthrun"))
            .size([750.0, 450.0], Condition::FirstUseEver)
            .size_constraints([750.0, 450.0], [2000.0, 2000.0])
            .title_bar(false)
            .build(|| {
                // Apply combined neuomorphic and RAGNAREK styling for the main window
                let neuomorphic_style = crate::settings::get_neuomorphic_style();
                use crate::utils::neuomorphic::NeuomorphicUi;
                ui.push_neuomorphic_window_style(&neuomorphic_style);
                
                // Get window position and size for background effects
                let window_pos = ui.window_pos();
                let window_size = ui.window_size();
                
                // Add subtle glow to window top border using a single draw list call
                {
                    let draw_list = ui.get_window_draw_list();
                    draw_list.add_rect_filled_multicolor(
                        [window_pos[0], window_pos[1]],
                        [window_pos[0] + window_size[0], window_pos[1] + 3.0],
                        [0.6, 0.2, 0.8, 0.8],  // Left
                        [0.2, 0.4, 0.9, 0.8],  // Right
                        [0.2, 0.4, 0.9, 0.0],  // Bottom right
                        [0.6, 0.2, 0.8, 0.0]   // Bottom left
                    );
                }
                
                // RAGNAREK-styled title with glow effect
                use crate::utils::ragnarek::RagnarekUi;
                let glow_color = [0.6, 0.3, 0.9, 0.6]; // Purple glow
                ui.ragnarek_text_with_glow("VALTHRUN", [1.0, 1.0, 1.0, 1.0], glow_color);
                
                // Version number with smaller text
                ui.same_line();
                ui.set_cursor_pos([ui.cursor_pos()[0] + 10.0, ui.cursor_pos()[1] + 5.0]);
                ui.text_colored([0.7, 0.7, 0.7, 0.8], &format!("v{}", VERSION));
                
                ui.spacing();
                ui.ragnarek_separator_gradient(
                    [0.81, 0.69, 0.06, 0.8], // Gold
                    [1.00, 0.11, 0.68, 0.8]  // Pink
                );

                let _content_font = ui.push_font(content_font);
                let mut settings = app.settings_mut();

                // Custom tab bar with RAGNAREK styling
                let tab_bar_height = 40.0;
                let tab_spacing = 5.0;
                let tab_names = ["Information", "Visuals", "ESP", "Grenade Helper", "Aim Assist", "Web Radar", "Theme", "Misc", "Hotkeys"];
                static mut SELECTED_TAB: usize = 0;
                
                let tab_area_start = ui.cursor_pos();
                let tab_area_width = ui.content_region_avail()[0];
                let tab_width = (tab_area_width - (tab_names.len() as f32 - 1.0) * tab_spacing) / tab_names.len() as f32;
                
                for (i, &name) in tab_names.iter().enumerate() {
                    if i > 0 {                        ui.same_line_with_spacing(0.0, tab_spacing);
                    }
                    let selected = unsafe { SELECTED_TAB == i };
                    if ui.ragnarek_tab(selected, "", name, [tab_width, tab_bar_height]) {
                        unsafe { SELECTED_TAB = i };
                    }
                }
                  ui.spacing();
                  // Custom tab content rendering based on selected tab
                match unsafe { SELECTED_TAB } {
                    0 => { // Information
                        let build_info = app.app_state.resolve::<StateBuildInfo>(()).ok();

                        ui.text(obfstr!("Valthrun an open source CS2 external read only kernel gameplay enhancer."));
                        ui.text(&format!("{} Version {} ({})", obfstr!("Valthrun"), VERSION, env!("BUILD_TIME")));
                        ui.text(&format!("{} Version {} ({})", obfstr!("CS2"), build_info.as_ref().map_or("error", |info| &info.revision), build_info.as_ref().map_or("error", |info| &info.build_datetime)));

                        let ydummy = ui.window_size()[1] - ui.cursor_pos()[1] - ui.text_line_height_with_spacing() * 2.0 - 12.0;
                        ui.dummy([0.0, ydummy]);
                        ui.separator();

                        ui.text(obfstr!("Join our discord:"));
                        ui.text_colored([0.18, 0.51, 0.97, 1.0], obfstr!("https://discord.gg/ecKbpAPW5T"));
                        if ui.is_item_hovered() {
                            ui.set_mouse_cursor(Some(imgui::MouseCursor::Hand));
                        }

                        if ui.is_item_clicked() {
                            self.discord_link_copied = Some(Instant::now());
                            ui.set_clipboard_text(obfstr!("https://discord.gg/ecKbpAPW5T"));
                        }

                        let show_copied = self.discord_link_copied.as_ref()
                            .map(|time| time.elapsed().as_millis() < 3_000)
                            .unwrap_or(false);

                        if show_copied {
                            ui.same_line();
                            ui.text("(Copied)");
                        }                    },
                    1 => { // Visuals
                        ui.set_next_item_width(150.0);
                        let neuomorphic_style = crate::settings::get_neuomorphic_style();
                        use crate::utils::neuomorphic::NeuomorphicUi;
                        ui.neuomorphic_combo_enum(obfstr!("ESP"), &[
                            (KeyToggleMode::Off, "Always Off"),
                            (KeyToggleMode::Trigger, "Trigger"),
                            (KeyToggleMode::TriggerInverted, "Trigger Inverted"),
                            (KeyToggleMode::Toggle, "Toggle"),
                            (KeyToggleMode::AlwaysOn, "Always On"),
                        ], &mut settings.esp_mode, &neuomorphic_style);

                        let neuomorphic_style = crate::settings::get_neuomorphic_style();
                        ui.neuomorphic_checkbox(obfstr!("Bomb Timer"), &mut settings.bomb_timer, &neuomorphic_style);
                        ui.neuomorphic_checkbox(obfstr!("Spectators List"), &mut settings.spectators_list, &neuomorphic_style);
                        ui.neuomorphic_checkbox(obfstr!("Grenade Helper"), &mut settings.grenade_helper.active, &neuomorphic_style);
                        ui.neuomorphic_checkbox(obfstr!("Sniper Crosshair"), &mut settings.sniper_crosshair, &neuomorphic_style);
                    },
                    2 => { // ESP
                        if settings.esp_mode == KeyToggleMode::Off {
                            let _style = ui.push_style_color(StyleColor::Text, [1.0, 0.76, 0.03, 1.0]);
                            ui.text(obfstr!("ESP has been disabled."));
                            ui.text(obfstr!("Please enable ESP under \"Visuals\" > \"ESP\""));
                        } else {
                            self.render_esp_settings(&mut *settings, ui);
                        }
                    },
                    3 => { // Grenade Helper
                        if settings.grenade_helper.active {
                            self.render_grenade_helper(&app.app_state, &mut settings.grenade_helper, ui, unicode_text);
                        } else {
                            let _style = ui.push_style_color(StyleColor::Text, [1.0, 0.76, 0.03, 1.0]);
                            ui.text(obfstr!("Grenade Helper has been disabled."));
                            ui.text(obfstr!("Please enable the grenade helper under \"Visuals\" > \"Grenade Helper\""));
                        }                        self.render_grenade_helper_transfer(&mut settings.grenade_helper, ui);
                    },
                    4 => { // Aim Assist
                        ui.set_next_item_width(150.0);
                        let neuomorphic_style = crate::settings::get_neuomorphic_style();
                        use crate::utils::neuomorphic::NeuomorphicUi;
                        ui.neuomorphic_combo_enum(obfstr!("Trigger Bot"), &[
                            (KeyToggleMode::Off, "Always Off"),
                            (KeyToggleMode::Trigger, "Trigger"),
                            (KeyToggleMode::TriggerInverted, "Trigger Inverted"),
                            (KeyToggleMode::Toggle, "Toggle"),
                            (KeyToggleMode::AlwaysOn, "Always On"),
                        ], &mut settings.trigger_bot_mode, &neuomorphic_style);

                        if !matches!(settings.trigger_bot_mode, KeyToggleMode::Off | KeyToggleMode::AlwaysOn) {
                            ui.button_key_optional(obfstr!("Trigger bot key"), &mut settings.key_trigger_bot, [150.0, 0.0]);
                        }
                        if !matches!(settings.trigger_bot_mode, KeyToggleMode::Off) {
                            let mut values_updated = false;
                            let slider_width = (ui.current_column_width() / 2.0 - 80.0).min(300.0).max(50.0);
                            let slider_width_1 = (ui.current_column_width() / 2.0 - 20.0).min(300.0).max(50.0);
                            ui.text(obfstr!("Trigger delay min: "));
                            ui.same_line();
                            ui.set_next_item_width(slider_width);
                            
                            let neuomorphic_style = crate::settings::get_neuomorphic_style();
                            // Convert u32 to f32 for slider, then back
                    let mut delay_min_f32 = settings.trigger_bot_delay_min as f32;
                    let result = ui.neuomorphic_slider_f32("##delay_min", &mut delay_min_f32, 0.0, 300.0, "%0.0fms", &neuomorphic_style);
                    settings.trigger_bot_delay_min = delay_min_f32 as u32;
                    values_updated |= result;
                            ui.same_line();

                            ui.text(" max: ");
                            ui.same_line();
                            ui.set_next_item_width(slider_width);
                            // Convert u32 to f32 for slider, then back
                    let mut delay_max_f32 = settings.trigger_bot_delay_max as f32;
                    let result = ui.neuomorphic_slider_f32("##delay_max", &mut delay_max_f32, 0.0, 300.0, "%0.0fms", &neuomorphic_style);
                    settings.trigger_bot_delay_max = delay_max_f32 as u32;
                    values_updated |= result;

                            ui.text(obfstr!("Shoot duration: "));
                            ui.same_line();
                            ui.set_next_item_width(slider_width_1);
                            // Convert u32 to f32 for slider, then back
                    let mut shot_duration_f32 = settings.trigger_bot_shot_duration as f32;
                    let result = ui.neuomorphic_slider_f32("##shoot_duration", &mut shot_duration_f32, 0.0, 1000.0, "%0.0fms", &neuomorphic_style);
                    settings.trigger_bot_shot_duration = shot_duration_f32 as u32;
                    values_updated |= result;

                            if values_updated {
                                /* fixup min/max */
                                let delay_min = settings.trigger_bot_delay_min.min(settings.trigger_bot_delay_max);
                                let delay_max = settings.trigger_bot_delay_min.max(settings.trigger_bot_delay_max);

                                settings.trigger_bot_delay_min = delay_min;
                                settings.trigger_bot_delay_max = delay_max;                            }

                            let neuomorphic_style = crate::settings::get_neuomorphic_style();
                            ui.neuomorphic_checkbox(obfstr!("Retest trigger target after delay"), &mut settings.trigger_bot_check_target_after_delay, &neuomorphic_style);
                            ui.neuomorphic_checkbox(obfstr!("Team Check"), &mut settings.trigger_bot_team_check, &neuomorphic_style);
                            ui.separator();
                        }                        let neuomorphic_style = crate::settings::get_neuomorphic_style();
                        ui.neuomorphic_checkbox("Simple Recoil Helper", &mut settings.aim_assist_recoil, &neuomorphic_style);
                    },
                    5 => { // Web Radar
                        ui.text(obfstr!("Operating the Valthrun Web Radar within the Valthrun Overlay is no longer supported."));
                        ui.text(obfstr!("Please use the standalone radar client."));
                    },                    6 => { // Theme
                        use crate::utils::neuomorphic::NeuomorphicUi;
                        let neuomorphic_style = crate::settings::get_neuomorphic_style();                        // RAGNAREK styled heading
                        ui.ragnarek_text_with_glow(
                            "Theme Settings",
                            [1.0, 1.0, 1.0, 1.0],
                            [0.5, 0.2, 0.9, 0.5]
                        );
                        ui.ragnarek_separator_gradient(
                            [0.3, 0.1, 0.6, 1.0],
                            [0.7, 0.3, 0.9, 1.0]
                        );
                        
                        // Theme style selection with RAGNAREK child window
                        ui.ragnarek_child_window("Theme Options", [ui.content_region_avail()[0] * 0.9, 0.0], || {
                            // Theme toggle
                            static mut THEME_ENABLED: bool = true;
                            static mut USE_RAGNAREK: bool = false;
                            unsafe {
                                if ui.ragnarek_checkbox("Enable Neuomorphic Theme", &mut THEME_ENABLED) {
                                    crate::settings::set_neuomorphic_enabled(THEME_ENABLED);
                                }
                                
                                // Option to use RAGNAREK styling (just visual for now)
                                ui.ragnarek_checkbox("Enable RAGNAREK UI Enhancements", &mut USE_RAGNAREK);
                                
                                if USE_RAGNAREK {
                                    ui.ragnarek_tooltip(|| {
                                        ui.text("RAGNAREK UI provides enhanced visual effects with glowing elements, gradients and animations");
                                    });
                                }
                            }
                            
                            ui.spacing();
                              // Theme variant buttons using RAGNAREK tabs
                            ui.text("Theme Variant:");                              let dark_selected = unsafe { crate::settings::is_dark_theme() };
                            let light_selected = !dark_selected;
                            if ui.ragnarek_tab(dark_selected, "🌑", "Dark", [120.0, 35.0]) {
                                crate::settings::switch_to_dark_theme();}
                              ui.same_line();
                            if ui.ragnarek_tab(light_selected, "☀️", "Light", [120.0, 35.0]) {
                                crate::settings::switch_to_light_theme();
                            }});
                          ui.spacing();
                        ui.ragnarek_separator_gradient(
                            [0.7, 0.3, 0.9, 1.0],
                            [0.3, 0.1, 0.6, 1.0]
                        );
                        ui.spacing();
                        
                        // Accent color picker with RAGNAREK styling
                        ui.text("Accent Color:");
                        static mut ACCENT_COLOR: [f32; 4] = [0.54, 0.17, 0.89, 1.0]; // Purple default
                        
                        unsafe {                            
                            if ui.ragnarek_color_picker(&"##accent_color", &mut ACCENT_COLOR) {
                                let color_bytes = [
                                    (ACCENT_COLOR[0] * 255.0) as u8,
                                    (ACCENT_COLOR[1] * 255.0) as u8,
                                    (ACCENT_COLOR[2] * 255.0) as u8,
                                    (ACCENT_COLOR[3] * 255.0) as u8,
                                ];
                                crate::settings::set_accent_color(color_bytes);
                            }
                              // Add theme loading progress demo with RAGNAREK progress bar
                            ui.spacing();
                            ui.spacing();
                            ui.text("Theme Loading:");
                            
                            // Animated progress value for demo
                            static mut PROGRESS: f32 = 0.75;
                            static mut LAST_UPDATE: std::time::Instant = unsafe { std::mem::zeroed() };
                            
                            unsafe {
                                if LAST_UPDATE.elapsed().as_millis() == 0 {
                                    LAST_UPDATE = std::time::Instant::now();
                                }
                                
                                if LAST_UPDATE.elapsed().as_millis() > 50 {
                                    PROGRESS = (PROGRESS + 0.01) % 1.0;
                                    if PROGRESS < 0.1 { PROGRESS = 0.1; }                                LAST_UPDATE = std::time::Instant::now();
                                }                            }
                              ui.ragnarek_progress_bar(PROGRESS, [ui.content_region_avail()[0] * 0.75, 20.0], Some(&format!("{:.0}%", PROGRESS * 100.0)));
ui.spacing();
ui.ragnarek_separator_gradient([0.1, 0.8, 0.1, 1.0], [0.8, 0.1, 0.1, 1.0]);
ui.spacing();
ui.ragnarek_text_with_glow("RAGNAREK Effects Showcase", [1.0, 1.0, 1.0, 1.0], [0.3, 0.6, 1.0, 0.8]);
ui.spacing();
if ui.ragnarek_animated_button("🚀 Pulse Effect Button", [200.0, 35.0], 1.0) {
    // Button clicked - could trigger effects
}
ui.same_line();
static mut DEMO_TOGGLE: bool = true;
ui.ragnarek_toggle_switch("Demo Toggle", unsafe { &mut DEMO_TOGGLE }, [100.0, 25.0]);
ui.spacing();
ui.text("System Status:");
ui.ragnarek_status_indicator("Online", [0.2, 0.8, 0.2, 1.0], true);
ui.same_line();
ui.text("Connected");
ui.spacing();
ui.text("Loading:");
ui.same_line();
ui.ragnarek_loading_spinner(8.0, 2.0, 1.0);
ui.spacing();
ui.text("Particle Effects:");
ui.ragnarek_particle_background([ui.content_region_avail()[0] * 0.8, 40.0], 15, [0.3, 0.6, 1.0, 0.3]);
ui.spacing();
ui.ragnarek_separator_gradient([0.8, 0.4, 0.0, 1.0], [1.0, 0.8, 0.0, 1.0]);
ui.spacing();
ui.ragnarek_text_with_glow("Color Presets", [1.0, 1.0, 1.0, 1.0], [0.8, 0.4, 0.0, 0.6]);
                        
                        let presets = [
                            ("Purple", [138, 43, 226, 255]),
                            ("Blue", [30, 144, 255, 255]),
                            ("Green", [50, 205, 50, 255]),
                            ("Orange", [255, 140, 0, 255]),
                            ("Pink", [255, 20, 147, 255]),
                            ("Cyan", [0, 206, 209, 255]),
                        ];
                        
                        for (i, (name, color)) in presets.iter().enumerate() {
                            if i > 0 && i % 3 == 0 {
                                // New line after every 3 buttons
                            } else if i > 0 {
                                ui.same_line();
                            }
                            
                            let button_color = imgui::ImColor32::from_rgba(color[0], color[1], color[2], color[3]);
                            let _token1 = ui.push_style_color(StyleColor::Button, to_f32_color(button_color));
                            let _token2 = ui.push_style_color(StyleColor::ButtonHovered, to_f32_color(button_color));
                            let _token3 = ui.push_style_color(StyleColor::ButtonActive, to_f32_color(button_color));
                              if ui.neuomorphic_button(name, [75.0, 25.0], &neuomorphic_style) {
                                crate::settings::set_accent_color(*color);
                                unsafe {
                                    ACCENT_COLOR = [
                                        color[0] as f32 / 255.0,
                                        color[1] as f32 / 255.0,
                                        color[2] as f32 / 255.0,
                                        color[3] as f32 / 255.0,
                                    ];
                                }
                            }                        }
                    },
                    7 => { // Misc
                        let neuomorphic_style = crate::settings::get_neuomorphic_style();
                        ui.neuomorphic_checkbox(obfstr!("Valthrun Watermark"), &mut settings.valthrun_watermark, &neuomorphic_style);

                        if ui.neuomorphic_checkbox(obfstr!("Hide overlay from screen capture"), &mut settings.hide_overlay_from_screen_capture, &neuomorphic_style) {
                            app.settings_screen_capture_changed.store(true, Ordering::Relaxed);
                        }
                        if ui.neuomorphic_checkbox(obfstr!("Show render debug overlay"), &mut settings.render_debug_window, &neuomorphic_style) {
                            app.settings_render_debug_window_changed.store(true, Ordering::Relaxed);
                        }
                    },
                    8 => { // Hotkeys
                        ui.button_key(obfstr!("Toggle Settings"), &mut settings.key_settings, [150.0, 0.0]);

                        {
                            let _enabled = ui.begin_enabled(matches!(settings.esp_mode, KeyToggleMode::Toggle | KeyToggleMode::Trigger));
                            ui.button_key_optional(obfstr!("ESP toggle/trigger"), &mut settings.esp_toogle, [150.0, 0.0]);
                        }                    },
                    _ => {}
                }
                ui.pop_neuomorphic_window_style();
                  // Add final RAGNAREK touches - subtle glow effect at the bottom of the window
                  // Add subtle bottom border glow
                {
                    let draw_list = ui.get_window_draw_list();
                    draw_list.add_rect_filled_multicolor(
                        [window_pos[0], window_pos[1] + window_size[1] - 3.0],
                        [window_pos[0] + window_size[0], window_pos[1] + window_size[1]],
                        [0.3, 0.1, 0.5, 0.0],   // Left
                        [0.1, 0.3, 0.6, 0.0],   // Right
                        [0.1, 0.3, 0.6, 0.4],   // Bottom right                        [0.3, 0.1, 0.5, 0.4]    // Bottom left
                    );
                }
            });
    }

    fn render_esp_target(
        &mut self,
        settings: &mut AppSettings,
        ui: &imgui::Ui,
        target: &EspSelector,
    ) {
        let config_key = target.config_key();
        let target_enabled = settings
            .esp_settings_enabled
            .get(&config_key)
            .cloned()
            .unwrap_or_default();

        let parent_enabled = target_enabled || {
            let mut current = target.parent();
            while let Some(parent) = current.take() {
                let enabled = settings
                    .esp_settings_enabled
                    .get(&parent.config_key())
                    .cloned()
                    .unwrap_or_default();

                if enabled {
                    current = Some(parent);
                    break;
                }

                current = parent.parent();
            }

            current.is_some()
        };

        {
            let pos_begin = ui.cursor_screen_pos();
            let clicked = ui
                .selectable_config(format!(
                    "{} ##{}",
                    target.config_display(),
                    target.config_key()
                ))
                .selected(target == &self.esp_selected_target)
                .flags(SelectableFlags::SPAN_ALL_COLUMNS)
                .build();

            let indicator_color = if target_enabled {
                ImColor32::from_rgb(0x4C, 0xAF, 0x50)
            } else if parent_enabled {
                ImColor32::from_rgb(0xFF, 0xC1, 0x07)
            } else {
                ImColor32::from_rgb(0xF4, 0x43, 0x36)
            };
            let pos_end = ui.cursor_screen_pos();
            let indicator_radius = ui.current_font_size() * 0.25;

            ui.get_window_draw_list()
                .add_circle(
                    [
                        pos_begin[0] - indicator_radius - 5.0,
                        pos_begin[1] + (pos_end[1] - pos_begin[1]) / 2.0 - indicator_radius / 2.0,
                    ],
                    indicator_radius,
                    indicator_color,
                )
                .filled(true)
                .build();

            if clicked {
                self.esp_pending_target = Some(target.clone());
            }
        }

        let children = target.children();
        if children.len() > 0 {
            ui.indent();
            for child in children.iter() {
                self.render_esp_target(settings, ui, child);
            }
            ui.unindent();
        }
    }

    fn render_esp_settings_player(
        &mut self,
        settings: &mut AppSettings,
        ui: &imgui::Ui,
        target: EspSelector,
    ) {
        let config_key = target.config_key();
        let config_enabled = settings
            .esp_settings_enabled
            .get(&config_key)
            .cloned()
            .unwrap_or_default();

        let config = match settings.esp_settings.entry(config_key.clone()) {
            Entry::Occupied(entry) => {
                let value = entry.into_mut();
                if let EspConfig::Player(value) = value {
                    value
                } else {
                    log::warn!("Detected invalid player config for {}", config_key);
                    *value = EspConfig::Player(EspPlayerSettings::new(&target));
                    if let EspConfig::Player(value) = value {
                        value
                    } else {
                        unreachable!()
                    }
                }
            }
            Entry::Vacant(entry) => {
                if let EspConfig::Player(value) =
                    entry.insert(EspConfig::Player(EspPlayerSettings::new(&target)))
                {
                    value
                } else {
                    unreachable!()
                }
            }
        };
        let _ui_enable_token = ui.begin_enabled(config_enabled);

        let content_height =
            ui.content_region_avail()[1] - ui.text_line_height_with_spacing() * 2.0 - 16.0;
        unsafe {
            imgui::sys::igSetNextItemOpen(
                matches!(
                    self.esp_player_active_header,
                    EspPlayerActiveHeader::Features
                ),
                0,
            );
        };
        if ui.collapsing_header("Features", TreeNodeFlags::empty()) {
            self.esp_player_active_header = EspPlayerActiveHeader::Features;
            if let Some(_token) = {
                ui.child_window("features")
                    .size([0.0, content_height])
                    .begin()
            } {
                ui.indent_by(5.0);
                ui.dummy([0.0, 5.0]);

                const COMBO_WIDTH: f32 = 150.0;
                {
                    const ESP_BOX_TYPES: [(EspBoxType, &'static str); 3] = [
                        (EspBoxType::None, "No"),
                        (EspBoxType::Box2D, "2D"),
                        (EspBoxType::Box3D, "3D"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(obfstr!("player box"), &ESP_BOX_TYPES, &mut config.box_type);
                }

                {
                    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
                    enum PlayerSkeletonType {
                        None,
                        Skeleton,
                    }

                    const PLAYER_SKELETON_TYPES: [(PlayerSkeletonType, &'static str); 2] = [
                        (PlayerSkeletonType::None, "No"),
                        (PlayerSkeletonType::Skeleton, "Show"),
                    ];

                    let mut skeleton_type = if config.skeleton {
                        PlayerSkeletonType::Skeleton
                    } else {
                        PlayerSkeletonType::None
                    };

                    ui.set_next_item_width(COMBO_WIDTH);
                    let value_changed = ui.combo_enum(
                        obfstr!("player skeleton"),
                        &PLAYER_SKELETON_TYPES,
                        &mut skeleton_type,
                    );

                    if value_changed {
                        config.skeleton = matches!(skeleton_type, PlayerSkeletonType::Skeleton);
                    }
                }

                {
                    const HEAD_DOT_TYPES: [(EspHeadDot, &'static str); 3] = [
                        (EspHeadDot::None, "No"),
                        (EspHeadDot::Filled, "Filled"),
                        (EspHeadDot::NotFilled, "Not Filled"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(obfstr!("head dot"), &HEAD_DOT_TYPES, &mut config.head_dot);
                }

                {
                    const TRACER_LINE_TYPES: [(EspTracePosition, &'static str); 7] = [
                        (EspTracePosition::None, "No"),
                        (EspTracePosition::TopLeft, "Top left"),
                        (EspTracePosition::TopCenter, "Top (center)"),
                        (EspTracePosition::TopRight, "Top right"),
                        (EspTracePosition::BottomLeft, "Bottom left"),
                        (EspTracePosition::BottomCenter, "Bottom (center)"),
                        (EspTracePosition::BottomRight, "Bottom right"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(
                        obfstr!("tracer lines"),
                        &TRACER_LINE_TYPES,
                        &mut config.tracer_lines,
                    );
                }

                {
                    const HEALTH_BAR_TYPES: [(EspHealthBar, &'static str); 5] = [
                        (EspHealthBar::None, "No"),
                        (EspHealthBar::Top, "Top"),
                        (EspHealthBar::Left, "Left"),
                        (EspHealthBar::Bottom, "Bottom"),
                        (EspHealthBar::Right, "Right"),
                    ];

                    ui.set_next_item_width(COMBO_WIDTH);
                    ui.combo_enum(
                        obfstr!("player health bar"),
                        &HEALTH_BAR_TYPES,
                        &mut config.health_bar,
                    );
                }
                ui.dummy([0.0, 10.0]);                ui.text("Player Info");
                let neuomorphic_style = crate::settings::get_neuomorphic_style();
                ui.neuomorphic_checkbox(obfstr!("Name"), &mut config.info_name, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Weapon"), &mut config.info_weapon, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Distance"), &mut config.info_distance, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Health"), &mut config.info_hp_text, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Kit"), &mut config.info_flag_kit, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Flashed"), &mut config.info_flag_flashed, &neuomorphic_style);
                ui.neuomorphic_checkbox(obfstr!("Near only"), &mut config.near_players, &neuomorphic_style);
                if config.near_players {
                    ui.same_line();
                    ui.neuomorphic_slider_f32("Max distance", &mut config.near_players_distance, 0.0, 50.0, "%.1f", &neuomorphic_style);
                }
            }
        }

        unsafe {
            imgui::sys::igSetNextItemOpen(
                matches!(self.esp_player_active_header, EspPlayerActiveHeader::Style),
                0,
            );
        };
        if ui.collapsing_header("Style & Colors", TreeNodeFlags::empty()) {
            self.esp_player_active_header = EspPlayerActiveHeader::Style;
            if let Some(_token) = {
                ui.child_window("styles")
                    .size([0.0, content_height])
                    .begin()
            } {
                ui.indent_by(5.0);
                ui.dummy([0.0, 5.0]);

                if let Some(_token) = {
                    let mut column_type = TableColumnSetup::new("Type");
                    column_type.init_width_or_weight = 130.0;
                    column_type.flags = TableColumnFlags::WIDTH_FIXED;

                    let mut column_value = TableColumnSetup::new("Value");
                    column_value.init_width_or_weight = 160.0;
                    column_value.flags = TableColumnFlags::WIDTH_FIXED;

                    ui.begin_table_header_with_flags(
                        "styles_table",
                        [TableColumnSetup::new("Name"), column_type, column_value],
                        TableFlags::ROW_BG
                            | TableFlags::BORDERS
                            | TableFlags::SIZING_STRETCH_PROP
                            | TableFlags::SCROLL_Y,
                    )
                } {
                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("ESP box color"),
                        &mut config.box_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("ESP box width"),
                        1.0,
                        10.0,
                        &mut config.box_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Player skeleton color"),
                        &mut config.skeleton_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Player skeleton width"),
                        1.0,
                        10.0,
                        &mut config.skeleton_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Head dot color"),
                        &mut config.head_dot_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Head dot thickness"),
                        1.0,
                        5.0,
                        &mut config.head_dot_thickness,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Head dot radius"),
                        0.0,
                        10.0,
                        &mut config.head_dot_base_radius,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Head dot z offset"),
                        0.0,
                        10.0,
                        &mut config.head_dot_z,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Health bar width"),
                        5.0,
                        30.0,
                        &mut config.health_bar_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Tracer line color"),
                        &mut config.tracer_lines_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_width(
                        ui,
                        obfstr!("Tracer line width"),
                        1.0,
                        10.0,
                        &mut config.tracer_lines_width,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info name"),
                        &mut config.info_name_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info distance"),
                        &mut config.info_distance_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info weapon"),
                        &mut config.info_weapon_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info health"),
                        &mut config.info_hp_text_color,
                    );

                    ui.table_next_row();
                    Self::render_esp_settings_player_style_color(
                        ui,
                        obfstr!("Color info player flags"),
                        &mut config.info_flags_color,
                    );
                }
            }
        }

        drop(_ui_enable_token);
    }

    fn render_esp_settings_player_style_width(
        ui: &imgui::Ui,
        label: &str,
        min: f32,
        max: f32,
        value: &mut f32,
    ) -> bool {
        ui.table_next_column();
        ui.text(label);

        ui.table_next_column();
        ui.text(&format!("{:.2} - {:.2}", min, max));        ui.table_next_column();
        let neuomorphic_style = crate::settings::get_neuomorphic_style();
        use crate::utils::neuomorphic::NeuomorphicUi;        if ui.neuomorphic_input_float(&format!("##{}_style_width", ui.table_row_index()), value, &neuomorphic_style) {
            *value = value.clamp(min, max);
            true
        } else {
            false
        }
    }

    fn render_esp_settings_player_style_color(ui: &imgui::Ui, label: &str, color: &mut EspColor) {
        ui.table_next_column();
        ui.text(label);

        ui.table_next_column();
        {
            let mut color_type = EspColorType::from_esp_color(color);
            ui.set_next_item_width(ui.content_region_avail()[0]);
            let color_type_changed = ui.combo_enum(
                &format!("##{}_color_type", ui.table_row_index()),
                &[
                    (EspColorType::Static, "Static"),
                    (EspColorType::HealthBased, "Health based"),
                    (EspColorType::HealthBasedRainbow, "Rainbow"),
                    (EspColorType::DistanceBased, "Distance"),
                ],
                &mut color_type,
            );

            if color_type_changed {
                *color = match color_type {
                    EspColorType::Static => EspColor::Static {
                        value: Color::from_f32([1.0, 1.0, 1.0, 1.0]),
                    },
                    EspColorType::HealthBased => EspColor::HealthBased {
                        max: Color::from_f32([0.0, 1.0, 0.0, 1.0]),
                        mid: Color::from_f32([1.0, 1.0, 0.0, 1.0]),
                        min: Color::from_f32([1.0, 0.0, 0.0, 1.0]),
                    },
                    EspColorType::HealthBasedRainbow => EspColor::HealthBasedRainbow { alpha: 1.0 },
                    EspColorType::DistanceBased => EspColor::DistanceBased {
                        near: Color::from_f32([1.0, 0.0, 0.0, 1.0]),
                        mid: Color::from_f32([1.0, 1.0, 0.0, 1.0]),
                        far: Color::from_f32([0.0, 1.0, 0.0, 1.0]),
                    },
                }
            }
        }

        ui.table_next_column();
        {
            match color {                EspColor::HealthBasedRainbow { alpha } => {
                    ui.text("Alpha:");
                    ui.same_line();
                    ui.set_next_item_width(100.0);
                    
                    // Use RAGNAREK style slider for alpha
                    use crate::utils::ragnarek::RagnarekUi;
                    if ui.ragnarek_slider_float("##rainbow_alpha", alpha, 0.1, 1.0) {
                        // Alpha value is updated automatically via mutable reference
                    }
                }
                EspColor::Static { value } => {
                    let mut color_value = value.as_f32();
                    
                    // Use RAGNAREK color picker for static colors
                    use crate::utils::ragnarek::RagnarekUi;
                    if ui.ragnarek_color_picker(&format!("##{}_static_value", ui.table_row_index()), &mut color_value) {
                        *value = Color::from_f32(color_value);
                    }
                }
                EspColor::HealthBased { max, mid, min } => {
                    let mut max_value = max.as_f32();
                    if ui.color_edit4(&format!("##{}_health_max", ui.table_row_index()), &mut max_value) {
                        *max = Color::from_f32(max_value);
                    }
                    ui.same_line();
                    ui.text(" => ");
                    ui.same_line();
                    let mut mid_value = mid.as_f32();
                    if ui.color_edit4(&format!("##{}_health_mid", ui.table_row_index()), &mut mid_value) {
                        *mid = Color::from_f32(mid_value);
                    }
                    ui.same_line();
                    ui.text(" => ");
                    ui.same_line();
                    let mut min_value = min.as_f32();
                    if ui.color_edit4(&format!("##{}_health_min", ui.table_row_index()), &mut min_value) {
                        *min = Color::from_f32(min_value);
                    }
                }
                EspColor::DistanceBased { near, mid, far } => {
                    let mut near_color = near.as_f32();
                    if ui.color_edit4(&format!("##{}_near", ui.table_row_index()), &mut near_color) {
                        *near = Color::from_f32(near_color);
                    }
                    ui.same_line();
                    ui.text(" => ");
                    ui.same_line();
                    let mut mid_color = mid.as_f32();
                    if ui.color_edit4(&format!("##{}_mid", ui.table_row_index()), &mut mid_color) {
                        *mid = Color::from_f32(mid_color);
                    }
                    ui.same_line();
                    ui.text(" => ");
                    ui.same_line();
                    let mut far_color = far.as_f32();
                    if ui.color_edit4(&format!("##{}_far", ui.table_row_index()), &mut far_color) {
                        *far = Color::from_f32(far_color);
                    }
                }
            }
        }
    }


    fn render_esp_settings_chicken(
        &mut self,
        _settings: &mut AppSettings,
        ui: &imgui::Ui,
        _target: EspSelector,
    ) {
        ui.text("Chicken!");
    }

    fn render_esp_settings_weapon(
        &mut self,
        _settings: &mut AppSettings,
        ui: &imgui::Ui,
        _target: EspSelector,
    ) {
        ui.text("Weapon!");
    }

    fn render_esp_settings(&mut self, settings: &mut AppSettings, ui: &imgui::Ui) {
        if let Some(target) = self.esp_pending_target.take() {
            self.esp_selected_target = target;
        }

        /* the left tree */
        let content_region = ui.content_region_avail();
        let original_style = ui.clone_style();
        let tree_width = (content_region[0] * 0.25).max(150.0);
        let content_width = (content_region[0] - tree_width - 5.0).max(300.0);

        ui.text("ESP Target");
        ui.same_line_with_pos(
            original_style.window_padding[0] * 2.0 + tree_width + original_style.window_border_size,
        );
        if !matches!(self.esp_selected_target, EspSelector::None) {
            let target_key = self.esp_selected_target.config_key();
            let target_enabled = settings
                .esp_settings_enabled
                .entry(target_key.to_string())
                .or_insert(false);            let neuomorphic_style = crate::settings::get_neuomorphic_style();
            ui.neuomorphic_checkbox(self.esp_selected_target.config_title().as_str(), target_enabled, &neuomorphic_style);

            let reset_text = "Reset config";
            let reset_text_width = ui.calc_text_size(&reset_text)[0];

            let total_width = ui.content_region_avail()[0] + 2.0;
            ui.same_line_with_pos(total_width - reset_text_width);

            let _enabled = ui.begin_enabled(*target_enabled);
            if ui.neuomorphic_button(reset_text, [0.0, 0.0], &neuomorphic_style) {
                /* just removing the key will work as a default config will be emplaced later */
                settings.esp_settings.remove(&target_key);
            }
        } else {
            ui.text("Target Configuration");
        };

        //ui.dummy([0.0, 10.0]);

        if let (Some(_token), _padding) = {
            let padding = ui.push_style_var(StyleVar::WindowPadding([
                0.0,
                original_style.window_padding[1],
            ]));
            let window = ui
                .child_window("ESP Target")
                .size([tree_width, 0.0])
                .border(true)
                .draw_background(true)
                .scroll_bar(true)
                .begin();

            (window, padding)
        } {
            ui.indent_by(
                original_style.window_padding[0] +
                    /* for the indicator */
                    ui.current_font_size() * 0.5 + 4.0,
            );

            self.render_esp_target(settings, ui, &EspSelector::Player);
            // self.render_esp_target(settings, ui, &EspSelector::Chicken);
            // self.render_esp_target(settings, ui, &EspSelector::Weapon)
        }
        ui.same_line();
        if let Some(_token) = {
            ui.child_window("Content")
                .size([content_width, 0.0])
                .scroll_bar(true)
                .begin()
        } {
            match &self.esp_selected_target {
                EspSelector::None => {}
                EspSelector::Player
                | EspSelector::PlayerTeam { .. }
                | EspSelector::PlayerTeamVisibility { .. } => {
                    self.render_esp_settings_player(settings, ui, self.esp_selected_target.clone())
                }
                EspSelector::Chicken => {
                    self.render_esp_settings_chicken(settings, ui, self.esp_selected_target.clone())
                }
                EspSelector::Weapon
                | EspSelector::WeaponGroup { .. }
                | EspSelector::WeaponSingle { .. } => {
                    self.render_esp_settings_weapon(settings, ui, self.esp_selected_target.clone())
                }
            }
        }
    }

    fn render_grenade_target(
        &mut self,
        settings: &mut GrenadeSettings,
        ui: &imgui::Ui,
        target: &GrenadeSettingsTarget,
    ) {
        let ident = ui.clone_style().indent_spacing * target.ident_level() as f32;
        if ident > 0.0 {
            ui.indent_by(ident);
        }

        let item_text = match target {
            GrenadeSettingsTarget::General => "Settings".to_string(),
            GrenadeSettingsTarget::MapType(value) => value.clone(),
            GrenadeSettingsTarget::Map {
                map_name,
                display_name,
            } => {
                let location_count = settings.map_spots.get(map_name).map(Vec::len).unwrap_or(0);
                format!(
                    "{} ({}) ##{}",
                    display_name,
                    location_count,
                    target.ui_token()
                )
            }
        };

        let clicked = ui
            .selectable_config(item_text)
            .selected(target == &self.grenade_helper_target)
            .flags(SelectableFlags::SPAN_ALL_COLUMNS)
            .build();

        if clicked && !matches!(target, GrenadeSettingsTarget::MapType(_)) {
            self.grenade_helper_pending_target = Some(target.clone());
        }

        if ident > 0.0 {
            ui.unindent_by(ident);
        }
    }

    fn render_grenade_helper(
        &mut self,
        states: &StateRegistry,
        settings: &mut GrenadeSettings,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) {
        if let Some(target) = self.grenade_helper_pending_target.take() {
            self.grenade_helper_target = target;
            self.grenade_helper_selected_id = 0;
            self.grenade_helper_new_item = None;
        }

        if let Some(target) = self.grenade_helper_pending_selected_id.take() {
            self.grenade_helper_selected_id = target;
            self.grenade_helper_new_item = None;
        }

        /* the left tree */
        let content_region = ui.content_region_avail();
        let original_style = ui.clone_style();
        let tree_width = (content_region[0] * 0.25).max(150.0);
        let content_width = content_region[0] - tree_width - 5.0;

        ui.text("Grenade Helper");

        ui.same_line_with_pos(
            original_style.window_padding[0] * 2.0 + tree_width + original_style.window_border_size,
        );
        ui.text("");

        {
            let text_import = "Import";
            let text_import_width = ui.calc_text_size(&text_import)[0];

            let text_export = "Export";
            let text_export_width = ui.calc_text_size(&text_export)[0];

            let total_width = ui.content_region_avail()[0] + 2.0;

            let mut grenade_helper_transfer_state =
                self.grenade_helper_transfer_state.lock().unwrap();
            let _buttons_disabled = ui.begin_disabled(!matches!(
                &*grenade_helper_transfer_state,
                GrenadeHelperTransferState::Idle
            ));
            ui.same_line_with_pos(
                total_width
                    - text_export_width
                    - original_style.frame_padding[0] * 2.0                    - text_import_width
                    - original_style.frame_padding[0] * 2.0,
            );
            let neuomorphic_style = crate::settings::get_neuomorphic_style();
            if ui.neuomorphic_button(text_export, [0.0, 0.0], &neuomorphic_style) {
                *grenade_helper_transfer_state = GrenadeHelperTransferState::Pending {
                    direction: GrenadeHelperTransferDirection::Export,
                };
            }

            ui.same_line_with_pos(total_width - text_import_width);
            if ui.neuomorphic_button(text_import, [0.0, 0.0], &neuomorphic_style) {
                *grenade_helper_transfer_state = GrenadeHelperTransferState::Pending {
                    direction: GrenadeHelperTransferDirection::Import,
                };
            }
        }

        //ui.dummy([0.0, 10.0]);

        if let (Some(_token), _padding) = {
            let padding = ui.push_style_var(StyleVar::WindowPadding([
                0.0,
                original_style.window_padding[1],
            ]));
            let window = ui
                .child_window("Helper Target")
                .size([tree_width, 0.0])
                .border(true)
                .draw_background(true)
                .scroll_bar(true)
                .begin();

            (window, padding)
        } {
            ui.indent_by(original_style.window_padding[0] + 4.0);

            for target in [
                GrenadeSettingsTarget::General,
                GrenadeSettingsTarget::MapType("Competitive Maps".to_owned()),
                GrenadeSettingsTarget::Map {
                    map_name: "de_ancient".to_owned(),
                    display_name: "Ancient".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_anubis".to_owned(),
                    display_name: "Anubis".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_dust2".to_owned(),
                    display_name: "Dust 2".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_inferno".to_owned(),
                    display_name: "Inferno".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_mills".to_owned(),
                    display_name: "Mills".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_mirage".to_owned(),
                    display_name: "Mirage".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_nuke".to_owned(),
                    display_name: "Nuke".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "cs_office".to_owned(),
                    display_name: "Office".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_overpass".to_owned(),
                    display_name: "Overpass".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_thera".to_owned(),
                    display_name: "Thera".to_owned(),
                },
                GrenadeSettingsTarget::Map {
                    map_name: "de_vertigo".to_owned(),
                    display_name: "Vertigo".to_owned(),
                },
            ] {
                self.render_grenade_target(settings, ui, &target);
            }
        }
        ui.same_line();
        if let Some(_token) = {
            ui.child_window("Content")
                .size([content_width, 0.0])
                .scroll_bar(true)
                .begin()
        } {
            match &self.grenade_helper_target {
                GrenadeSettingsTarget::General => {
                    self.render_grenade_helper_target_settings(states, settings, ui);
                }
                GrenadeSettingsTarget::MapType(_) => { /* Nothing to render */ }
                GrenadeSettingsTarget::Map { map_name, .. } => {
                    self.render_grenade_helper_target_map(
                        states,
                        settings,
                        ui,
                        &map_name.clone(),
                        unicode_text,
                    );
                }
            }
        }
    }

    fn render_grenade_helper_target_map(
        &mut self,
        states: &StateRegistry,
        settings: &mut GrenadeSettings,
        ui: &imgui::Ui,
        map_name: &str,
        unicode_text: &UnicodeTextRenderer,
    ) {
        /* the left tree */
        let content_region = ui.content_region_avail();
        let original_style = ui.clone_style();
        let tree_width = (content_region[0] * 0.25).max(150.0);
        let content_width = content_region[0] - tree_width - original_style.item_spacing[0];

        /* The list itself */
        {
            ui.text("Available spots");
            let text_width = ui.calc_text_size("Available spots")[0];
            let button_width = tree_width - text_width - original_style.item_spacing[0];

            ui.same_line();            ui.set_next_item_width(button_width);
            let neuomorphic_style = crate::settings::get_neuomorphic_style();
            use crate::utils::neuomorphic::NeuomorphicUi;
            ui.neuomorphic_combo_enum(
                "##sort_type",
                &[
                    (GrenadeSortOrder::Alphabetical, "A-z"),
                    (GrenadeSortOrder::AlphabeticalReverse, "Z-a"),
                ],
                &mut settings.ui_sort_order,
                &neuomorphic_style
            );

            if let (Some(_token), _padding) = {
                let padding = ui.push_style_var(StyleVar::WindowPadding([
                    0.0,
                    original_style.window_padding[1],
                ]));
                let window = ui
                    .child_window("Map Target")
                    .size([
                        tree_width,
                        content_region[1]
                            - ui.text_line_height_with_spacing() * 2.0
                            - original_style.frame_padding[1] * 4.0,
                    ])
                    .border(true)
                    .draw_background(true)
                    .scroll_bar(true)
                    .begin();

                (window, padding)
            } {
                ui.indent_by(original_style.window_padding[0]);

                if let Some(grenades) = settings.map_spots.get(map_name) {
                    let mut sorted_grenades = grenades.iter().collect::<Vec<_>>();
                    settings.ui_sort_order.sort(&mut sorted_grenades);

                    for grenade in sorted_grenades.iter() {
                        let grenade_types = grenade
                            .grenade_types
                            .iter()
                            .map(GrenadeType::display_name)
                            .collect::<Vec<_>>()
                            .join(", ");

                        let clicked = ui
                            .selectable_config(format!(
                                "{} ({}) ##{}",
                                grenade.name, grenade_types, grenade.id
                            ))
                            .selected(grenade.id == self.grenade_helper_selected_id)
                            .flags(SelectableFlags::SPAN_ALL_COLUMNS)
                            .build();
                        unicode_text.register_unicode_text(&grenade.name);

                        if clicked {
                            self.grenade_helper_pending_selected_id = Some(grenade.id);
                        }
                    }
                }
            }

            /* Add / delete buttons */
            {
                let mut delete_current_grenade = false;
                let current_grenade_position = settings
                    .map_spots
                    .get(map_name)
                    .map(|spots| {
                        spots
                            .iter()
                            .position(|spot| spot.id == self.grenade_helper_selected_id)
                    })
                    .flatten();

                let button_width = (tree_width - original_style.item_spacing[0]) / 2.0;
                ui.set_cursor_pos([
                    0.0,
                    content_region[1]
                        - ui.text_line_height()
                        - original_style.frame_padding[1] * 2.0,
                ]);                let neuomorphic_style = crate::settings::get_neuomorphic_style();
                if ui.neuomorphic_button("New", [button_width, 0.0], &neuomorphic_style) {
                    self.grenade_helper_new_item = Some(Default::default());
                    self.grenade_helper_selected_id = 0;
                }

                let _button_disabled = ui.begin_disabled(current_grenade_position.is_none());
                ui.same_line();
                if ui.neuomorphic_button("Delete", [button_width, 0.0], &neuomorphic_style) {
                    if self.grenade_helper_skip_confirmation_dialog {
                        delete_current_grenade = true;
                    } else {
                        ui.open_popup("Delete item? ##delete_grenade_helper_spot");
                    }
                }

                if let Some(_token) = ui
                    .modal_popup_config("Delete item? ##delete_grenade_helper_spot")
                    .resizable(false)
                    .movable(false)
                    .always_auto_resize(true)
                    .begin_popup()
                {
                    ui.text("Are you sure you want to delete this item?");
                    ui.spacing();
                    ui.separator();

                    ui.spacing();
                                       let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Yes", [100.0, 0.0], &neuomorphic_style) {
                        ui.close_current_popup();
                        delete_current_grenade = true;
                    }

                    ui.same_line();
                    if ui.neuomorphic_button("No", [100.0, 0.0], &neuomorphic_style) {
                        ui.close_current_popup();
                    }
                }

                if delete_current_grenade {
                    if let Some(grenades) = settings.map_spots.get_mut(map_name) {
                        grenades.remove(current_grenade_position.unwrap());
                    }
                }
            }
        }

        /* grenade info */
        ui.set_cursor_pos([tree_width + original_style.item_spacing[0], 0.0]);
        if let Some(_token) = {
            ui.child_window("Content")
                .size([content_width, 0.0])
                .scroll_bar(true)
                .begin()
        } {
            if let Some(current_grenade) = {
                settings
                    .map_spots
                    .get_mut(map_name)
                    .map(|spots| {
                        spots
                            .iter_mut()
                            .find(|spot| spot.id == self.grenade_helper_selected_id)
                    })
                    .flatten()
                    .or(self.grenade_helper_new_item.as_mut())
            } {
                let mut assign_current_position = false;
                let _full_width = ui.push_item_width(-1.0);

                if current_grenade.id > 0 {
                    ui.text("Grenade Info");
                } else {
                    ui.text("Add a new grenade spot");
                }                ui.text("Name");
                let neuomorphic_style = crate::settings::get_neuomorphic_style();
                use crate::utils::neuomorphic::NeuomorphicUi;
        ui.neuomorphic_input_text("##grenade_helper_spot_name", &mut current_grenade.name, &neuomorphic_style);
                unicode_text.register_unicode_text(&current_grenade.name);                ui.text("Description");
                // We need to add a neuomorphic styling for multiline text inputs
                ui.push_style_color(StyleColor::FrameBg, to_f32_color(neuomorphic_style.palette.surface_depressed));
                ui.push_style_color(StyleColor::FrameBgHovered, to_f32_color(neuomorphic_style.palette.surface));
                ui.push_style_color(StyleColor::FrameBgActive, to_f32_color(neuomorphic_style.palette.surface_elevated));
                ui.push_style_color(StyleColor::Text, to_f32_color(neuomorphic_style.palette.text_primary));
                ui.push_style_var(StyleVar::FrameRounding(neuomorphic_style.border_radius));
                ui.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
                  ui.input_text_multiline(
                    "##grenade_helper_spot_description",
                    &mut current_grenade.description,
                    [0.0, 100.0]);
                  unicode_text.register_unicode_text(&current_grenade.description);ui.text("Eye position");
                // Apply neuomorphic styling to the float3 input
                ui.push_style_color(StyleColor::FrameBg, to_f32_color(neuomorphic_style.palette.surface_depressed));
                ui.push_style_color(StyleColor::FrameBgHovered, to_f32_color(neuomorphic_style.palette.surface));
                ui.push_style_color(StyleColor::FrameBgActive, to_f32_color(neuomorphic_style.palette.surface_elevated));
                ui.push_style_color(StyleColor::Text, to_f32_color(neuomorphic_style.palette.text_primary));
                ui.push_style_var(StyleVar::FrameRounding(neuomorphic_style.border_radius));
                ui.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
                  ui.input_float3(
                    "##grenade_helper_spot_eye_position",
                    &mut current_grenade.eye_position);                
                ui.text("Pitch/Yaw");
                // Apply neuomorphic styling to the float2 input
                ui.push_style_color(StyleColor::FrameBg, to_f32_color(neuomorphic_style.palette.surface_depressed));
                ui.push_style_color(StyleColor::FrameBgHovered, to_f32_color(neuomorphic_style.palette.surface));
                ui.push_style_color(StyleColor::FrameBgActive, to_f32_color(neuomorphic_style.palette.surface_elevated));
                ui.push_style_color(StyleColor::Text, to_f32_color(neuomorphic_style.palette.text_primary));
                ui.push_style_var(StyleVar::FrameRounding(neuomorphic_style.border_radius));
                ui.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
                  ui.input_float2(
                    "##grenade_helper_spot_ptch_yaw",
                    &mut current_grenade.eye_direction);
                
                let current_map = states
                    .get::<StateCurrentMap>(())
                    .map(|value| value.current_map.clone())
                    .flatten();

                let current_player_position = states
                    .resolve::<StateGrenadeHelperPlayerLocation>(())
                    .map(|value| {
                        if let StateGrenadeHelperPlayerLocation::Valid {
                            eye_position,
                            eye_direction,
                        } = *value
                        {
                            Some((eye_position, eye_direction))
                        } else {
                            None
                        }
                    });

                {                    let button_enabled =
                        current_player_position.as_ref().unwrap_or(&None).is_some();
                    let _enabled_token = ui.begin_enabled(button_enabled);
                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Use current", [0.0, 0.0], &neuomorphic_style) {
                        if current_map
                            .as_ref()
                            .map(|current_map| current_map == map_name)
                            .unwrap_or(false)
                        {
                            assign_current_position = true;
                        } else {
                            /* Map differs */
                            ui.open_popup(
                                "Are you sure?##grenade_helper_use_current_map_different",
                            );
                        }
                    }

                    if ui.is_item_hovered() {
                        match &current_player_position {
                            Ok(Some(_)) => {
                                ui.tooltip_text("Copy your current location and direction")
                            }
                            Ok(None) => ui.tooltip_text("You don't have a valid player position"),
                            Err(err) => ui.tooltip_text(format!("Error: {:#}", err)),
                        }
                    }
                }

                if let Some(_token) = ui
                    .modal_popup_config("Are you sure?##grenade_helper_use_current_map_different")
                    .resizable(false)
                    .always_auto_resize(true)
                    .begin_popup()
                {
                    ui.text("The current map does not match the selected map.");
                    ui.text(format!("Selected map: {}", map_name));
                    ui.text(format!(
                        "Current map: {}",
                        current_map
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or("unknown")
                    ));
                    ui.new_line();
                    ui.text("Do you want to copy the location anyways?");

                    ui.spacing();
                    ui.separator();
                    ui.spacing();                    let button_width =
                        (ui.content_region_avail()[0] - original_style.item_spacing[0]) / 2.0;
                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Yes", [button_width, 0.0], &neuomorphic_style) {
                        ui.close_current_popup();
                        assign_current_position = true;
                    }

                    ui.same_line();
                    if ui.neuomorphic_button("No", [button_width, 0.0], &neuomorphic_style) {
                        ui.close_current_popup();
                    }
                }

                if assign_current_position {
                    if let Some((eye_position, eye_direction)) =
                        current_player_position.ok().flatten()
                    {
                        current_grenade.eye_position = eye_position.as_slice().try_into().unwrap();
                        current_grenade.eye_direction =
                            eye_direction.as_slice().try_into().unwrap();
                    }
                }

                for grenade_type in [
                    GrenadeType::Smoke,
                    GrenadeType::Flashbang,
                    GrenadeType::Explosive,
                    GrenadeType::Molotov,
                ] {
                    let current_position = current_grenade
                        .grenade_types
                        .iter()
                        .position(|value| *value == grenade_type);

                    let mut enabled = current_position.is_some();
                    if ui.checkbox(grenade_type.display_name(), &mut enabled) {
                        if let Some(current_position) = current_position {
                            current_grenade.grenade_types.remove(current_position);
                        } else {
                            current_grenade.grenade_types.push(grenade_type);
                        }
                    }
                }

                if current_grenade.id == 0 {                    let region_avail = ui.content_region_max();
                    ui.set_cursor_pos([region_avail[0] - 100.0, ui.cursor_pos()[1]]);
                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Create", [100.0, 0.0], &neuomorphic_style) {
                        if let Some(mut grenade) = self.grenade_helper_new_item.take() {
                            let grenades =
                                settings.map_spots.entry(map_name.to_string()).or_default();

                            grenade.id = GrenadeSpotInfo::new_id();
                            self.grenade_helper_pending_selected_id = Some(grenade.id);

                            grenades.push(grenade);
                        }
                    }
                }
            } else {
                let text = "Please select an item";
                let text_bounds = ui.calc_text_size(text);
                let region_avail = ui.content_region_avail();

                ui.set_cursor_pos([
                    (region_avail[0] - text_bounds[0]) / 2.0,
                    (region_avail[1] - text_bounds[1]) / 2.0,
                ]);

                ui.text_colored(
                    ui.style_color(StyleColor::TextDisabled),
                    "Please select a grenade",
                );
            }
        }
    }

    fn render_grenade_helper_target_settings(
        &mut self,
        _states: &StateRegistry,
        settings: &mut GrenadeSettings,
        ui: &imgui::Ui,
    ) {    fn render_color(ui: &imgui::Ui, label: &str, value: &mut Color) {
            let mut color_value = value.as_f32();
            
            let neuomorphic_style = crate::settings::get_neuomorphic_style();
            use crate::utils::neuomorphic::NeuomorphicUi;
            
            if ui.neuomorphic_color_edit4(label, &mut color_value, &neuomorphic_style) {
                *value = Color::from_f32(color_value);
            }
        }

        ui.text("UI Settings");
        ui.spacing();        let neuomorphic_style = crate::settings::get_neuomorphic_style();
        use crate::utils::neuomorphic::NeuomorphicUi;        ui.neuomorphic_input_float("Circle distance", &mut settings.circle_distance, &neuomorphic_style);        ui.neuomorphic_input_float("Circle radius", &mut settings.circle_radius, &neuomorphic_style);
            
        // Apply neuomorphic styling to the scalar input
        ui.push_style_color(StyleColor::FrameBg, to_f32_color(neuomorphic_style.palette.surface_depressed));
        ui.push_style_color(StyleColor::FrameBgHovered, to_f32_color(neuomorphic_style.palette.surface));
        ui.push_style_color(StyleColor::FrameBgActive, to_f32_color(neuomorphic_style.palette.surface_elevated));
        ui.push_style_color(StyleColor::Text, to_f32_color(neuomorphic_style.palette.text_primary));
        ui.push_style_var(StyleVar::FrameRounding(neuomorphic_style.border_radius));
        ui.push_style_var(StyleVar::FramePadding([10.0, 8.0]));          ui.input_scalar("Circle segments", &mut settings.circle_segments);
            
        ui.neuomorphic_input_float("Angle threshold yaw", &mut settings.angle_threshold_yaw, &neuomorphic_style);ui.neuomorphic_input_float("Angle threshold pitch", &mut settings.angle_threshold_pitch, &neuomorphic_style);

        render_color(ui, "Color position", &mut settings.color_position);
        render_color(
            ui,
            "Color position (active)",
            &mut settings.color_position_active,
        );
        render_color(ui, "Color angle", &mut settings.color_angle);
        render_color(
            ui,
            "Color angle  (active)",
            &mut settings.color_angle_active,
        );

        ui.checkbox(
            obfstr!("ViewBox Background"),
            &mut settings.grenade_background,
        );
    }

    fn render_grenade_helper_transfer(&mut self, settings: &mut GrenadeSettings, ui: &imgui::Ui) {
        let mut transfer_state = self.grenade_helper_transfer_state.lock().unwrap();
        match &*transfer_state {
            GrenadeHelperTransferState::Idle => return,

            GrenadeHelperTransferState::Pending { direction } => {
                let executor: Box<











                    dyn FnOnce() -> anyhow::Result<GrenadeHelperTransferState> + Send,
                > = match direction {
                    GrenadeHelperTransferDirection::Export => {
                        let items = settings.map_spots.clone();
                        Box::new(move || {
                            // GrenadeHelperTransferState
                            let Some(target_path) = rfd::FileDialog::new()
                                .add_filter("Valthrun Grenade Spots", &["vgs"])
                                .save_file()
                            else {
                                return Ok(GrenadeHelperTransferState::Idle);
                            };

                            let items = serde_json::to_string(&items)?;
                            let mut output = File::options()
                                .create(true)
                                .truncate(true)
                                .write(true)
                                .open(&target_path)
                                .context("open destination")?;
                            output.write_all(items.as_bytes()).context("write")?;

                            Ok(GrenadeHelperTransferState::ExportSuccess { target_path })
                        })
                    }
                    GrenadeHelperTransferDirection::Import => {
                        Box::new(move || {
                            // GrenadeHelperTransferState
                            let Some(target_path) = rfd::FileDialog::new()
                                .add_filter("Valthrun Grenade Spots", &["vgs"])
                                .pick_file()
                            else {
                                return Ok(GrenadeHelperTransferState::Idle);
                            };

                            let input = File::options()
                                .read(true)
                                .open(target_path)
                                .context("open file")?;

                            let elements = serde_json::from_reader(&mut BufReader::new(input))
                                .context("parse file")?;

                            Ok(GrenadeHelperTransferState::ImportPending { elements })
                        })
                    }
                };

                thread::spawn({
                    let direction = direction.clone();
                    let grenade_helper_transfer_state = self.grenade_helper_transfer_state.clone();
                    move || {
                        let result = executor();
                        let mut transfer_state = grenade_helper_transfer_state.lock().unwrap();
                        match result {
                            Ok(new_state) => {
                                *transfer_state = new_state;
                            }
                            Err(err) => {
                                *transfer_state = GrenadeHelperTransferState::Failed {
                                    direction,
                                    message: format!("{:#}", err),
                                };
                            }
                        }
                    }
                });
                *transfer_state = GrenadeHelperTransferState::Active {
                    direction: direction.clone(),
                };
            }
            GrenadeHelperTransferState::Active { .. } => {
                /* Just waiting for the file picker to finish. */
            }

            GrenadeHelperTransferState::ImportPending { elements } => {
                let mut popup_open = true;
                if let Some(_popup) = ui
                    .modal_popup_config("Data Import")
                    .always_auto_resize(true)
                    .opened(&mut popup_open)
                    .begin_popup()
                {
                    let total_count = elements.values().map(|spots| spots.len()).sum::<usize>();

                    ui.text(format!(
                        "The following locations have been loaded ({})",
                        total_count
                    ));
                    for (map_name, spots) in elements.iter() {
                        ui.text(format!("- {} ({} spots)", map_name, spots.len()));
                    }

                    ui.new_line();
                    ui.text("Would you like to replace the current configuration?");

                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    let button_width =
                        (ui.content_region_avail()[0] - ui.clone_style().item_spacing[0]) / 2.0;                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Cancel", [button_width, 0.0], &neuomorphic_style) {
                        *transfer_state = GrenadeHelperTransferState::Idle;
                        return;
                    }

                    ui.same_line();
                    if ui.neuomorphic_button("Yes", [button_width, 0.0], &neuomorphic_style) {
                        settings.map_spots = elements.clone();
                        *transfer_state = GrenadeHelperTransferState::ImportSuccess {
                            count: total_count,
                            replacing: false,
                        };
                    }
                } else {
                    ui.open_popup("Data Import");
                }
            }

            GrenadeHelperTransferState::Failed { direction, message } => {
                let mut popup_open = true;
                let popup_name = format!(
                    "{} failed",
                    match direction {
                        GrenadeHelperTransferDirection::Export => "Export",
                        GrenadeHelperTransferDirection::Import => "Import",
                    }
                );
                if let Some(_popup) = ui
                    .modal_popup_config(&popup_name)
                    .opened(&mut popup_open)
                    .always_auto_resize(true)
                    .begin_popup()
                {                    ui.text("A fatal error occurred:");
                    ui.spacing();

                    ui.text(message);

                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                                       let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Close", [100.0, 0.0], &neuomorphic_style) {
                        popup_open = false;
                    }
                } else {
                    ui.open_popup(&popup_name);
                }

                if !popup_open {
                    *transfer_state = GrenadeHelperTransferState::Idle;
                }
            }
            GrenadeHelperTransferState::ExportSuccess { target_path } => {
                let mut popup_open = true;
                if let Some(_popup) = ui
                    .modal_popup_config("Export successfull")
                    .opened(&mut popup_open)
                    .always_auto_resize(true)
                    .begin_popup()
                {                    ui.text("All grenade helper spots have been exported to");
                    ui.text(format!("{}", target_path.display()));

                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Close", [100.0, 0.0], &neuomorphic_style) {
                        popup_open = false;
                    }
                } else {
                    ui.open_popup("Export successfull");
                }

                if !popup_open {
                    *transfer_state = GrenadeHelperTransferState::Idle;
                }
            }
            GrenadeHelperTransferState::ImportSuccess { count, .. } => {
                let mut popup_open = true;
                if let Some(_popup) = ui
                    .modal_popup_config("Import successful")
                    .opened(&mut popup_open)
                    .always_auto_resize(true)
                    .begin_popup()
                {                    ui.text(format!("{} grenade helper spots have been imported", count));

                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    let neuomorphic_style = crate::settings::get_neuomorphic_style();
                    if ui.neuomorphic_button("Close", [100.0, 0.0], &neuomorphic_style) {
                        popup_open = false;
                    }
                } else {
                    ui.open_popup("Import successfull");
                }

                if !popup_open {
                    *transfer_state = GrenadeHelperTransferState::Idle;
                }
            }
        }
    }
}
// Helper to convert ImColor32 to [f32; 4]



fn to_f32_color(color: imgui::ImColor32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    ]
}
