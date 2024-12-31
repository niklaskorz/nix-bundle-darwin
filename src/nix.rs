use std::{
    collections::HashMap,
    ffi::OsStr,
    io::BufRead,
    path::PathBuf,
    process::{Command, Stdio},
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
        .args(["--extra-experimental-features", "nix-command flakes"])
        .args(["derivation", "show"])
        .arg(installable)
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        bail!("nix derivation show failed");
    }
    let metas = serde_json::from_slice(&output.stdout)?;
    Ok(metas)
}

pub(crate) fn build<I, S>(installables: I, args: I) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("nix")
        .args(["--extra-experimental-features", "nix-command flakes"])
        .args(["build", "--no-link", "--print-out-paths"])
        .args(args)
        .args(installables)
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        bail!("nix build failed");
    }
    output
        .stdout
        .lines()
        .map(|it| it.map_err(anyhow::Error::from))
        .collect()
}
