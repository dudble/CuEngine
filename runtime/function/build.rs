use std::path::Path;
use std::process::Command;

fn compile_shader() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let shader_dir = Path::new("shaders");

    for shader in ["sprite.vert", "sprite.frag"] {
        let source = shader_dir.join(shader);
        println!("cargo:rerun-if-changed={}", source.display()); //监听着色器变化
        let output = Path::new(&out_dir).join(format!("{shader}.spv"));
        let status = Command::new("glslc")
            .arg(&source)
            .arg("-o")
            .arg(&output)
            .status()
            .unwrap_or_else(|error| panic!("failed to run glslc for {}: {error}", source.display()));

        if !status.success() {
            panic!("glslc failed to compile {}", source.display());
        }
    }
}

fn main() {
    compile_shader();
}
