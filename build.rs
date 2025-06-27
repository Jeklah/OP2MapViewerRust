fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        let target = env::var("TARGET").unwrap();
        if target.contains("windows") {
            // Tell cargo to look for a resource file
            println!("cargo:rerun-if-changed=src/op2mapviewer.rc");
            println!("cargo:rerun-if-changed=src/op2mapviewer.exe.manifest");

            // Only try to use rc.exe if we're doing MSVC
            if target.contains("msvc") {
                embed_resource::compile("src/op2mapviewer.rc");
            } else {
                // For GNU targets (MinGW), use windres
                let windres_output = std::process::Command::new("windres")
                    .args(&[
                        "src/op2mapviewer.rc",
                        "-O",
                        "coff",
                        "-o",
                        "op2mapviewer.res",
                    ])
                    .output();

                match windres_output {
                    Ok(_) => {
                        // Link the resource file
                        println!("cargo:rustc-link-arg=op2mapviewer.res");
                    }
                    Err(e) => {
                        println!("cargo:warning=Failed to compile resources: {}", e);
                    }
                }
            }
        }
    }
}
