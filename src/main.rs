mod copy;
mod nix;

use anyhow::{bail, Context, Result};
use clap::Parser;
use copy::{copy_dependencies, recursive_writable_copy};
use std::{fs, path::Path};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Flake installable to bundle, e.g., nixpkgs#hello
    #[arg(long)]
    flake: String,

    /// Overwrite existing bundles
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let results_path = std::env::current_dir()?.join("results");
    std::fs::create_dir_all(&results_path)?;

    nix::build(&args.flake)?;
    let derivations = nix::get_derivation_metas(&args.flake)?;
    for drv in derivations.values() {
        println!("Bundling {}", drv.name);
        let output = drv
            .outputs
            .get("bin")
            .or(drv.outputs.get("out"))
            .context("no bin or out outputs found")?;
        println!("Source: {}", output.path.display());

        let applications_dir = output.path.join("Applications");
        if applications_dir.is_dir() {
            println!("Source contains applications");
            for entry in fs::read_dir(&applications_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.extension().is_some_and(|ext| ext == "app") {
                    bundle_application(&path, &results_path, args.force)?;
                }
            }
        }
    }

    Ok(())
}

fn bundle_application(app_path: &Path, results_path: &Path, force: bool) -> Result<()> {
    println!("Bundling application {}", app_path.display());
    let app_name = app_path.file_name().context("cannot determine app name")?;
    let target_path = results_path.join(app_name);
    if target_path.exists() {
        if force {
            println!(
                "Target path already exists and will be removed: {}",
                target_path.display()
            );
            std::fs::remove_dir_all(&target_path)?;
        } else {
            bail!(
                "Target path already exists and `--force` was not provided: {}",
                target_path.display()
            );
        }
    }
    recursive_writable_copy(app_path, &target_path)?;
    let dependencies = nix::get_dependencies(app_path)?;
    let target_store = target_path.join("Contents").join("nix");
    fs::create_dir_all(&target_store)?;
    copy_dependencies(&dependencies, &target_store)?;
    Ok(())
}
