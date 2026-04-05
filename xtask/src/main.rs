use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Deserialize)]
struct CargoManifest {
    package: Option<PackageSection>,
}

#[derive(Deserialize)]
struct PackageSection {
    version: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check-tag-version") => {
            let tag_name = args.next().ok_or_else(|| {
                "Usage: cargo run -q -p xtask -- check-tag-version vX.Y.Z".to_string()
            })?;
            if args.next().is_some() {
                return Err("Usage: cargo run -q -p xtask -- check-tag-version vX.Y.Z".to_string());
            }
            check_tag_version(&tag_name)
        }
        _ => Err("Usage: cargo run -q -p xtask -- check-tag-version vX.Y.Z".to_string()),
    }
}

fn check_tag_version(tag_name: &str) -> Result<(), String> {
    if !tag_name.starts_with('v') {
        return Err(format!(
            "Expected a release tag starting with 'v', got: {tag_name}"
        ));
    }

    let tag_version = &tag_name[1..];
    let manifest_path = Path::new("crates/clap-tui/Cargo.toml");
    let manifest_text = fs::read_to_string(manifest_path)
        .map_err(|error| format!("Could not read {}: {error}", manifest_path.display()))?;
    let manifest: CargoManifest = toml::from_str(&manifest_text)
        .map_err(|error| format!("Could not parse {}: {error}", manifest_path.display()))?;
    let manifest_version = manifest
        .package
        .and_then(|package| package.version)
        .ok_or_else(|| {
            format!(
                "Could not read package.version from {}",
                manifest_path.display()
            )
        })?;

    if tag_version != manifest_version {
        return Err(format!(
            "Release tag {tag_name} does not match clap-tui version {manifest_version}"
        ));
    }

    println!("Release tag {tag_name} matches clap-tui version {manifest_version}");
    Ok(())
}
