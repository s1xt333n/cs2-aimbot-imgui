fn main() {
    // Tell cargo to tell rustc to link Windows libraries 
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");    // For now, just compile a minimal stub to test the integration
    // We'll rely on the main application's ImGui linkage
    cc::Build::new()
        .cpp(true)
        .file("standalone_ragnarek.cpp")
        .define("IMGUI_DISABLE_OBSOLETE_FUNCTIONS", None)
        .flag_if_supported("/std:c++17")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/DNOMINMAX")
        .flag_if_supported("/D_WIN32_WINNT=0x0601")
        .compile("ragnarek_imgui");
}