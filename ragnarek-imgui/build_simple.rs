use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link Windows libraries 
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");
    
    // Just compile our simple implementation without external ImGui files
    cc::Build::new()
        .cpp(true)
        .file("imgui_edited_simple.cpp")
        .file("ragnarek_wrapper.cpp")
        .flag_if_supported("/std:c++17")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/DNOMINMAX")
        .flag_if_supported("/D_WIN32_WINNT=0x0601")
        .compile("ragnarek_imgui");
    
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
