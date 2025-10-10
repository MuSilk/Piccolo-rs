use std::{env, fs, path::Path, process::Command};

const GLSLC_PATH: &str = "D:\\VulkanSDK\\1.3.268.0\\Bin\\glslc.exe";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:warning= start compiling GLSL shaders...");

    let project_dir = env::var("CARGO_MANIFEST_DIR")?;
    let shader_dir = Path::new(&project_dir).join("src").join("shader").join("glsl");
    let spv_dir = Path::new(&project_dir).join("src").join("shader").join("generated").join("spv");
    let shader_rs_path = Path::new(&project_dir).join("src").join("shader").join("generated").join("shader.rs");

    if fs::exists(&spv_dir)? {
        fs::remove_dir_all(&spv_dir)?;
    }
    fs::create_dir_all(&spv_dir)?;

    let mut shader_module_lines = Vec::new();

    for entry in fs::read_dir(&shader_dir)? {
        let entry = entry?;
        let path = entry.path();

        match path.extension().and_then(|s| s.to_str()) {
            Some("vert") | Some("frag") => {
                println!("cargo:warning= compiling shader: {:?}", path);
                let generated_file_name = path.file_name().unwrap().to_str().unwrap().to_owned() + ".spv";
                let result = Command::new(GLSLC_PATH)
                    .arg(&path)
                    .arg("-o")
                    .arg(spv_dir.join(&generated_file_name))
                    .output()?;

                if !result.status.success() {
                    let err = String::from_utf8_lossy(&result.stderr);
                    panic!("glslc compile failed ({}):\n{}", path.display(), err);
                }

                shader_module_lines.push(format!(
                    "pub static {}: &[u8] = include_bytes!(r\"{}\");",
                    path.file_name().unwrap().to_str().unwrap().to_uppercase().replace('.', "_"),
                    spv_dir.join(&generated_file_name).display()
                ));

                println!("cargo:rerun-if-changed={}", path.display());
            },
            _ => {}
        }
    }

    if fs::exists(&shader_rs_path)? {
        fs::remove_file(&shader_rs_path)?;
    }

    let shader_rs_content = shader_module_lines.join("\n") + "\n";
    fs::write(&shader_rs_path, shader_rs_content)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/shader/glsl");
    println!("cargo:rerun-if-changed={}", shader_rs_path.display());

    Ok(())
}