use std::process::Command;

fn main() {
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if let Ok(version) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=RUSTC_VERSION={}", version.trim());
        }
    }

    // if cfg!(target_os = "windows") {
    if let Ok(output) = Command::new("cmd.exe")
        .args(["/c", "wmic os get version /value"])
        .output()
    {
        if let Ok(version) = String::from_utf8(output.stdout) {
            println!(
                "cargo:rustc-env=WINDOWS_VERSION={}",
                version.trim().replace("Version=", "")
            );
        }
    }
    // } else {
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        if let Ok(version) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=UNIX_KERNEL_RELEASE={}", version.trim());
        }
    }
    // }
}