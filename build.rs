use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=install.sh");
    println!("cargo:rerun-if-changed=configs");
    println!("cargo:rerun-if-changed=README.md");

    let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.chars().all(|character| character.is_ascii_hexdigit()) {
        println!("cargo:rustc-env=STACC_BUILD_GIT_HASH={hash}");
    }
}
