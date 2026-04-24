use serde::Deserialize;
use std::env;
use std::path::Path;
use std::process::ExitCode;

const MAIN_MANIFEST_PATH: &str = "crates/clap-tui/Cargo.toml";

#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<PackageSection>,
}

#[derive(Debug, Deserialize)]
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
        _ => Err("Usage:\n  cargo run -q -p xtask -- check-tag-version vX.Y.Z".to_string()),
    }
}

fn check_tag_version(tag_name: &str) -> Result<(), String> {
    let manifest = read_manifest(Path::new(MAIN_MANIFEST_PATH))?;
    let expected_version = manifest_version(&manifest, MAIN_MANIFEST_PATH)?;

    if !tag_name.starts_with("v") {
        return Err(format!(
            "Expected a release tag starting with 'v' for clap-tui, got: {tag_name}"
        ));
    }

    let tag_version = &tag_name[1..];
    if tag_version != expected_version {
        return Err(format!(
            "Release tag {tag_name} does not match clap-tui version {expected_version} from {MAIN_MANIFEST_PATH}"
        ));
    }

    println!("Release tag {tag_name} matches clap-tui version {expected_version}");
    Ok(())
}

fn read_manifest(path: &Path) -> Result<CargoManifest, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("Could not read manifest {}: {error}", path.display()))?;
    toml::from_str(&text)
        .map_err(|error| format!("Could not parse manifest {}: {error}", path.display()))
}

fn manifest_version(manifest: &CargoManifest, manifest_path: &str) -> Result<String, String> {
    manifest
        .package
        .as_ref()
        .and_then(|package| package.version.clone())
        .ok_or_else(|| format!("Could not read package.version from {manifest_path}"))
}
