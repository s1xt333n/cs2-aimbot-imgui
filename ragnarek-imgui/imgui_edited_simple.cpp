#include "imgui.h"
#include "imgui_internal.h"
#include <cstring>

// Simple implementation of RAGNAREK functions without D3D11 dependencies
namespace edited 
{
    bool BeginChild(const char* str_id, const ImVec2& size, ImGuiChildFlags child_flags, ImGuiWindowFlags window_flags) {
        return ImGui::BeginChild(str_id, size, child_flags, window_flags);
    }

    void EndChild() {
        ImGui::EndChild();
    }

    bool Tab(bool selected, ImGuiID id, const char* icon, const ImVec2& size_arg) {
        // Simple tab implementation using button
        ImGuiContext& g = *GImGui;
        ImGuiWindow* window = g.CurrentWindow;
        if (window->SkipItems)
            return false;

        const ImGuiStyle& style = g.Style;
        ImGuiID buttonId = window->GetID(id);
        
        ImVec2 pos = window->DC.CursorPos;
        ImRect bb(pos, pos + size_arg);

        ImGui::ItemSize(bb);
        if (!ImGui::ItemAdd(bb, buttonId))
            return false;

        bool hovered, held;
        bool pressed = ImGui::ButtonBehavior(bb, buttonId, &hovered, &held, ImGuiButtonFlags_None);

        // Render
        ImU32 col = ImGui::GetColorU32(selected ? ImGuiCol_ButtonActive : (hovered ? ImGuiCol_ButtonHovered : ImGuiCol_Button));
        ImGui::RenderNavHighlight(bb, buttonId);
        ImGui::RenderFrame(bb.Min, bb.Max, col, true, style.FrameRounding);

        if (icon && icon[0]) {
            ImVec2 text_size = ImGui::CalcTextSize(icon);
            ImVec2 text_pos = bb.Min + (bb.GetSize() - text_size) * 0.5f;
            ImGui::RenderText(text_pos, icon);
        }

        return pressed;
    }

    bool Checkbox(const char* label, bool* v) {
        return ImGui::Checkbox(label, v);
    }

    void CheckboxClicked(const char* label, bool* v) {
        if (ImGui::Checkbox(label, v)) {
            // Handle click
        }
    }

    bool CheckboxPicker(const char* label, bool* v, float col[3], ImGuiColorEditFlags flags) {
        bool changed = false;
        
        changed |= ImGui::Checkbox(label, v);
        ImGui::SameLine();
        
        float color_with_alpha[4] = { col[0], col[1], col[2], 1.0f };
        changed |= ImGui::ColorEdit3("##color", col, flags);
        
        return changed;
    }

    bool CheckboxDoublePicker(const char* label, bool* v, float col1[3], float col2[3], ImGuiColorEditFlags flags) {
        bool changed = false;
        
        changed |= ImGui::Checkbox(label, v);
        ImGui::SameLine();
        changed |= ImGui::ColorEdit3("##color1", col1, flags);
        ImGui::SameLine();
        changed |= ImGui::ColorEdit3("##color2", col2, flags);
        
        return changed;
    }

    bool SliderInt(const char* label, int* v, int v_min, int v_max, const char* format, ImGuiSliderFlags flags) {
        return ImGui::SliderInt(label, v, v_min, v_max, format, flags);
    }

    bool SliderFloat(const char* label, float* v, float v_min, float v_max, const char* format, ImGuiSliderFlags flags) {
        return ImGui::SliderFloat(label, v, v_min, v_max, format, flags);
    }

    bool RangeSliderFloat(const char* label, float* v1, float* v2, float v_min, float v_max, const char* display_format, float power) {
        bool changed = false;
        ImGui::Text("%s", label);
        changed |= ImGui::SliderFloat("##min", v1, v_min, v_max, display_format);
        ImGui::SameLine();
        changed |= ImGui::SliderFloat("##max", v2, v_min, v_max, display_format);
        return changed;
    }

    bool ColorEdit4(const char* label, float col[4], ImGuiColorEditFlags flags) {
        return ImGui::ColorEdit4(label, col, flags);
    }

    bool ColorPicker4(const char* label, float col[4], ImGuiColorEditFlags flags, const float* ref_col) {
        return ImGui::ColorPicker4(label, col, flags, ref_col);
    }

    bool ColorButton(const char* desc_id, const ImVec4& col, ImGuiColorEditFlags flags, const ImVec2& size) {
        return ImGui::ColorButton(desc_id, col, flags, size);
    }

    bool Selectable(const char* label, bool selected, ImGuiSelectableFlags flags, const ImVec2& size) {
        return ImGui::Selectable(label, selected, flags, size);
    }

    bool Selectable(const char* label, bool* p_selected, ImGuiSelectableFlags flags, const ImVec2& size) {
        return ImGui::Selectable(label, p_selected, flags, size);
    }

    bool BeginCombo(const char* label, const char* preview_value, int val, bool multi, ImGuiComboFlags flags) {
        return ImGui::BeginCombo(label, preview_value, flags);
    }

    void EndCombo() {
        ImGui::EndCombo();
    }

    void MultiCombo(const char* label, bool variable[], const char* labels[], int count) {
        if (ImGui::BeginCombo(label, "Multi Select")) {
            for (int i = 0; i < count; i++) {
                ImGui::Selectable(labels[i], &variable[i], ImGuiSelectableFlags_DontClosePopups);
            }
            ImGui::EndCombo();
        }
    }

    bool Combo(const char* label, int* current_item, const char* const items[], int items_count, int popup_max_height_in_items) {
        return ImGui::Combo(label, current_item, items, items_count, popup_max_height_in_items);
    }

    void TextCenter(const ImVec2& p_min, const ImVec2& p_max, ImU32 col, const char* text, const ImVec2& align) {
        ImGui::PushStyleColor(ImGuiCol_Text, col);
        ImGui::RenderTextClipped(p_min, p_max, text, NULL, NULL, align, NULL);
        ImGui::PopStyleColor();
    }

    bool Keybind(const char* label, int* key, bool show_label) {
        bool changed = false;
        
        if (show_label) {
            ImGui::Text("%s", label);
            ImGui::SameLine();
        }
        
        char key_name[32];
        if (*key == 0) {
            strcpy(key_name, "None");
        } else if (*key == 1) {
            strcpy(key_name, "LMB");
        } else if (*key == 2) {
            strcpy(key_name, "RMB");
        } else if (*key >= 65 && *key <= 90) {
            sprintf(key_name, "%c", *key);
        } else {
            sprintf(key_name, "Key %d", *key);
        }
        
        if (ImGui::Button(key_name)) {
            // In a real implementation, you'd capture the next key press
            changed = true;
        }
        
        return changed;
    }

    bool icon_box(const char* icon, ImVec2 size, ImU32 color_bg, ImU32 color_icon, ImU32 color_border) {
        ImGuiWindow* window = ImGui::GetCurrentWindow();
        if (window->SkipItems)
            return false;

        ImVec2 pos = window->DC.CursorPos;
        ImRect bb(pos, pos + size);
        
        ImGui::ItemSize(bb);
        if (!ImGui::ItemAdd(bb, 0))
            return false;

        bool hovered, held;
        bool pressed = ImGui::ButtonBehavior(bb, ImGui::GetID(icon), &hovered, &held);

        // Draw background
        ImGui::GetWindowDrawList()->AddRectFilled(bb.Min, bb.Max, color_bg);
        
        // Draw border
        ImGui::GetWindowDrawList()->AddRect(bb.Min, bb.Max, color_border);
        
        // Draw icon text
        if (icon && icon[0]) {
            ImVec2 text_size = ImGui::CalcTextSize(icon);
            ImVec2 text_pos = bb.Min + (bb.GetSize() - text_size) * 0.5f;
            ImGui::GetWindowDrawList()->AddText(text_pos, color_icon, icon);
        }

        return pressed;
    }

    bool color_button(const char* name, ImVec2 size, ImU32 color_bg) {
        return icon_box(name, size, color_bg, IM_COL32_WHITE, IM_COL32_BLACK);
    }

    void esp_preview(ImTextureID player_preview, bool* nickname, float nick_color[4], bool* weapon, float weapon_color[4], 
                     int* hp, float hp_color[4], bool* zoom, float zoom_color[4], bool* bomb, float bomb_color[4], 
                     bool* c4, float c4_color[4], bool* money, float money_color[4], bool* hit, float hit_color[4], 
                     bool* esp_box, float box_color[4], bool* hp_line, float hp_line_color[4]) {
        // Simple ESP preview implementation
        ImGui::BeginChild("ESP Preview", ImVec2(200, 300), true);
        
        ImGui::Text("ESP Preview");
        ImGui::Separator();
        
        if (nickname && *nickname) {
            ImGui::TextColored(ImVec4(nick_color[0], nick_color[1], nick_color[2], nick_color[3]), "Player Name");
        }
        
        if (weapon && *weapon) {
            ImGui::TextColored(ImVec4(weapon_color[0], weapon_color[1], weapon_color[2], weapon_color[3]), "AK-47");
        }
        
        if (hp && *hp > 0) {
            ImGui::TextColored(ImVec4(hp_color[0], hp_color[1], hp_color[2], hp_color[3]), "HP: %d", *hp);
        }
        
        ImGui::EndChild();
    }
}
