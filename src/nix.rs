use std::{
    collections::HashMap,
    io::BufRead,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct DerivationMeta {
    pub(crate) name: String,
    pub(crate) outputs: HashMap<String, DerivationOutput>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DerivationOutput {
    pub(crate) path: PathBuf,
}

pub(crate) fn get_derivation_metas(installable: &str) -> Result<HashMap<String, DerivationMeta>> {
    let output = Command::new("nix")
        .arg("derivation")
        .arg("show")
        .arg(installable)
        .output()?;
    if !output.status.success() {
        bail!(
            "nix derivation show failed: {}",
            String::from_utf8(output.stderr).unwrap_or("stderr cannot be parsed".into())
        );
    }
    let metas = serde_json::from_slice(&output.stdout)?;
    Ok(metas)
}

pub(crate) fn build(installable: &str) -> Result<()> {
    let output = Command::new("nix")
        .arg("build")
        .arg("--max-jobs")
        .arg("auto")
        .arg("--no-link")
        .arg(installable)
        .output()?;
    if !output.status.success() {
        bail!(
            "nix build failed: {}",
            String::from_utf8(output.stderr).unwrap_or("stderr cannot be parsed".into())
        );
    }
    Ok(())
}

pub(crate) fn get_dependencies(store_path: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("nix-store")
        .arg("--query")
        .arg("--requisites")
        .arg(store_path)
        .output()?;
    if !output.status.success() {
        bail!(
            "nix-store query failed: {}",
            String::from_utf8(output.stderr).unwrap_or("stderr cannot be parsed".into())
        );
    }
    let dependencies: std::result::Result<Vec<_>, _> = output
        .stdout
        .lines()
        .map(|result| result.map(|line| line.into()))
        .collect();
    Ok(dependencies?)
}
