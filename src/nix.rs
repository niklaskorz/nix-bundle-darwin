use std::{
    ffi::OsStr,
    io::BufRead,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{bail, Result};

pub(crate) fn build<I, S>(installables: I, args: I) -> Result<Vec<PathBuf>>
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
        .map(|it| it.map(|line| line.into()).map_err(anyhow::Error::from))
        .collect()
}
