use std::process::Command;

fn main() {
    // Prefer MERMAID_ASCII_VERSION env (set by Docker/CI), fall back to git tag, then "dev".
    let version = std::env::var("MERMAID_ASCII_VERSION")
        .ok()
        .filter(|s| !s.is_empty() && s != "dev")
        .or_else(|| {
            Command::new("git")
                .args(["describe", "--tags", "--always"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=MERMAID_ASCII_VERSION={}", version);

    // Compile .hom files to .rs if homunc is available in PATH.
    // Generated .rs files are checked in, so this is optional.
    compile_hom_files();
}

fn compile_hom_files() {
    let dirs = ["src"];
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "hom") {
                let rs_path = path.with_extension("rs");
                // Only recompile if .hom is newer than .rs
                let needs_compile = !rs_path.exists()
                    || std::fs::metadata(&path)
                        .and_then(|hom| {
                            std::fs::metadata(&rs_path)
                                .map(|rs| hom.modified().unwrap() > rs.modified().unwrap())
                        })
                        .unwrap_or(true);
                if needs_compile {
                    let status = Command::new("homunc")
                        .args([
                            "--raw",
                            &path.to_string_lossy(),
                            "-o",
                            &rs_path.to_string_lossy(),
                        ])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            println!(
                                "cargo:warning=Compiled {} -> {}",
                                path.display(),
                                rs_path.display()
                            );
                        }
                        _ => {
                            // homunc not available — use checked-in .rs file
                        }
                    }
                }
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
