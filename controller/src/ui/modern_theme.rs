use imgui::{StyleColor, StyleVar, Ui};

/// Modern dark theme for ImGui
pub struct ModernTheme {
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
    pub accent_color: [f32; 4],
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub border_color: [f32; 4],
    pub success_color: [f32; 4],
    pub warning_color: [f32; 4],
    pub error_color: [f32; 4],
}

impl Default for ModernTheme {
    fn default() -> Self {
        Self {
            primary_color: [0.26, 0.59, 0.98, 1.0],      // Blue
            secondary_color: [0.4, 0.4, 0.4, 1.0],       // Gray
            accent_color: [0.0, 0.7, 0.4, 1.0],          // Green
            background_color: [0.1, 0.1, 0.1, 1.0],      // Dark gray
            text_color: [0.9, 0.9, 0.9, 1.0],            // Light gray
            border_color: [0.3, 0.3, 0.3, 1.0],          // Medium gray
            success_color: [0.0, 0.8, 0.4, 1.0],         // Green
            warning_color: [1.0, 0.6, 0.0, 1.0],         // Orange
            error_color: [0.9, 0.2, 0.2, 1.0],           // Red
        }
    }
}

impl ModernTheme {
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Applies the modern theme to ImGui
    pub fn apply(&self, ui: &Ui) {
        let style = ui.style_mut();
        
        // Colors
        style.colors[StyleColor::Text as usize] = self.text_color;
        style.colors[StyleColor::TextDisabled as usize] = [0.5, 0.5, 0.5, 1.0];
        
        // Window colors
        style.colors[StyleColor::WindowBg as usize] = [0.12, 0.12, 0.12, 0.94];
        style.colors[StyleColor::ChildBg as usize] = [0.08, 0.08, 0.08, 0.0];
        style.colors[StyleColor::PopupBg as usize] = [0.08, 0.08, 0.08, 0.94];
        
        // Border colors
        style.colors[StyleColor::Border as usize] = self.border_color;
        style.colors[StyleColor::BorderShadow as usize] = [0.0, 0.0, 0.0, 0.0];
        
        // Frame colors (buttons, sliders, etc.)
        style.colors[StyleColor::FrameBg as usize] = [0.16, 0.16, 0.16, 0.54];
        style.colors[StyleColor::FrameBgHovered as usize] = [0.26, 0.26, 0.26, 0.4];
        style.colors[StyleColor::FrameBgActive as usize] = [0.26, 0.26, 0.26, 0.67];
        
        // Title bar
        style.colors[StyleColor::TitleBg as usize] = [0.04, 0.04, 0.04, 1.0];
        style.colors[StyleColor::TitleBgActive as usize] = [0.16, 0.16, 0.16, 1.0];
        style.colors[StyleColor::TitleBgCollapsed as usize] = [0.0, 0.0, 0.0, 0.51];
        
        // Menu bar
        style.colors[StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.0];
        
        // Scrollbar
        style.colors[StyleColor::ScrollbarBg as usize] = [0.02, 0.02, 0.02, 0.53];
        style.colors[StyleColor::ScrollbarGrab as usize] = [0.31, 0.31, 0.31, 1.0];
        style.colors[StyleColor::ScrollbarGrabHovered as usize] = [0.41, 0.41, 0.41, 1.0];
        style.colors[StyleColor::ScrollbarGrabActive as usize] = [0.51, 0.51, 0.51, 1.0];
        
        // Check mark
        style.colors[StyleColor::CheckMark as usize] = self.primary_color;
        
        // Slider
        style.colors[StyleColor::SliderGrab as usize] = self.primary_color;
        style.colors[StyleColor::SliderGrabActive as usize] = [
            self.primary_color[0],
            self.primary_color[1],
            self.primary_color[2],
            0.7,
        ];
        
        // Button
        style.colors[StyleColor::Button as usize] = [0.2, 0.2, 0.2, 0.4];
        style.colors[StyleColor::ButtonHovered as usize] = self.primary_color;
        style.colors[StyleColor::ButtonActive as usize] = [
            self.primary_color[0] * 0.8,
            self.primary_color[1] * 0.8,
            self.primary_color[2] * 0.8,
            1.0,
        ];
        
        // Header
        style.colors[StyleColor::Header as usize] = [0.2, 0.2, 0.2, 0.31];
        style.colors[StyleColor::HeaderHovered as usize] = [0.2, 0.2, 0.2, 0.8];
        style.colors[StyleColor::HeaderActive as usize] = [0.2, 0.2, 0.2, 1.0];
        
        // Separator
        style.colors[StyleColor::Separator as usize] = self.border_color;
        style.colors[StyleColor::SeparatorHovered as usize] = [0.1, 0.4, 0.75, 0.78];
        style.colors[StyleColor::SeparatorActive as usize] = [0.1, 0.4, 0.75, 1.0];
        
        // Resize grip
        style.colors[StyleColor::ResizeGrip as usize] = [0.26, 0.59, 0.98, 0.25];
        style.colors[StyleColor::ResizeGripHovered as usize] = [0.26, 0.59, 0.98, 0.67];
        style.colors[StyleColor::ResizeGripActive as usize] = [0.26, 0.59, 0.98, 0.95];
        
        // Tab
        style.colors[StyleColor::Tab as usize] = [0.18, 0.18, 0.18, 0.86];
        style.colors[StyleColor::TabHovered as usize] = self.primary_color;
        style.colors[StyleColor::TabActive as usize] = [0.2, 0.2, 0.2, 1.0];
        style.colors[StyleColor::TabUnfocused as usize] = [0.07, 0.1, 0.15, 0.97];
        style.colors[StyleColor::TabUnfocusedActive as usize] = [0.14, 0.14, 0.14, 1.0];
        
        // Plot
        style.colors[StyleColor::PlotLines as usize] = [0.61, 0.61, 0.61, 1.0];
        style.colors[StyleColor::PlotLinesHovered as usize] = [1.0, 0.43, 0.35, 1.0];
        style.colors[StyleColor::PlotHistogram as usize] = [0.9, 0.7, 0.0, 1.0];
        style.colors[StyleColor::PlotHistogramHovered as usize] = [1.0, 0.6, 0.0, 1.0];
        
        // Text selection
        style.colors[StyleColor::TextSelectedBg as usize] = [0.26, 0.59, 0.98, 0.35];
        
        // Drag drop
        style.colors[StyleColor::DragDropTarget as usize] = [1.0, 1.0, 0.0, 0.9];
        
        // Navigation
        style.colors[StyleColor::NavHighlight as usize] = [0.26, 0.59, 0.98, 1.0];
        style.colors[StyleColor::NavWindowingHighlight as usize] = [1.0, 1.0, 1.0, 0.7];
        style.colors[StyleColor::NavWindowingDimBg as usize] = [0.8, 0.8, 0.8, 0.2];
        
        // Modal
        style.colors[StyleColor::ModalWindowDimBg as usize] = [0.8, 0.8, 0.8, 0.35];
        
        // Rounded corners and modern styling
        style.window_rounding = 6.0;
        style.child_rounding = 6.0;
        style.frame_rounding = 4.0;
        style.popup_rounding = 4.0;
        style.scrollbar_rounding = 9.0;
        style.grab_rounding = 3.0;
        style.tab_rounding = 4.0;
        
        // Borders
        style.window_border_size = 1.0;
        style.child_border_size = 1.0;
        style.popup_border_size = 1.0;
        style.frame_border_size = 0.0;
        style.tab_border_size = 0.0;
        
        // Spacing
        style.window_padding = [8.0, 8.0];
        style.frame_padding = [5.0, 2.0];
        style.item_spacing = [6.0, 4.0];
        style.item_inner_spacing = [6.0, 6.0];
        style.indent_spacing = 25.0;
        style.scrollbar_size = 15.0;
        style.grab_min_size = 10.0;
        
        // Alignment
        style.window_title_align = [0.0, 0.5];
        style.button_text_align = [0.5, 0.5];
    }
    
    /// Creates a styled button with custom colors
    pub fn styled_button(&self, ui: &Ui, label: &str, size: [f32; 2], button_type: ButtonType) -> bool {
        let colors = match button_type {
            ButtonType::Primary => [
                [self.primary_color[0], self.primary_color[1], self.primary_color[2], 0.6],
                self.primary_color,
                [self.primary_color[0] * 0.8, self.primary_color[1] * 0.8, self.primary_color[2] * 0.8, 1.0],
            ],
            ButtonType::Success => [
                [self.success_color[0], self.success_color[1], self.success_color[2], 0.6],
                self.success_color,
                [self.success_color[0] * 0.8, self.success_color[1] * 0.8, self.success_color[2] * 0.8, 1.0],
            ],
            ButtonType::Warning => [
                [self.warning_color[0], self.warning_color[1], self.warning_color[2], 0.6],
                self.warning_color,
                [self.warning_color[0] * 0.8, self.warning_color[1] * 0.8, self.warning_color[2] * 0.8, 1.0],
            ],
            ButtonType::Danger => [
                [self.error_color[0], self.error_color[1], self.error_color[2], 0.6],
                self.error_color,
                [self.error_color[0] * 0.8, self.error_color[1] * 0.8, self.error_color[2] * 0.8, 1.0],
            ],
            ButtonType::Secondary => [
                [self.secondary_color[0], self.secondary_color[1], self.secondary_color[2], 0.6],
                self.secondary_color,
                [self.secondary_color[0] * 0.8, self.secondary_color[1] * 0.8, self.secondary_color[2] * 0.8, 1.0],
            ],
        };
        
        let _style = ui.push_style_colors(&[
            (StyleColor::Button, colors[0]),
            (StyleColor::ButtonHovered, colors[1]),
            (StyleColor::ButtonActive, colors[2]),
        ]);
        
        ui.button_with_size(label, size)
    }
    
    /// Creates a styled collapsing header
    pub fn styled_collapsing_header(&self, ui: &Ui, label: &str, default_open: bool) -> bool {
        let _style = ui.push_style_colors(&[
            (StyleColor::Header, [0.2, 0.2, 0.2, 0.4]),
            (StyleColor::HeaderHovered, [0.3, 0.3, 0.3, 0.8]),
            (StyleColor::HeaderActive, self.primary_color),
        ]);
        
        ui.collapsing_header(label, if default_open { 
            imgui::TreeNodeFlags::DEFAULT_OPEN 
        } else { 
            imgui::TreeNodeFlags::empty() 
        })
    }
    
    /// Creates a status indicator
    pub fn status_indicator(&self, ui: &Ui, label: &str, status: StatusType) {
        let color = match status {
            StatusType::Active => self.success_color,
            StatusType::Inactive => self.secondary_color,
            StatusType::Warning => self.warning_color,
            StatusType::Error => self.error_color,
        };
        
        let draw_list = ui.get_window_draw_list();
        let cursor_pos = ui.cursor_screen_pos();
        
        // Draw status circle
        draw_list.add_circle(
            [cursor_pos[0] + 6.0, cursor_pos[1] + ui.text_line_height() * 0.5],
            4.0,
            color,
        ).filled(true).build();
        
        // Add some spacing and draw the label
        ui.dummy([16.0, 0.0]);
        ui.same_line();
        ui.text(label);
    }
    
    /// Creates a progress bar with custom styling
    pub fn styled_progress_bar(&self, ui: &Ui, fraction: f32, size: [f32; 2], label: Option<&str>) {
        let _style = ui.push_style_colors(&[
            (StyleColor::PlotHistogram, self.primary_color),
            (StyleColor::FrameBg, [0.1, 0.1, 0.1, 0.8]),
        ]);
        
        ui.progress_bar(fraction)
            .size(size)
            .overlay_text(label.unwrap_or(""))
            .build();
    }
    
    /// Creates a tooltip with modern styling
    pub fn styled_tooltip(&self, ui: &Ui, text: &str) {
        if ui.is_item_hovered() {
            let _style = ui.push_style_colors(&[
                (StyleColor::PopupBg, [0.08, 0.08, 0.08, 0.95]),
                (StyleColor::Border, self.border_color),
            ]);
            
            ui.tooltip_text(text);
        }
    }
    
    /// Creates a modern tab bar
    pub fn begin_tab_bar(&self, ui: &Ui, str_id: &str) -> Option<imgui::TabBarToken> {
        let _style = ui.push_style_colors(&[
            (StyleColor::Tab, [0.15, 0.15, 0.15, 0.86]),
            (StyleColor::TabHovered, [0.25, 0.25, 0.25, 0.8]),
            (StyleColor::TabActive, [0.2, 0.2, 0.2, 1.0]),
            (StyleColor::TabUnfocused, [0.1, 0.1, 0.1, 0.97]),
            (StyleColor::TabUnfocusedActive, [0.15, 0.15, 0.15, 1.0]),
        ]);
        
        ui.tab_bar(str_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusType {
    Active,
    Inactive,
    Warning,
    Error,
}

/// Modern UI components and utilities
pub struct ModernUI;

impl ModernUI {
    /// Creates a modern settings section
    pub fn settings_section<F>(ui: &Ui, title: &str, theme: &ModernTheme, content: F) 
    where
        F: FnOnce(&Ui),
    {
        let _style = ui.push_style_var(StyleVar::ChildRounding(6.0));
        
        ui.child_window(title)
            .size([0.0, 0.0])
            .border(true)
            .build(|| {
                // Section header
                let _header_style = ui.push_style_colors(&[
                    (StyleColor::Text, theme.primary_color),
                ]);
                
                ui.text(title);
                ui.separator();
                ui.spacing();
                
                content(ui);
            });
    }
    
    /// Creates a key binding input
    pub fn key_binding_input(ui: &Ui, label: &str, current_key: &mut Option<imgui::Key>) -> bool {
        ui.text(label);
        ui.same_line();
        
        let key_text = if let Some(key) = current_key {
            format!("{:?}", key)
        } else {
            "None".to_string()
        };
        
        if ui.button(&key_text) {
            // In a real implementation, this would open a key capture dialog
            return true;
        }
        
        false
    }
    
    /// Creates a slider with value display
    pub fn value_slider<T>(ui: &Ui, label: &str, value: &mut T, min: T, max: T) -> bool 
    where
        T: Copy + PartialOrd + std::fmt::Display,
        for<'a> imgui::Slider<'a, T>: imgui::SliderFlags<T>,
    {
        let mut changed = false;
        
        ui.text(label);
        ui.same_line_with_spacing(150.0, 0.0);
        
        ui.set_next_item_width(200.0);
        if ui.slider(&format!("##{}", label), value, min, max) {
            changed = true;
        }
        
        ui.same_line();
        ui.text(&format!("{}", value));
        
        changed
    }
    
    /// Creates a performance metrics display
    pub fn performance_metrics(ui: &Ui, theme: &ModernTheme, fps: f32, frame_time: f32) {
        ui.text("Performance");
        ui.separator();
        
        // FPS display
        let fps_color = if fps > 60.0 {
            theme.success_color
        } else if fps > 30.0 {
            theme.warning_color
        } else {
            theme.error_color
        };
        
        let _fps_style = ui.push_style_color(StyleColor::Text, fps_color);
        ui.text(&format!("FPS: {:.1}", fps));
        
        // Frame time
        ui.text(&format!("Frame time: {:.2}ms", frame_time));
    }
}