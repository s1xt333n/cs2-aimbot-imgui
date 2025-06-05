// RAGNAREK ImGui Integration for Valthrun
// This module provides enhanced UI components using the RAGNAREK v2 ImGui implementation

use imgui::{Ui, StyleColor, StyleVar, DrawListMut};
use std::time::{SystemTime, UNIX_EPOCH};

/// RAGNAREK Enhanced UI trait that extends the neuomorphic design
pub trait RagnarekUi {
    /// Enhanced tab with RAGNAREK styling
    fn ragnarek_tab(&self, selected: bool, icon: &str, label: &str, size: [f32; 2]) -> bool;
    
    /// Enhanced checkbox with RAGNAREK animations
    fn ragnarek_checkbox(&self, label: &str, value: &mut bool) -> bool;
    
    /// Enhanced color picker with RAGNAREK interface
    fn ragnarek_color_picker(&self, label: &str, color: &mut [f32; 4]) -> bool;
    
    /// Enhanced slider with RAGNAREK styling
    fn ragnarek_slider_float(&self, label: &str, value: &mut f32, min: f32, max: f32) -> bool;
    
    /// Enhanced combo box with RAGNAREK animations
    fn ragnarek_combo(&self, label: &str, current_item: &mut usize, items: &[&str]) -> bool;
    
    /// Enhanced selectable with RAGNAREK hover effects
    fn ragnarek_selectable(&self, label: &str, selected: bool, size: [f32; 2]) -> bool;
    
    /// Enhanced keybind selector with RAGNAREK styling
    fn ragnarek_keybind(&self, label: &str, key: &mut i32) -> bool;
    
    /// RAGNAREK child window with enhanced borders and shadows
    fn ragnarek_child_window<F>(&self, name: &str, size: [f32; 2], f: F) -> bool
    where
        F: FnOnce();
    
    // Advanced RAGNAREK components
    /// RAGNAREK-style text with glow effect
    fn ragnarek_text_with_glow(&self, text: &str, color: [f32; 4], glow_color: [f32; 4]);
    
    /// RAGNAREK-style separator with gradient
    fn ragnarek_separator_gradient(&self, color1: [f32; 4], color2: [f32; 4]);
    
    /// RAGNAREK-style progress bar with glow
    fn ragnarek_progress_bar(&self, fraction: f32, size: [f32; 2], overlay_text: Option<&str>);
    
    /// RAGNAREK-style tooltip with enhanced styling
    fn ragnarek_tooltip<F>(&self, f: F) 
    where
        F: FnOnce();
    
    /// Enhanced animated button with particle effects
    fn ragnarek_animated_button(&self, label: &str, size: [f32; 2], pulse_intensity: f32) -> bool;    /// Glowing input field with enhanced borders
    fn ragnarek_input_text(&self, label: &str, buffer: &mut String, size: [f32; 2]) -> bool;
    
    /// Animated loading spinner with RAGNAREK styling
    fn ragnarek_loading_spinner(&self, radius: f32, thickness: f32, speed: f32);
    
    /// Enhanced window header with gradient background
    fn ragnarek_window_header(&self, title: &str, gradient_start: [f32; 4], gradient_end: [f32; 4]);
    
    /// Particle effect background for sections
    fn ragnarek_particle_background(&self, size: [f32; 2], particle_count: u32, color: [f32; 4]);
    
    /// Animated toggle switch with RAGNAREK styling
    fn ragnarek_toggle_switch(&self, label: &str, value: &mut bool, size: [f32; 2]) -> bool;
    
    /// Enhanced group box with glow effects
    fn ragnarek_group_box<F>(&self, label: &str, size: [f32; 2], glow_color: [f32; 4], f: F)
    where
        F: FnOnce();

    /// Status indicator with pulsing effects
    fn ragnarek_status_indicator(&self, status: &str, color: [f32; 4], pulse: bool);
}

impl RagnarekUi for Ui {
    fn ragnarek_tab(&self, selected: bool, icon: &str, label: &str, size: [f32; 2]) -> bool {
        // Apply RAGNAREK tab styling
        let bg_color = if selected {
            [0.2, 0.4, 0.8, 1.0] // Selected blue
        } else {
            [0.15, 0.15, 0.15, 1.0] // Dark background
        };
        
        let hover_color = [0.25, 0.45, 0.85, 1.0];
        let active_color = [0.3, 0.5, 0.9, 1.0];
        
        // Push RAGNAREK styling
        let _color_token1 = self.push_style_color(StyleColor::Button, bg_color);
        let _color_token2 = self.push_style_color(StyleColor::ButtonHovered, hover_color);
        let _color_token3 = self.push_style_color(StyleColor::ButtonActive, active_color);
        let _style_token1 = self.push_style_var(StyleVar::FrameRounding(8.0));
        let _style_token2 = self.push_style_var(StyleVar::FramePadding([12.0, 8.0]));
        
        // Create the tab button
        let button_label = format!("{} {}", icon, label);
        let result = self.button_with_size(&button_label, size);
        
        // Add subtle glow effect for selected tab
        if selected {
            let min = self.item_rect_min();
            let max = self.item_rect_max();
            
            // Get draw list and add glow outline
            let draw_list = self.get_window_draw_list();
            draw_list.add_rect(
                [min[0] - 1.0, min[1] - 1.0],
                [max[0] + 1.0, max[1] + 1.0],
                [0.2, 0.4, 0.8, 0.5]
            ).thickness(2.0).build();
        }
        
        result
    }
    
    fn ragnarek_checkbox(&self, label: &str, value: &mut bool) -> bool {
        // Enhanced checkbox with RAGNAREK animations
        let check_color = [0.2, 0.8, 0.2, 1.0]; // Green check
        let bg_color = if *value {
            [0.1, 0.3, 0.1, 1.0] // Dark green background when checked
        } else {
            [0.2, 0.2, 0.2, 1.0] // Dark background when unchecked
        };
        
        let _color_token1 = self.push_style_color(StyleColor::CheckMark, check_color);
        let _color_token2 = self.push_style_color(StyleColor::FrameBg, bg_color);
        let _style_token = self.push_style_var(StyleVar::FrameRounding(4.0));
        
        let result = self.checkbox(label, value);
        
        // Add custom animation effects here if needed
        if result {
            // Trigger animation or effect
        }
        
        result
    }
    
    fn ragnarek_color_picker(&self, label: &str, color: &mut [f32; 4]) -> bool {
        // Enhanced color picker with RAGNAREK styling
        let _style_token1 = self.push_style_var(StyleVar::FrameRounding(6.0));
        let _style_token2 = self.push_style_var(StyleVar::WindowRounding(8.0));
        
        self.color_edit4(label, color)
    }
    
    fn ragnarek_slider_float(&self, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
        // Enhanced slider with RAGNAREK styling
        let grab_color = [0.3, 0.6, 1.0, 1.0]; // Blue grab
        let track_color = [0.2, 0.2, 0.2, 1.0]; // Dark track
        
        let _color_token1 = self.push_style_color(StyleColor::SliderGrab, grab_color);
        let _color_token2 = self.push_style_color(StyleColor::SliderGrabActive, [0.4, 0.7, 1.0, 1.0]);
        let _color_token3 = self.push_style_color(StyleColor::FrameBg, track_color);
        let _style_token = self.push_style_var(StyleVar::GrabRounding(6.0));
        
        self.slider(label, min, max, value)
    }    fn ragnarek_combo(&self, label: &str, current_item: &mut usize, items: &[&str]) -> bool {
        // Enhanced combo with RAGNAREK styling
        let _color_token1 = self.push_style_color(StyleColor::Header, [0.2, 0.2, 0.2, 1.0]);
        let _color_token2 = self.push_style_color(StyleColor::HeaderHovered, [0.3, 0.3, 0.3, 1.0]);
        let _style_token = self.push_style_var(StyleVar::FrameRounding(6.0));
        
        self.combo(label, current_item, items, |item| (*item).into())
    }
    
    fn ragnarek_selectable(&self, label: &str, selected: bool, size: [f32; 2]) -> bool {
        // Enhanced selectable with RAGNAREK hover effects
        let select_color = if selected {
            [0.2, 0.4, 0.8, 1.0]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };
        
        let _color_token1 = self.push_style_color(StyleColor::Header, select_color);
        let _color_token2 = self.push_style_color(StyleColor::HeaderHovered, [0.25, 0.45, 0.85, 0.8]);
        let _style_token = self.push_style_var(StyleVar::SelectableTextAlign([0.0, 0.5]));
        
        self.selectable_config(label)
            .selected(selected)
            .size(size)
            .build()
    }
    
    fn ragnarek_keybind(&self, label: &str, key: &mut i32) -> bool {
        // Enhanced keybind selector with RAGNAREK styling
        let key_name = match *key {
            0 => "None".to_string(),
            1 => "Mouse 1".to_string(),
            2 => "Mouse 2".to_string(),
            _ => format!("Key {}", *key),
        };
        
        let button_label = format!("{}: {}", label, key_name);
        
        let _color_token1 = self.push_style_color(StyleColor::Button, [0.3, 0.3, 0.3, 1.0]);
        let _color_token2 = self.push_style_color(StyleColor::ButtonHovered, [0.4, 0.4, 0.4, 1.0]);
        let _style_token = self.push_style_var(StyleVar::FrameRounding(4.0));
        
        let clicked = self.button(&button_label);
        
        if clicked {
            // Here you would implement key capture logic
            // For now, just cycle through some example keys
            *key = (*key + 1) % 10;
        }
        
        clicked
    }
    
    fn ragnarek_child_window<F>(&self, name: &str, size: [f32; 2], f: F) -> bool
    where
        F: FnOnce()
    {
        // Enhanced child window with RAGNAREK borders and shadows
        let _color_token1 = self.push_style_color(StyleColor::ChildBg, [0.1, 0.1, 0.1, 0.9]);
        let _color_token2 = self.push_style_color(StyleColor::Border, [0.3, 0.3, 0.3, 1.0]);
        let _style_token1 = self.push_style_var(StyleVar::ChildRounding(8.0));
        let _style_token2 = self.push_style_var(StyleVar::ChildBorderSize(1.0));
        let _style_token3 = self.push_style_var(StyleVar::WindowPadding([8.0, 8.0]));        if let Some(_token) = self.child_window(name)
            .size(size)
            .border(true)
            .begin()
        {
            f();
            true
        } else {
            false
        }
    }    /// RAGNAREK-style text with glow effect
    fn ragnarek_text_with_glow(&self, text: &str, color: [f32; 4], glow_color: [f32; 4]) {
        let pos = self.cursor_screen_pos();
        let draw_list = self.get_window_draw_list();
        
        // Draw glow effect (offset text in glow color)
        for x_offset in [-1.0, 0.0, 1.0] {
            for y_offset in [-1.0, 0.0, 1.0] {
                if x_offset != 0.0 || y_offset != 0.0 {
                    draw_list.add_text(
                        [pos[0] + x_offset, pos[1] + y_offset],
                        glow_color,
                        text
                    );
                }
            }
        }
        
        // Draw main text
        draw_list.add_text(pos, color, text);
        self.dummy([self.calc_text_size(text)[0], self.text_line_height()]);
    }/// RAGNAREK-style separator with gradient
    fn ragnarek_separator_gradient(&self, color1: [f32; 4], color2: [f32; 4]) {
        let pos = self.cursor_screen_pos();
        let width = self.content_region_avail()[0];
        
        // Get draw list and draw gradient line
        let draw_list = self.get_window_draw_list();
        draw_list.add_rect_filled_multicolor(
            pos,
            [pos[0] + width, pos[1] + 2.0],
            color1,
            color2,
            color2,
            color1
        );
        
        self.dummy([width, 4.0]);
    }    /// RAGNAREK-style progress bar with glow
    fn ragnarek_progress_bar(&self, fraction: f32, size: [f32; 2], overlay_text: Option<&str>) {
        let pos = self.cursor_screen_pos();
        let draw_list = self.get_window_draw_list();
        
        // Background
        draw_list.add_rect(
            pos,
            [pos[0] + size[0], pos[1] + size[1]],
            [0.2, 0.2, 0.2, 1.0]
        )
        .filled(true)
        .build();
        
        // Progress bar with gradient
        let progress_width = size[0] * fraction;
        if progress_width > 0.0 {
            draw_list.add_rect_filled_multicolor(
                pos,
                [pos[0] + progress_width, pos[1] + size[1]],
                [0.1, 0.6, 1.0, 1.0],  // Blue start
                [0.0, 0.4, 0.8, 1.0],  // Blue end
                [0.0, 0.4, 0.8, 1.0],  // Blue end
                [0.1, 0.6, 1.0, 1.0]   // Blue start
            );
            
            // Glow effect
            draw_list.add_rect(
                [pos[0], pos[1] - 1.0],
                [pos[0] + progress_width, pos[1] + size[1] + 1.0],
                [0.1, 0.6, 1.0, 0.3]
            )
            .filled(true)
            .build();
        }
        
        // Border
        draw_list.add_rect(
            pos,
            [pos[0] + size[0], pos[1] + size[1]],
            [0.4, 0.4, 0.4, 1.0]
        )
        .thickness(1.0)
        .build();
        
        // Overlay text
        if let Some(text) = overlay_text {
            let text_size = self.calc_text_size(text);
            let text_pos = [
                pos[0] + (size[0] - text_size[0]) * 0.5,
                pos[1] + (size[1] - text_size[1]) * 0.5
            ];
            draw_list.add_text(text_pos, [1.0, 1.0, 1.0, 1.0], text);
        }
        
        self.dummy(size);
    }
    
    /// RAGNAREK-style tooltip with enhanced styling
    fn ragnarek_tooltip<F>(&self, f: F) 
    where
        F: FnOnce()
    {
        if self.is_item_hovered() {
            let _color_token = self.push_style_color(StyleColor::PopupBg, [0.1, 0.1, 0.1, 0.95]);
            let _style_token1 = self.push_style_var(StyleVar::WindowRounding(8.0));
            let _style_token2 = self.push_style_var(StyleVar::WindowPadding([8.0, 6.0]));
            
            self.tooltip(f);
        }
    }
    
    fn ragnarek_animated_button(&self, label: &str, size: [f32; 2], pulse_intensity: f32) -> bool {
        // Enhanced animated button with RAGNAREK styling
        let pulse_color = [0.3, 0.6, 1.0, 1.0]; // Blue pulse
        let bg_color = [0.2, 0.2, 0.2, 1.0]; // Dark background
        
        let _color_token1 = self.push_style_color(StyleColor::Button, bg_color);
        let _color_token2 = self.push_style_color(StyleColor::ButtonHovered, pulse_color);
        let _color_token3 = self.push_style_color(StyleColor::ButtonActive, [0.4, 0.7, 1.0, 1.0]);
        let _style_token1 = self.push_style_var(StyleVar::FrameRounding(6.0));
        let _style_token2 = self.push_style_var(StyleVar::FramePadding([12.0, 8.0]));
        
        // Create the button with pulse effect
        let result = self.button_with_size(label, size);
        
        // Add pulse animation
        if result {
            // Trigger pulse effect
        }
        
        result
    }
    
    fn ragnarek_input_text(&self, label: &str, buffer: &mut String, size: [f32; 2]) -> bool {
        // Glowing input field with enhanced borders
        let glow_color = [0.3, 0.6, 1.0, 1.0]; // Blue glow
        let bg_color = [0.1, 0.1, 0.1, 1.0]; // Dark background
        
        let _color_token1 = self.push_style_color(StyleColor::FrameBg, bg_color);
        let _color_token2 = self.push_style_color(StyleColor::Border, glow_color);        let _style_token1 = self.push_style_var(StyleVar::FrameRounding(4.0));
        let _style_token2 = self.push_style_var(StyleVar::FrameBorderSize(1.0));
          // Create the input text field
        let result = self.input_text(label, buffer)
            .build();
        
        // Add glow effect
        if result {
            // Trigger glow animation
        }
        
        result
    }    fn ragnarek_loading_spinner(&self, radius: f32, thickness: f32, speed: f32) {
        // Animated loading spinner with RAGNAREK styling
        let center = self.cursor_screen_pos();
        let draw_list = self.get_window_draw_list();
        
        // Draw the spinner circles
        for i in 0..3 {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / 3.0;
            let x_offset = radius * angle.cos();
            let y_offset = radius * angle.sin();
            
            draw_list.add_circle(
                [center[0] + x_offset, center[1] + y_offset],
                thickness,
                [0.3, 0.6, 1.0, 1.0]
            ).thickness(thickness).build();
        }
        
        // Add rotation animation
        // (In a real implementation, you'd update the angle based on time and speed)
    }    fn ragnarek_window_header(&self, title: &str, gradient_start: [f32; 4], gradient_end: [f32; 4]) {
        // Enhanced window header with gradient background
        let pos = self.cursor_screen_pos();
        let size = [self.content_region_avail()[0], self.text_line_height() * 2.0];
        let draw_list = self.get_window_draw_list();
        
        // Draw gradient background
        draw_list.add_rect_filled_multicolor(
            pos,
            [pos[0] + size[0], pos[1] + size[1]],
            gradient_start,
            gradient_end,
            gradient_end,
            gradient_start
        );
        
        // Draw title text
        draw_list.add_text(pos, [1.0, 1.0, 1.0, 1.0], title);
        
        self.dummy(size);
    }    fn ragnarek_particle_background(&self, size: [f32; 2], particle_count: u32, color: [f32; 4]) {
        // Particle effect background for sections
        let pos = self.cursor_screen_pos();
        let draw_list = self.get_window_draw_list();
        
        // Get current time for pseudo-random particle placement
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f32;
        
        // Draw particles with deterministic "random" positions based on time
        for i in 0..particle_count {
            let seed = (time * 0.001 + i as f32 * 17.0) % 1000.0;
            let x_offset = (seed.sin() * 0.5 + 0.5) * size[0];
            let y_offset = ((seed + 100.0).cos() * 0.5 + 0.5) * size[1];
            let particle_size = ((seed + 200.0).sin() * 0.5 + 0.5) * 2.0 + 1.0;
            
            draw_list.add_circle(
                [pos[0] + x_offset, pos[1] + y_offset],
                particle_size,
                color
            )
            .filled(true)
            .build();
        }
        
        self.dummy(size);
    }
    
    fn ragnarek_toggle_switch(&self, label: &str, value: &mut bool, size: [f32; 2]) -> bool {
        // Animated toggle switch with RAGNAREK styling
        let bg_color = if *value {
            [0.2, 0.8, 0.2, 1.0] // Green when on
        } else {
            [0.8, 0.2, 0.2, 1.0] // Red when off
        };
        
        let _color_token1 = self.push_style_color(StyleColor::Button, bg_color);
        let _color_token2 = self.push_style_color(StyleColor::ButtonHovered, [0.3, 0.9, 0.3, 1.0]);
        let _color_token3 = self.push_style_color(StyleColor::ButtonActive, [0.1, 0.6, 0.1, 1.0]);
        let _style_token1 = self.push_style_var(StyleVar::FrameRounding(12.0));
        let _style_token2 = self.push_style_var(StyleVar::FramePadding([4.0, 2.0]));
        
        // Create the toggle button
        let result = self.button_with_size(label, size);
        
        // Update value on click
        if result {
            *value = !*value;
        }
        
        result
    }
    
    fn ragnarek_group_box<F>(&self, label: &str, size: [f32; 2], glow_color: [f32; 4], f: F)
    where
        F: FnOnce()
    {
        // Enhanced group box with glow effects
        let _color_token1 = self.push_style_color(StyleColor::ChildBg, [0.1, 0.1, 0.1, 0.9]);
        let _color_token2 = self.push_style_color(StyleColor::Border, glow_color);
        let _style_token1 = self.push_style_var(StyleVar::ChildRounding(8.0));
        let _style_token2 = self.push_style_var(StyleVar::ChildBorderSize(1.0));
        let _style_token3 = self.push_style_var(StyleVar::WindowPadding([8.0, 8.0]));
        
        if let Some(_token) = self.child_window(label)
            .size(size)
            .border(true)
            .begin()
        {
            f();
        }
    }    fn ragnarek_status_indicator(&self, status: &str, color: [f32; 4], pulse: bool) {
        // Status indicator with pulsing effects
        let pos = self.cursor_screen_pos();
        let size = [16.0, 16.0];
        let draw_list = self.get_window_draw_list();
        
        // Draw circle for status
        draw_list.add_circle(pos, size[0] * 0.5, color)
            .filled(true)
            .build();
        
        // Add pulsing animation
        if pulse {
            // (In a real implementation, you'd animate the size and color over time)
        }
        
        self.dummy(size);
    }
}

/// RAGNAREK Theme configuration
pub struct RagnarekTheme {
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
    pub accent_color: [f32; 4],
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
}

impl Default for RagnarekTheme {
    fn default() -> Self {
        Self {
            primary_color: [0.2, 0.4, 0.8, 1.0],     // Blue
            secondary_color: [0.3, 0.3, 0.3, 1.0],   // Gray
            accent_color: [0.2, 0.8, 0.2, 1.0],      // Green
            background_color: [0.1, 0.1, 0.1, 1.0],  // Dark
            text_color: [1.0, 1.0, 1.0, 1.0],        // White
        }
    }
}

impl RagnarekTheme {
    pub fn apply_to_imgui(&self, ui: &Ui) {
        // Apply the RAGNAREK theme colors to ImGui
        let _color_tokens = [
            ui.push_style_color(StyleColor::WindowBg, self.background_color),
            ui.push_style_color(StyleColor::Text, self.text_color),
            ui.push_style_color(StyleColor::Button, self.secondary_color),
            ui.push_style_color(StyleColor::ButtonHovered, self.primary_color),
            ui.push_style_color(StyleColor::ButtonActive, self.accent_color),
        ];
        
        let _style_tokens = [
            ui.push_style_var(StyleVar::WindowRounding(12.0)),
            ui.push_style_var(StyleVar::FrameRounding(6.0)),
            ui.push_style_var(StyleVar::GrabRounding(6.0)),
            ui.push_style_var(StyleVar::WindowPadding([12.0, 12.0])),
            ui.push_style_var(StyleVar::FramePadding([8.0, 6.0])),
        ];
    }
}
