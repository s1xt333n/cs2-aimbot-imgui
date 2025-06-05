#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Child window functions
bool ragnarek_begin_child(const char* name, float size_x, float size_y, uint32_t child_flags, uint32_t window_flags);
void ragnarek_end_child();

// Tab function
bool ragnarek_tab(bool selected, uint32_t id, const char* icon, float size_x, float size_y);

// Checkbox functions
bool ragnarek_checkbox(const char* label, bool* value);
void ragnarek_checkbox_clicked(const char* label, bool* value);
bool ragnarek_checkbox_picker(const char* label, bool* value, float color[3], uint32_t flags);
bool ragnarek_checkbox_double_picker(const char* label, bool* value, float color1[3], float color2[3], uint32_t flags);

// Slider functions
bool ragnarek_slider_int(const char* label, int* value, int min, int max, const char* format, uint32_t flags);
bool ragnarek_slider_float(const char* label, float* value, float min, float max, const char* format, uint32_t flags);
bool ragnarek_range_slider_float(const char* label, float* v1, float* v2, float min, float max, const char* format, float power);

// Color functions
bool ragnarek_color_edit4(const char* label, float color[4], uint32_t flags);
bool ragnarek_color_picker4(const char* label, float color[4], uint32_t flags, const float* ref_color);
bool ragnarek_color_button(const char* desc_id, float color[4], uint32_t flags, float size_x, float size_y);

// Selectable functions
bool ragnarek_selectable(const char* label, bool selected, uint32_t flags, float size_x, float size_y);
bool ragnarek_selectable_ptr(const char* label, bool* selected, uint32_t flags, float size_x, float size_y);

// Combo functions
bool ragnarek_begin_combo(const char* label, const char* preview_value, int val, bool multi, uint32_t flags);
void ragnarek_end_combo();
void ragnarek_multi_combo(const char* label, bool variables[], const char* labels[], int count);
bool ragnarek_combo_array(const char* label, int* current_item, const char* const items[], int items_count, int popup_max_height);

// Keybind function
bool ragnarek_keybind(const char* label, int* key, bool show_label);

// Utility functions
void ragnarek_text_center(float p_min_x, float p_min_y, float p_max_x, float p_max_y, uint32_t color, const char* text, float align_x, float align_y);
bool ragnarek_icon_box(const char* icon, float size_x, float size_y, uint32_t color_bg, uint32_t color_icon, uint32_t color_border);
bool ragnarek_color_button_simple(const char* name, float size_x, float size_y, uint32_t color_bg);

// ESP preview function
void ragnarek_esp_preview(void* player_preview, bool* nickname, float nick_color[4], bool* weapon, float weapon_color[4], 
                          int* hp, float hp_color[4], bool* zoom, float zoom_color[4], bool* bomb, float bomb_color[4], 
                          bool* c4, float c4_color[4], bool* money, float money_color[4], bool* hit, float hit_color[4], 
                          bool* esp_box, float box_color[4], bool* hp_line, float hp_line_color[4]);

#ifdef __cplusplus
}
#endif
