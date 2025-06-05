// Neuomorphic Design System for ImGui
// Ultra-modern, sleek UI implementation with depth and soft shadows

use imgui::{
    DrawListMut,
    ImColor32,
    StyleColor,
    StyleVar,
    Ui,
    draw_list::DrawFlags,
};

/// Neuomorphic color palette for the modern design
#[derive(Clone)]
pub struct NeuomorphicPalette {
    // Base colors
    pub background: ImColor32,
    pub surface: ImColor32,
    pub surface_elevated: ImColor32,
    pub surface_depressed: ImColor32,
    
    // Accent colors
    pub primary: ImColor32,
    pub primary_hover: ImColor32,
    pub primary_active: ImColor32,
    
    // Text colors
    pub text_primary: ImColor32,
    pub text_secondary: ImColor32,
    pub text_disabled: ImColor32,
    
    // Shadow colors
    pub shadow_light: ImColor32,
    pub shadow_dark: ImColor32,
    
    // Border colors
    pub border_light: ImColor32,
    pub border_dark: ImColor32,
}

impl Default for NeuomorphicPalette {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl NeuomorphicPalette {
    /// Modern dark neuomorphic theme with subtle purple accents
    pub fn dark_theme() -> Self {
        Self {
            // Base neutral colors with slight purple tint
            background: ImColor32::from_rgba(28, 30, 35, 255),       // #1c1e23
            surface: ImColor32::from_rgba(35, 38, 45, 255),          // #23262d
            surface_elevated: ImColor32::from_rgba(42, 45, 52, 255), // #2a2d34
            surface_depressed: ImColor32::from_rgba(25, 27, 32, 255), // #191b20
            
            // Modern purple accent
            primary: ImColor32::from_rgba(138, 43, 226, 255),        // #8a2be2
            primary_hover: ImColor32::from_rgba(155, 70, 235, 255),  // #9b46eb
            primary_active: ImColor32::from_rgba(120, 30, 200, 255), // #781ec8
            
            // High contrast text
            text_primary: ImColor32::from_rgba(245, 246, 250, 255),  // #f5f6fa
            text_secondary: ImColor32::from_rgba(180, 185, 195, 255), // #b4b9c3
            text_disabled: ImColor32::from_rgba(120, 125, 135, 255),  // #787d87
            
            // Soft shadows for depth
            shadow_light: ImColor32::from_rgba(55, 60, 70, 80),      // Subtle highlight
            shadow_dark: ImColor32::from_rgba(15, 17, 22, 120),      // Soft shadow
            
            // Subtle borders
            border_light: ImColor32::from_rgba(60, 65, 75, 255),     // #3c414b
            border_dark: ImColor32::from_rgba(20, 22, 27, 255),      // #14161b
        }
    }
    
    /// Light neuomorphic theme for alternative styling
    pub fn light_theme() -> Self {
        Self {
            background: ImColor32::from_rgba(240, 242, 247, 255),
            surface: ImColor32::from_rgba(235, 238, 245, 255),
            surface_elevated: ImColor32::from_rgba(250, 252, 255, 255),
            surface_depressed: ImColor32::from_rgba(225, 230, 240, 255),
            
            primary: ImColor32::from_rgba(138, 43, 226, 255),
            primary_hover: ImColor32::from_rgba(155, 70, 235, 255),
            primary_active: ImColor32::from_rgba(120, 30, 200, 255),
            
            text_primary: ImColor32::from_rgba(30, 35, 45, 255),
            text_secondary: ImColor32::from_rgba(80, 90, 105, 255),
            text_disabled: ImColor32::from_rgba(140, 150, 165, 255),
            
            shadow_light: ImColor32::from_rgba(255, 255, 255, 150),
            shadow_dark: ImColor32::from_rgba(180, 190, 210, 80),
            
            border_light: ImColor32::from_rgba(255, 255, 255, 200),
            border_dark: ImColor32::from_rgba(200, 210, 225, 255),
        }
    }
}

/// Neuomorphic styling configuration
#[derive(Clone)]
pub struct NeuomorphicStyle {
    pub palette: NeuomorphicPalette,
    pub shadow_offset: f32,
    pub shadow_blur: f32,
    pub border_radius: f32,
    pub elevation: f32,
}

impl Default for NeuomorphicStyle {
    fn default() -> Self {
        Self {
            palette: NeuomorphicPalette::default(),
            shadow_offset: 2.0,
            shadow_blur: 4.0,
            border_radius: 8.0,
            elevation: 4.0,
        }
    }
}

/// Apply neuomorphic styling to ImGui
pub trait NeuomorphicUi {
    fn apply_neuomorphic_style(&self, style: &NeuomorphicStyle);
    fn push_neuomorphic_window_style(&self, style: &NeuomorphicStyle);
    fn pop_neuomorphic_window_style(&self);
    fn push_neuomorphic_button_style(&self, style: &NeuomorphicStyle);
    fn pop_neuomorphic_button_style(&self);
    fn neuomorphic_button(&self, label: &str, size: [f32; 2], style: &NeuomorphicStyle) -> bool;
    fn neuomorphic_checkbox(&self, label: &str, value: &mut bool, style: &NeuomorphicStyle) -> bool;
    fn neuomorphic_slider_f32(&self, label: &str, value: &mut f32, min: f32, max: f32, format: &str, style: &NeuomorphicStyle) -> bool;
    fn neuomorphic_combo_enum<T: PartialEq + Copy>(&self, label: impl AsRef<str>, values: &[(T, &'static str)], value: &mut T, style: &NeuomorphicStyle) -> bool;    fn neuomorphic_input_text<'p>(&self, label: impl AsRef<str>, buf: &'p mut String, style: &NeuomorphicStyle) -> bool;
    fn neuomorphic_input_float(&self, label: impl AsRef<str>, value: &mut f32, style: &NeuomorphicStyle) -> bool;
    fn neuomorphic_color_edit4<'a>(&self, label: impl AsRef<str>, col: &'a mut [f32; 4], style: &NeuomorphicStyle) -> bool;
}

impl NeuomorphicUi for Ui {    fn apply_neuomorphic_style(&self, style: &NeuomorphicStyle) {
        let mut ui_style = self.clone_style();
        
        // Helper function to convert ImColor32 to [f32; 4]
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        // Window styling
        ui_style[StyleColor::WindowBg] = to_f32_color(style.palette.background);
        ui_style[StyleColor::ChildBg] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::PopupBg] = to_f32_color(style.palette.surface_elevated);
        ui_style[StyleColor::Border] = to_f32_color(style.palette.border_dark);
        ui_style[StyleColor::BorderShadow] = to_f32_color(style.palette.shadow_dark);
          // Frame styling (inputs, etc.)
        ui_style[StyleColor::FrameBg] = to_f32_color(style.palette.surface_depressed);
        ui_style[StyleColor::FrameBgHovered] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::FrameBgActive] = to_f32_color(style.palette.surface_elevated);
        
        // Title bar
        ui_style[StyleColor::TitleBg] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::TitleBgActive] = to_f32_color(style.palette.surface_elevated);
        ui_style[StyleColor::TitleBgCollapsed] = to_f32_color(style.palette.surface_depressed);
        
        // Button styling
        ui_style[StyleColor::Button] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::ButtonHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::ButtonActive] = to_f32_color(style.palette.primary_active);
        
        // Header styling (for collapsing headers, etc.)
        ui_style[StyleColor::Header] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::HeaderHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::HeaderActive] = to_f32_color(style.palette.primary_active);
        
        // Separator
        ui_style[StyleColor::Separator] = to_f32_color(style.palette.border_dark);
        ui_style[StyleColor::SeparatorHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::SeparatorActive] = to_f32_color(style.palette.primary_active);
          // Resize grip
        ui_style[StyleColor::ResizeGrip] = to_f32_color(style.palette.border_dark);
        ui_style[StyleColor::ResizeGripHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::ResizeGripActive] = to_f32_color(style.palette.primary_active);
        
        // Tab styling
        ui_style[StyleColor::Tab] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::TabHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::TabActive] = to_f32_color(style.palette.primary);
        ui_style[StyleColor::TabUnfocused] = to_f32_color(style.palette.surface_depressed);
        ui_style[StyleColor::TabUnfocusedActive] = to_f32_color(style.palette.surface);
        
        // Text colors
        ui_style[StyleColor::Text] = to_f32_color(style.palette.text_primary);
        ui_style[StyleColor::TextDisabled] = to_f32_color(style.palette.text_disabled);
        ui_style[StyleColor::TextSelectedBg] = to_f32_color(style.palette.primary);
        
        // Checkbox
        ui_style[StyleColor::CheckMark] = to_f32_color(style.palette.primary);
        
        // Slider
        ui_style[StyleColor::SliderGrab] = to_f32_color(style.palette.primary);
        ui_style[StyleColor::SliderGrabActive] = to_f32_color(style.palette.primary_active);
          // Scrollbar
        ui_style[StyleColor::ScrollbarBg] = to_f32_color(style.palette.surface_depressed);
        ui_style[StyleColor::ScrollbarGrab] = to_f32_color(style.palette.surface);
        ui_style[StyleColor::ScrollbarGrabHovered] = to_f32_color(style.palette.primary_hover);
        ui_style[StyleColor::ScrollbarGrabActive] = to_f32_color(style.palette.primary_active);
        
        // Apply rounded corners and padding
        ui_style.window_rounding = style.border_radius;
        ui_style.child_rounding = style.border_radius;
        ui_style.frame_rounding = style.border_radius;
        ui_style.popup_rounding = style.border_radius;
        ui_style.scrollbar_rounding = style.border_radius;
        ui_style.grab_rounding = style.border_radius;
        ui_style.tab_rounding = style.border_radius;
        
        // Adjust spacing for modern look
        ui_style.window_padding = [12.0, 12.0];
        ui_style.frame_padding = [8.0, 6.0];
        ui_style.item_spacing = [8.0, 6.0];
        ui_style.item_inner_spacing = [6.0, 4.0];
        ui_style.indent_spacing = 20.0;
        ui_style.scrollbar_size = 14.0;
        ui_style.grab_min_size = 12.0;
        
        // Border and shadow settings
        ui_style.window_border_size = 1.0;
        ui_style.child_border_size = 1.0;
        ui_style.popup_border_size = 1.0;
        ui_style.frame_border_size = 0.0;
        ui_style.tab_border_size = 0.0;
    }
      fn push_neuomorphic_window_style(&self, style: &NeuomorphicStyle) {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        self.push_style_color(StyleColor::WindowBg, to_f32_color(style.palette.surface));
        self.push_style_color(StyleColor::Border, to_f32_color(style.palette.border_light));
        self.push_style_var(StyleVar::WindowRounding(style.border_radius));
        self.push_style_var(StyleVar::WindowPadding([12.0, 12.0]));
    }
      fn pop_neuomorphic_window_style(&self) {
        // Using tokens instead of manual pop calls
    }
      fn push_neuomorphic_button_style(&self, style: &NeuomorphicStyle) {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        self.push_style_color(StyleColor::Button, to_f32_color(style.palette.surface));
        self.push_style_color(StyleColor::ButtonHovered, to_f32_color(style.palette.primary_hover));
        self.push_style_color(StyleColor::ButtonActive, to_f32_color(style.palette.primary_active));
        self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        self.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
    }
      fn pop_neuomorphic_button_style(&self) {
        // Using tokens instead of manual pop calls
    }
    
    fn neuomorphic_button(&self, label: &str, size: [f32; 2], style: &NeuomorphicStyle) -> bool {
        self.push_neuomorphic_button_style(style);
        let result = self.button_with_size(label, size);
        self.pop_neuomorphic_button_style();
        result
    }    fn neuomorphic_checkbox(&self, label: &str, value: &mut bool, style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::CheckMark, to_f32_color(style.palette.primary));
        let _color_token2 = self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        let _color_token3 = self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
        let _color_token4 = self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        let _var_token = self.push_style_var(StyleVar::FrameRounding(style.border_radius * 0.5));
        
        self.checkbox(label, value)
    }    fn neuomorphic_slider_f32(&self, label: &str, value: &mut f32, min: f32, max: f32, format: &str, style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        let _color_token2 = self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
        let _color_token3 = self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        let _color_token4 = self.push_style_color(StyleColor::SliderGrab, to_f32_color(style.palette.primary));
        let _color_token5 = self.push_style_color(StyleColor::SliderGrabActive, to_f32_color(style.palette.primary_active));
        let _var_token1 = self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        let _var_token2 = self.push_style_var(StyleVar::GrabRounding(style.border_radius));
        
        self.slider(label, min, max, value)
    }    fn neuomorphic_combo_enum<T: PartialEq + Copy>(&self, label: impl AsRef<str>, values: &[(T, &'static str)], value: &mut T, style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        let _color_token2 = self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
        let _color_token3 = self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        let _color_token4 = self.push_style_color(StyleColor::PopupBg, to_f32_color(style.palette.surface));
        let _color_token5 = self.push_style_color(StyleColor::Header, to_f32_color(style.palette.surface_elevated));
        let _color_token6 = self.push_style_color(StyleColor::HeaderHovered, to_f32_color(style.palette.primary_hover));
        let _color_token7 = self.push_style_color(StyleColor::HeaderActive, to_f32_color(style.palette.primary_active));
        let _var_token1 = self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        let _var_token2 = self.push_style_var(StyleVar::PopupRounding(style.border_radius));
        
        use crate::utils::ImguiComboEnum;
        self.combo_enum(label, values, value)
    }    fn neuomorphic_input_text<'p>(&self, label: impl AsRef<str>, buf: &'p mut String, style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        let _color_token2 = self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
        let _color_token3 = self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        let _color_token4 = self.push_style_color(StyleColor::Text, to_f32_color(style.palette.text_primary));
        let _var_token1 = self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        let _var_token2 = self.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
        
        self.input_text(label, buf).build()
    }    fn neuomorphic_input_float(&self, label: impl AsRef<str>, value: &mut f32, style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        let _color_token2 = self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
        let _color_token3 = self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        let _color_token4 = self.push_style_color(StyleColor::Text, to_f32_color(style.palette.text_primary));
        let _var_token1 = self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        let _var_token2 = self.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
        
        self.input_float(label.as_ref(), value).build()
    }
      fn neuomorphic_color_edit4<'a>(&self, label: impl AsRef<str>, col: &'a mut [f32; 4], style: &NeuomorphicStyle) -> bool {
        // Convert ImColor32 to [f32; 4] for push_style_color
        let to_f32_color = |color: ImColor32| -> [f32; 4] {
            [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            ]
        };
        
        self.push_style_color(StyleColor::FrameBg, to_f32_color(style.palette.surface_depressed));
        self.push_style_color(StyleColor::FrameBgHovered, to_f32_color(style.palette.surface));
                self.push_style_color(StyleColor::FrameBgActive, to_f32_color(style.palette.surface_elevated));
        self.push_style_var(StyleVar::FrameRounding(style.border_radius));
        self.push_style_var(StyleVar::FramePadding([10.0, 8.0]));
        
        // In ImGui 0.12.0, color_edit4 returns bool directly
        let result = self.color_edit4(label.as_ref(), col);
        
        // Tokens automatically clean up styles when dropped
        result
    }
}

/// Enhanced drawing functions for neuomorphic effects
pub trait NeuomorphicDrawList {
    fn add_neuomorphic_rect(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle, elevated: bool);
    fn add_neuomorphic_rect_filled(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle, elevated: bool);
    fn add_soft_shadow(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle);
    fn add_inner_glow(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle);
}

impl NeuomorphicDrawList for DrawListMut<'_> {    fn add_neuomorphic_rect(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle, elevated: bool) {
        // Simplified implementation using only add_rect
        // Add border
        self.add_rect(
            p_min, 
            p_max,
            style.palette.border_light,
        );
    }
    
    fn add_neuomorphic_rect_filled(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle, elevated: bool) {
        self.add_neuomorphic_rect(p_min, p_max, style, elevated);
    }    fn add_soft_shadow(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle) {
        // Simplified shadow implementation
        let shadow_offset = style.shadow_offset;
        let shadow_p_min = [p_min[0] + shadow_offset, p_min[1] + shadow_offset];
        let shadow_p_max = [p_max[0] + shadow_offset, p_max[1] + shadow_offset];
        
        // Simple shadow using add_rect
        self.add_rect(
            shadow_p_min,
            shadow_p_max,
            style.palette.shadow_dark,
        );
    }    fn add_inner_glow(&self, p_min: [f32; 2], p_max: [f32; 2], style: &NeuomorphicStyle) {
        let glow_inset = 2.0;
        let glow_p_min = [p_min[0] + glow_inset, p_min[1] + glow_inset];
        let glow_p_max = [p_max[0] - glow_inset, p_max[1] - glow_inset];
        
        // Simple glow using add_rect
        self.add_rect(
            glow_p_min,
            glow_p_max,
            style.palette.shadow_light,
        );
    }
}
