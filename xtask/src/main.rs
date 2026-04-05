use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const MAIN_MANIFEST_PATH: &str = "crates/clap-tui/Cargo.toml";
const MACRO_MANIFEST_PATH: &str = "crates/clap-tui-macros/Cargo.toml";
const MACRO_CRATE_NAME: &str = "clap-tui-macros";

#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<PackageSection>,
    dependencies: Option<BTreeMap<String, DependencySpec>>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencySpec {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Deserialize)]
struct DetailedDependency {
    version: Option<String>,
}

#[derive(Debug)]
struct ReleasePlan {
    clap_tui_version: String,
    clap_tui_macros_requirement: String,
    clap_tui_macros_version: String,
    clap_tui_macros_manifest_version: String,
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
        Some("check-macro-tag-version") => {
            let tag_name = args.next().ok_or_else(|| {
                "Usage: cargo run -q -p xtask -- check-macro-tag-version clap-tui-macros-vX.Y.Z"
                    .to_string()
            })?;
            if args.next().is_some() {
                return Err(
                    "Usage: cargo run -q -p xtask -- check-macro-tag-version clap-tui-macros-vX.Y.Z"
                        .to_string(),
                );
            }
            check_macro_tag_version(&tag_name)
        }
        Some("release-plan") => {
            let github_output = parse_github_output_flag(args)?;
            print_release_plan(github_output.as_deref())
        }
        Some("check-crate-version-published") => {
            let crate_name = args.next().ok_or_else(|| {
                "Usage: cargo run -q -p xtask -- check-crate-version-published <crate> <version>"
                    .to_string()
            })?;
            let version = args.next().ok_or_else(|| {
                "Usage: cargo run -q -p xtask -- check-crate-version-published <crate> <version>"
                    .to_string()
            })?;
            if args.next().is_some() {
                return Err(
                    "Usage: cargo run -q -p xtask -- check-crate-version-published <crate> <version>"
                        .to_string(),
                );
            }
            check_crate_version_published(&crate_name, &version)
        }
        _ => Err(
            "Usage:\n  cargo run -q -p xtask -- check-tag-version vX.Y.Z\n  cargo run -q -p xtask -- check-macro-tag-version clap-tui-macros-vX.Y.Z\n  cargo run -q -p xtask -- release-plan [--github-output PATH]\n  cargo run -q -p xtask -- check-crate-version-published <crate> <version>"
                .to_string(),
        ),
    }
}

fn parse_github_output_flag<I>(mut args: I) -> Result<Option<String>, String>
where
    I: Iterator<Item = String>,
{
    match args.next().as_deref() {
        None => Ok(None),
        Some("--github-output") => {
            let path = args.next().ok_or_else(|| {
                "Usage: cargo run -q -p xtask -- release-plan [--github-output PATH]".to_string()
            })?;
            if args.next().is_some() {
                return Err(
                    "Usage: cargo run -q -p xtask -- release-plan [--github-output PATH]"
                        .to_string(),
                );
            }
            Ok(Some(path))
        }
        Some(_) => {
            Err("Usage: cargo run -q -p xtask -- release-plan [--github-output PATH]".to_string())
        }
    }
}

fn check_tag_version(tag_name: &str) -> Result<(), String> {
    let release_plan = load_release_plan()?;
    validate_tag_version(
        tag_name,
        "v",
        &release_plan.clap_tui_version,
        "clap-tui",
        MAIN_MANIFEST_PATH,
    )
}

fn check_macro_tag_version(tag_name: &str) -> Result<(), String> {
    let macro_manifest = read_manifest(Path::new(MACRO_MANIFEST_PATH))?;
    let macro_version = manifest_version(&macro_manifest, MACRO_MANIFEST_PATH)?;
    validate_tag_version(
        tag_name,
        "clap-tui-macros-v",
        &macro_version,
        MACRO_CRATE_NAME,
        MACRO_MANIFEST_PATH,
    )
}

fn validate_tag_version(
    tag_name: &str,
    prefix: &str,
    expected_version: &str,
    crate_name: &str,
    manifest_path: &str,
) -> Result<(), String> {
    if !tag_name.starts_with(prefix) {
        return Err(format!(
            "Expected a release tag starting with '{prefix}' for {crate_name}, got: {tag_name}"
        ));
    }

    let tag_version = &tag_name[prefix.len()..];
    if tag_version != expected_version {
        return Err(format!(
            "Release tag {tag_name} does not match {crate_name} version {expected_version} from {manifest_path}"
        ));
    }

    println!("Release tag {tag_name} matches {crate_name} version {expected_version}");
    Ok(())
}

fn print_release_plan(github_output_path: Option<&str>) -> Result<(), String> {
    let release_plan = load_release_plan()?;

    println!("clap-tui version: {}", release_plan.clap_tui_version);
    println!(
        "referenced clap-tui-macros requirement: {}",
        release_plan.clap_tui_macros_requirement
    );
    println!(
        "referenced clap-tui-macros version: {}",
        release_plan.clap_tui_macros_version
    );
    println!(
        "workspace clap-tui-macros manifest version: {}",
        release_plan.clap_tui_macros_manifest_version
    );

    if let Some(path) = github_output_path {
        fs::write(
            path,
            format!(
                "clap_tui_version={}\nclap_tui_macros_requirement={}\nclap_tui_macros_version={}\nclap_tui_macros_manifest_version={}\n",
                release_plan.clap_tui_version,
                release_plan.clap_tui_macros_requirement,
                release_plan.clap_tui_macros_version,
                release_plan.clap_tui_macros_manifest_version,
            ),
        )
        .map_err(|error| format!("Could not write GitHub output file {path}: {error}"))?;
    }

    Ok(())
}

fn check_crate_version_published(crate_name: &str, version: &str) -> Result<(), String> {
    let client = Client::builder()
        .user_agent("clap-tui-xtask/0.1.0")
        .build()
        .map_err(|error| format!("Could not construct crates.io client: {error}"))?;
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
    let response = client.get(&url).send().map_err(|error| {
        format!("Could not query crates.io for {crate_name} {version}: {error}")
    })?;

    match response.status() {
        StatusCode::OK => {
            println!("{crate_name} {version} is available on crates.io");
            Ok(())
        }
        StatusCode::NOT_FOUND => Err(format!(
            "{crate_name} {version} is not available on crates.io yet. Publish it independently before retrying the clap-tui release workflow."
        )),
        status => Err(format!(
            "Unexpected crates.io response while checking {crate_name} {version}: {status}"
        )),
    }
}

fn load_release_plan() -> Result<ReleasePlan, String> {
    let clap_tui_manifest = read_manifest(Path::new(MAIN_MANIFEST_PATH))?;
    let clap_tui_macros_manifest = read_manifest(Path::new(MACRO_MANIFEST_PATH))?;

    let clap_tui_version = manifest_version(&clap_tui_manifest, MAIN_MANIFEST_PATH)?;
    let clap_tui_macros_manifest_version =
        manifest_version(&clap_tui_macros_manifest, MACRO_MANIFEST_PATH)?;
    let clap_tui_macros_requirement = clap_tui_manifest
        .dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.get(MACRO_CRATE_NAME))
        .and_then(dependency_version)
        .ok_or_else(|| {
            format!(
                "Could not read {MACRO_CRATE_NAME} dependency version from {MAIN_MANIFEST_PATH}"
            )
        })?;
    let clap_tui_macros_version =
        normalize_exact_version_requirement(&clap_tui_macros_requirement)?;

    Ok(ReleasePlan {
        clap_tui_version,
        clap_tui_macros_requirement,
        clap_tui_macros_version,
        clap_tui_macros_manifest_version,
    })
}

fn read_manifest(path: &Path) -> Result<CargoManifest, String> {
    let manifest_text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    toml::from_str(&manifest_text)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn manifest_version(manifest: &CargoManifest, path: &str) -> Result<String, String> {
    manifest
        .package
        .as_ref()
        .and_then(|package| package.version.clone())
        .ok_or_else(|| format!("Could not read package.version from {path}"))
}

fn dependency_version(spec: &DependencySpec) -> Option<String> {
    match spec {
        DependencySpec::Simple(version) => Some(version.clone()),
        DependencySpec::Detailed(details) => details.version.clone(),
    }
}

fn normalize_exact_version_requirement(requirement: &str) -> Result<String, String> {
    let trimmed = requirement.trim();
    let normalized = trimmed.trim_start_matches('=').trim();

    if trimmed == normalized {
        return Err(format!(
            "Expected {MACRO_CRATE_NAME} dependency in {MAIN_MANIFEST_PATH} to use an exact version requirement like =0.1.0, got: {requirement}"
        ));
    }

    if normalized.is_empty() {
        return Err(format!(
            "Expected {MACRO_CRATE_NAME} dependency in {MAIN_MANIFEST_PATH} to include a version after '=', got: {requirement}"
        ));
    }

    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        DependencySpec, DetailedDependency, dependency_version,
        normalize_exact_version_requirement, validate_tag_version,
    };

    #[test]
    fn reads_simple_dependency_version() {
        let spec = DependencySpec::Simple("0.1.0".to_string());
        assert_eq!(dependency_version(&spec).as_deref(), Some("0.1.0"));
    }

    #[test]
    fn reads_detailed_dependency_version() {
        let spec = DependencySpec::Detailed(DetailedDependency {
            version: Some("0.2.0".to_string()),
        });
        assert_eq!(dependency_version(&spec).as_deref(), Some("0.2.0"));
    }

    #[test]
    fn handles_missing_detailed_dependency_version() {
        let spec = DependencySpec::Detailed(DetailedDependency { version: None });
        assert_eq!(dependency_version(&spec), None);
    }

    #[test]
    fn normalizes_exact_version_requirement() {
        assert_eq!(
            normalize_exact_version_requirement("=0.3.1").as_deref(),
            Ok("0.3.1")
        );
    }

    #[test]
    fn rejects_non_exact_version_requirement() {
        let error = normalize_exact_version_requirement("^0.3.1")
            .expect_err("non-exact version requirements should be rejected");
        assert!(error.contains("exact version requirement"));
    }

    #[test]
    fn validates_main_release_tags() {
        assert!(validate_tag_version("v0.3.1", "v", "0.3.1", "clap-tui", "manifest").is_ok());
    }

    #[test]
    fn validates_macro_release_tags() {
        assert!(
            validate_tag_version(
                "clap-tui-macros-v0.3.1",
                "clap-tui-macros-v",
                "0.3.1",
                "clap-tui-macros",
                "manifest",
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_wrong_tag_prefix() {
        let error = validate_tag_version(
            "v0.3.1",
            "clap-tui-macros-v",
            "0.3.1",
            "clap-tui-macros",
            "manifest",
        )
        .expect_err("wrong tag prefix should be rejected");
        assert!(error.contains("starting with 'clap-tui-macros-v'"));
    }
}
