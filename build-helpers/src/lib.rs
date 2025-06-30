pub fn setup_llvm_toolchain() {
    let target = std::env::var("TARGET").unwrap_or_default();

    if target == "wasm32-unknown-unknown" && cfg!(target_os = "macos") {
        match std::process::Command::new("brew")
            .arg("--prefix")
            .arg("llvm")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("cargo:warning=Using LLVM from Homebrew at {path}");
                    println!("cargo:rustc-env=CC={path}/bin/clang");
                    println!("cargo:rustc-env=AR={path}/bin/llvm-ar");
                } else {
                    println!("cargo:warning=`brew` is installed, but `llvm` is not. Run: `brew install llvm`");
                }
            }
            Err(e) => {
                println!("cargo:warning=Homebrew not found (brew command failed: {e}). Falling back to system tools.");
            }
        }
    }

    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=AR");
}
