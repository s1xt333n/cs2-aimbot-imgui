// Global theme manager for the neuomorphic design system
use std::sync::{Arc, Mutex, OnceLock};

use crate::utils::neuomorphic::{NeuomorphicStyle, NeuomorphicPalette};

/// Global theme manager instance
static NEUOMORPHIC_THEME: OnceLock<Arc<Mutex<NeuomorphicThemeManager>>> = OnceLock::new();

/// Initialize the global theme manager
fn get_theme_manager() -> &'static Arc<Mutex<NeuomorphicThemeManager>> {
    NEUOMORPHIC_THEME.get_or_init(|| Arc::new(Mutex::new(NeuomorphicThemeManager::new())))
}

/// Theme manager for consistent styling across the application
pub struct NeuomorphicThemeManager {
    current_style: NeuomorphicStyle,
    is_enabled: bool,
}

impl NeuomorphicThemeManager {
    pub fn new() -> Self {
        Self {
            current_style: NeuomorphicStyle::default(),
            is_enabled: true,
        }
    }
    
    pub fn get_style(&self) -> &NeuomorphicStyle {
        &self.current_style
    }
    
    pub fn set_style(&mut self, style: NeuomorphicStyle) {
        self.current_style = style;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }
    
    pub fn switch_to_dark_theme(&mut self) {
        self.current_style.palette = NeuomorphicPalette::dark_theme();
    }
      pub fn switch_to_light_theme(&mut self) {
        self.current_style.palette = NeuomorphicPalette::light_theme();
    }
      /// Check if the current theme is dark by comparing background brightness
    pub fn is_dark_theme(&self) -> bool {
        let bg = self.current_style.palette.background;
        // Extract RGB values from ImColor32
        let r = bg.r as f32;
        let g = bg.g as f32;
        let b = bg.b as f32;
        
        // Calculate perceived brightness using standard luminance formula
        let brightness = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
        
        // If brightness is less than 0.5, consider it dark
        brightness < 0.5
    }

    pub fn set_accent_color(&mut self, color: [u8; 4]) {
        let primary = imgui::ImColor32::from_rgba(color[0], color[1], color[2], color[3]);
        self.current_style.palette.primary = primary;
        
        // Generate hover and active variants
        let hover_factor = 1.2;
        let active_factor = 0.8;
        
        self.current_style.palette.primary_hover = imgui::ImColor32::from_rgba(
            ((color[0] as f32 * hover_factor).min(255.0)) as u8,
            ((color[1] as f32 * hover_factor).min(255.0)) as u8,
            ((color[2] as f32 * hover_factor).min(255.0)) as u8,
            color[3],
        );
        
        self.current_style.palette.primary_active = imgui::ImColor32::from_rgba(
            (color[0] as f32 * active_factor) as u8,
            (color[1] as f32 * active_factor) as u8,
            (color[2] as f32 * active_factor) as u8,
            color[3],
        );
    }
}

/// Convenience function to get the current theme
pub fn get_neuomorphic_style() -> NeuomorphicStyle {
    get_theme_manager()
        .lock()
        .unwrap()
        .get_style()
        .clone()
}

/// Apply the global neuomorphic theme to the current UI
pub fn apply_global_neuomorphic_theme(ui: &imgui::Ui) {
    let theme = get_theme_manager().lock().unwrap();
    if theme.is_enabled() {
        use crate::utils::neuomorphic::NeuomorphicUi;
        ui.apply_neuomorphic_style(theme.get_style());
    }
}

/// Enable or disable the neuomorphic theme
pub fn set_neuomorphic_enabled(enabled: bool) {
    get_theme_manager().lock().unwrap().set_enabled(enabled);
}

/// Switch to dark theme
pub fn switch_to_dark_theme() {
    get_theme_manager().lock().unwrap().switch_to_dark_theme();
}

/// Switch to light theme  
pub fn switch_to_light_theme() {
    get_theme_manager().lock().unwrap().switch_to_light_theme();
}

/// Check if the current theme is dark
pub fn is_dark_theme() -> bool {
    get_theme_manager().lock().unwrap().is_dark_theme()
}

/// Set custom accent color
pub fn set_accent_color(color: [u8; 4]) {
    get_theme_manager().lock().unwrap().set_accent_color(color);
}

/// Macro for easy neuomorphic styling in UI code
#[macro_export]
macro_rules! with_neuomorphic_style {
    ($ui:expr, $style_type:ident, $block:block) => {
        {
            let style = crate::settings::theme::get_neuomorphic_style();
            $ui.paste(stringify!(push_neuomorphic_), stringify!($style_type), "_style")(&style);
            let result = $block;
            $ui.paste(stringify!(pop_neuomorphic_), stringify!($style_type), "_style")();
            result
        }
    };
}
