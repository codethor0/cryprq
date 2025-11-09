// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use std::env;
use std::process::Command;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

fn main() {
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-changed=build.rs");

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    let git_sha = resolve_git_sha();
    let rustc_version = resolve_rustc_version();
    let features = collect_features();
    let build_time = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    let version_string = format!("{pkg_version} ({git_sha})");

    println!("cargo:rustc-env=CRYPRQ_BUILD_SHA={git_sha}");
    println!("cargo:rustc-env=CRYPRQ_BUILD_RUSTC={rustc_version}");
    println!("cargo:rustc-env=CRYPRQ_BUILD_FEATURES={features}");
    println!("cargo:rustc-env=CRYPRQ_BUILD_TIME={build_time}");
    println!("cargo:rustc-env=CRYPRQ_VERSION_STRING={version_string}");
}

fn resolve_git_sha() -> String {
    if let Some(env_sha) = env::var("GITHUB_SHA").ok() {
        return short_sha(&env_sha);
    }

    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let sha = String::from_utf8(output.stdout).ok()?;
            let trimmed = sha.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn short_sha(value: &str) -> String {
    if value.len() >= 7 {
        value[..7].to_string()
    } else {
        value.to_string()
    }
}

fn resolve_rustc_version() -> String {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn collect_features() -> String {
    let mut features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            key.strip_prefix("CARGO_FEATURE_")
                .map(|name| name.to_lowercase())
        })
        .collect();
    features.sort();
    if features.is_empty() {
        "default".to_string()
    } else {
        features.join(",")
    }
}
