use std::os::raw::{c_char, c_float, c_int};
use std::ffi::CString;

// External FFI function declarations
extern "C" {
    fn ragnarek_begin_child(name: *const c_char, size_x: c_float, size_y: c_float, child_flags: u32, window_flags: u32) -> bool;
    fn ragnarek_end_child();
    fn ragnarek_tab(selected: bool, id: u32, icon: *const c_char, size_x: c_float, size_y: c_float) -> bool;
    fn ragnarek_checkbox(label: *const c_char, value: *mut bool) -> bool;
    fn ragnarek_checkbox_clicked(label: *const c_char, value: *mut bool);
    fn ragnarek_checkbox_picker(label: *const c_char, value: *mut bool, color: *mut c_float, flags: u32) -> bool;
    fn ragnarek_slider_int(label: *const c_char, value: *mut c_int, min: c_int, max: c_int, format: *const c_char, flags: u32) -> bool;
    fn ragnarek_slider_float(label: *const c_char, value: *mut c_float, min: c_float, max: c_float, format: *const c_char, flags: u32) -> bool;
    fn ragnarek_color_edit4(label: *const c_char, color: *mut c_float, flags: u32) -> bool;
    fn ragnarek_color_picker4(label: *const c_char, color: *mut c_float, flags: u32, ref_color: *const c_float) -> bool;
    fn ragnarek_selectable(label: *const c_char, selected: bool, flags: u32, size_x: c_float, size_y: c_float) -> bool;
    fn ragnarek_selectable_ptr(label: *const c_char, selected: *mut bool, flags: u32, size_x: c_float, size_y: c_float) -> bool;
    fn ragnarek_begin_combo(label: *const c_char, preview_value: *const c_char, val: c_int, multi: bool, flags: u32) -> bool;
    fn ragnarek_end_combo();
    fn ragnarek_keybind(label: *const c_char, key: *mut c_int, show_label: bool) -> bool;
}

pub mod ragnarek {
    use super::*;
    
    // Safe Rust wrappers for the RAGNAREK ImGui functions
    pub struct RagnarekContext {
        _phantom: std::marker::PhantomData<()>,
    }
    
    impl RagnarekContext {
        pub fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
        
        // Wrapper for edited::BeginChild
        pub fn begin_child(&self, name: &str, size: [f32; 2], child_flags: u32, window_flags: u32) -> bool {
            let c_name = CString::new(name).unwrap();
            unsafe {
                ragnarek_begin_child(c_name.as_ptr(), size[0], size[1], child_flags, window_flags)
            }
        }
        
        // Wrapper for edited::EndChild
        pub fn end_child(&self) {
            unsafe {
                ragnarek_end_child();
            }
        }
        
        // Wrapper for edited::Tab
        pub fn tab(&self, selected: bool, id: u32, icon: &str, size: [f32; 2]) -> bool {
            let c_icon = CString::new(icon).unwrap();
            unsafe {
                ragnarek_tab(selected, id, c_icon.as_ptr(), size[0], size[1])
            }
        }
        
        // Wrapper for edited::Checkbox
        pub fn checkbox(&self, label: &str, value: &mut bool) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_checkbox(c_label.as_ptr(), value as *mut bool)
            }
        }
        
        // Wrapper for edited::CheckboxClicked
        pub fn checkbox_clicked(&self, label: &str, value: &mut bool) {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_checkbox_clicked(c_label.as_ptr(), value as *mut bool);
            }
        }
        
        // Wrapper for edited::CheckboxPicker
        pub fn checkbox_picker(&self, label: &str, value: &mut bool, color: &mut [f32; 3], flags: u32) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_checkbox_picker(c_label.as_ptr(), value as *mut bool, color.as_mut_ptr(), flags)
            }
        }
        
        // Wrapper for edited::SliderInt
        pub fn slider_int(&self, label: &str, value: &mut i32, min: i32, max: i32, format: &str, flags: u32) -> bool {
            let c_label = CString::new(label).unwrap();
            let c_format = CString::new(format).unwrap();
            unsafe {
                ragnarek_slider_int(c_label.as_ptr(), value as *mut c_int, min, max, c_format.as_ptr(), flags)
            }
        }
        
        // Wrapper for edited::SliderFloat
        pub fn slider_float(&self, label: &str, value: &mut f32, min: f32, max: f32, format: &str, flags: u32) -> bool {
            let c_label = CString::new(label).unwrap();
            let c_format = CString::new(format).unwrap();
            unsafe {
                ragnarek_slider_float(c_label.as_ptr(), value as *mut c_float, min, max, c_format.as_ptr(), flags)
            }
        }
        
        // Wrapper for edited::ColorEdit4
        pub fn color_edit4(&self, label: &str, color: &mut [f32; 4], flags: u32) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_color_edit4(c_label.as_ptr(), color.as_mut_ptr(), flags)
            }
        }
        
        // Wrapper for edited::ColorPicker4
        pub fn color_picker4(&self, label: &str, color: &mut [f32; 4], flags: u32, ref_color: Option<&[f32; 4]>) -> bool {
            let c_label = CString::new(label).unwrap();
            let ref_ptr = ref_color.map_or(std::ptr::null(), |r| r.as_ptr());
            unsafe {
                ragnarek_color_picker4(c_label.as_ptr(), color.as_mut_ptr(), flags, ref_ptr)
            }
        }
        
        // Wrapper for edited::Selectable
        pub fn selectable(&self, label: &str, selected: bool, flags: u32, size: [f32; 2]) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_selectable(c_label.as_ptr(), selected, flags, size[0], size[1])
            }
        }
        
        // Wrapper for edited::Selectable with pointer
        pub fn selectable_ptr(&self, label: &str, selected: &mut bool, flags: u32, size: [f32; 2]) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_selectable_ptr(c_label.as_ptr(), selected as *mut bool, flags, size[0], size[1])
            }
        }
        
        // Wrapper for edited::BeginCombo
        pub fn begin_combo(&self, label: &str, preview_value: &str, val: i32, multi: bool, flags: u32) -> bool {
            let c_label = CString::new(label).unwrap();
            let c_preview = CString::new(preview_value).unwrap();
            unsafe {
                ragnarek_begin_combo(c_label.as_ptr(), c_preview.as_ptr(), val, multi, flags)
            }
        }
        
        // Wrapper for edited::EndCombo
        pub fn end_combo(&self) {
            unsafe {
                ragnarek_end_combo();
            }
        }
        
        // Wrapper for edited::Keybind
        pub fn keybind(&self, label: &str, key: &mut i32, show_label: bool) -> bool {
            let c_label = CString::new(label).unwrap();
            unsafe {
                ragnarek_keybind(c_label.as_ptr(), key as *mut c_int, show_label)
            }
        }
    }
}

// Re-export the main types
pub use ragnarek::*;
