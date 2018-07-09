use std::env;
use std::process::Command;

fn main() {
    // copied from https://github.com/emoon/dynamic_reload/blob/master/build.rs
    let profile = env::var("PROFILE").unwrap_or("Debug".to_string());

    let mut build_predict_command = Command::new("cargo");
    build_predict_command
        .arg("build")
        .arg("--manifest-path")
        .arg("predict/Cargo.toml");

    // FIXME this is copied from dynamic_reload example and doesn't seem to work with
    // PROFILE=Release set when building brick, not sure why. also doesn't pass on --release on
    // cargo build brick
    if profile == "Release" {
        build_predict_command.arg("--release");
    }

    println!("Building predict crate:\n\n    {:?}\n", build_predict_command);
    build_predict_command
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
}
