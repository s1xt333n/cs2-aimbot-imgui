#include "wrapper.h"
#include "imgui.h"

// Forward declare the edited namespace functions from our simple implementation
namespace edited {
    bool BeginChild(const char* str_id, const ImVec2& size, ImGuiChildFlags child_flags, ImGuiWindowFlags window_flags);
    void EndChild();
    bool Tab(bool selected, ImGuiID id, const char* icon, const ImVec2& size_arg);
    bool Checkbox(const char* label, bool* v);
    void CheckboxClicked(const char* label, bool* v);
    bool CheckboxPicker(const char* label, bool* v, float col[3], ImGuiColorEditFlags flags);
    bool CheckboxDoublePicker(const char* label, bool* v, float col1[3], float col2[3], ImGuiColorEditFlags flags);
    bool SliderInt(const char* label, int* v, int v_min, int v_max, const char* format, ImGuiSliderFlags flags);
    bool SliderFloat(const char* label, float* v, float v_min, float v_max, const char* format, ImGuiSliderFlags flags);
    bool RangeSliderFloat(const char* label, float* v1, float* v2, float v_min, float v_max, const char* display_format, float power);
    bool ColorEdit4(const char* label, float col[4], ImGuiColorEditFlags flags);
    bool ColorPicker4(const char* label, float col[4], ImGuiColorEditFlags flags, const float* ref_col);
    bool ColorButton(const char* desc_id, const ImVec4& col, ImGuiColorEditFlags flags, const ImVec2& size);
    bool Selectable(const char* label, bool selected, ImGuiSelectableFlags flags, const ImVec2& size);
    bool Selectable(const char* label, bool* p_selected, ImGuiSelectableFlags flags, const ImVec2& size);
    bool BeginCombo(const char* label, const char* preview_value, int val, bool multi, ImGuiComboFlags flags);
    void EndCombo();
    void MultiCombo(const char* label, bool variable[], const char* labels[], int count);
    bool Combo(const char* label, int* current_item, const char* const items[], int items_count, int popup_max_height_in_items);
    void TextCenter(const ImVec2& p_min, const ImVec2& p_max, ImU32 col, const char* text, const ImVec2& align);
    bool Keybind(const char* label, int* key, bool show_label);
    bool icon_box(const char* icon, ImVec2 size, ImU32 color_bg, ImU32 color_icon, ImU32 color_border);
    bool color_button(const char* name, ImVec2 size, ImU32 color_bg);
    void esp_preview(ImTextureID player_preview, bool* nickname, float nick_color[4], bool* weapon, float weapon_color[4], 
                     int* hp, float hp_color[4], bool* zoom, float zoom_color[4], bool* bomb, float bomb_color[4], 
                     bool* c4, float c4_color[4], bool* money, float money_color[4], bool* hit, float hit_color[4], 
                     bool* esp_box, float box_color[4], bool* hp_line, float hp_line_color[4]);
}

extern "C" {
    // Child window functions
    bool ragnarek_begin_child(const char* name, float size_x, float size_y, uint32_t child_flags, uint32_t window_flags) {
        return edited::BeginChild(name, ImVec2(size_x, size_y), child_flags, window_flags);
    }
    
    void ragnarek_end_child() {
        edited::EndChild();
    }
    
    // Tab function
    bool ragnarek_tab(bool selected, uint32_t id, const char* icon, float size_x, float size_y) {
        return edited::Tab(selected, id, icon, ImVec2(size_x, size_y));
    }
    
    // Checkbox functions
    bool ragnarek_checkbox(const char* label, bool* value) {
        return edited::Checkbox(label, value);
    }
    
    void ragnarek_checkbox_clicked(const char* label, bool* value) {
        edited::CheckboxClicked(label, value);
    }
    
    bool ragnarek_checkbox_picker(const char* label, bool* value, float color[3], uint32_t flags) {
        return edited::CheckboxPicker(label, value, color, flags);
    }
    
    bool ragnarek_checkbox_double_picker(const char* label, bool* value, float color1[3], float color2[3], uint32_t flags) {
        return edited::CheckboxDoublePicker(label, value, color1, color2, flags);
    }
    
    // Slider functions
    bool ragnarek_slider_int(const char* label, int* value, int min, int max, const char* format, uint32_t flags) {
        return edited::SliderInt(label, value, min, max, format, flags);
    }
    
    bool ragnarek_slider_float(const char* label, float* value, float min, float max, const char* format, uint32_t flags) {
        return edited::SliderFloat(label, value, min, max, format, flags);
    }
    
    bool ragnarek_range_slider_float(const char* label, float* v1, float* v2, float min, float max, const char* format, float power) {
        return edited::RangeSliderFloat(label, v1, v2, min, max, format, power);
    }
    
    // Color functions
    bool ragnarek_color_edit4(const char* label, float color[4], uint32_t flags) {
        return edited::ColorEdit4(label, color, flags);
    }
    
    bool ragnarek_color_picker4(const char* label, float color[4], uint32_t flags, const float* ref_color) {
        return edited::ColorPicker4(label, color, flags, ref_color);
    }
    
    bool ragnarek_color_button(const char* desc_id, float color[4], uint32_t flags, float size_x, float size_y) {
        return edited::ColorButton(desc_id, ImVec4(color[0], color[1], color[2], color[3]), flags, ImVec2(size_x, size_y));
    }
    
    // Selectable functions
    bool ragnarek_selectable(const char* label, bool selected, uint32_t flags, float size_x, float size_y) {
        return edited::Selectable(label, selected, flags, ImVec2(size_x, size_y));
    }
    
    bool ragnarek_selectable_ptr(const char* label, bool* selected, uint32_t flags, float size_x, float size_y) {
        return edited::Selectable(label, selected, flags, ImVec2(size_x, size_y));
    }
    
    // Combo functions
    bool ragnarek_begin_combo(const char* label, const char* preview_value, int val, bool multi, uint32_t flags) {
        return edited::BeginCombo(label, preview_value, val, multi, flags);
    }
    
    void ragnarek_end_combo() {
        edited::EndCombo();
    }
    
    void ragnarek_multi_combo(const char* label, bool variables[], const char* labels[], int count) {
        edited::MultiCombo(label, variables, labels, count);
    }
    
    bool ragnarek_combo_array(const char* label, int* current_item, const char* const items[], int items_count, int popup_max_height) {
        return edited::Combo(label, current_item, items, items_count, popup_max_height);
    }
    
    // Keybind function
    bool ragnarek_keybind(const char* label, int* key, bool show_label) {
        return edited::Keybind(label, key, show_label);
    }
    
    // Utility functions
    void ragnarek_text_center(float p_min_x, float p_min_y, float p_max_x, float p_max_y, uint32_t color, const char* text, float align_x, float align_y) {
        edited::TextCenter(ImVec2(p_min_x, p_min_y), ImVec2(p_max_x, p_max_y), color, text, ImVec2(align_x, align_y));
    }
    
    bool ragnarek_icon_box(const char* icon, float size_x, float size_y, uint32_t color_bg, uint32_t color_icon, uint32_t color_border) {
        return edited::icon_box(icon, ImVec2(size_x, size_y), color_bg, color_icon, color_border);
    }
    
    bool ragnarek_color_button_simple(const char* name, float size_x, float size_y, uint32_t color_bg) {
        return edited::color_button(name, ImVec2(size_x, size_y), color_bg);
    }
    
    // ESP preview function
    void ragnarek_esp_preview(void* player_preview, bool* nickname, float nick_color[4], bool* weapon, float weapon_color[4], 
                              int* hp, float hp_color[4], bool* zoom, float zoom_color[4], bool* bomb, float bomb_color[4], 
                              bool* c4, float c4_color[4], bool* money, float money_color[4], bool* hit, float hit_color[4], 
                              bool* esp_box, float box_color[4], bool* hp_line, float hp_line_color[4]) {
        edited::esp_preview((ImTextureID)player_preview, nickname, nick_color, weapon, weapon_color, hp, hp_color, 
                          zoom, zoom_color, bomb, bomb_color, c4, c4_color, money, money_color, hit, hit_color, 
                          esp_box, box_color, hp_line, hp_line_color);
    }
}
