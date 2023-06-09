use std::process::Command;

fn main() {
    let git_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_or(String::from("unknown"), |output| {
            String::from(String::from_utf8_lossy(&output.stdout))
        });
    println!("cargo:rustc-env=GIT_SHA={git_sha}");
}
