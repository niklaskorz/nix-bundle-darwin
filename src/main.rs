mod copy;
mod macho;
mod nix;
mod paths;

use anyhow::{bail, Context, Result};
use apple_codesign::{BundleSigner, SigningSettings};
use clap::{Args, Parser};
use copy::recursive_writable_copy;
use std::{fs, path::Path};

/// A darwin-compatible alternative to nix-bundle
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Parameters {
    /// What to bundle.
    /// Installables that resolve to derivations are built (or substituted if possible).
    /// Store path installables are substituted.
    installables: Vec<String>,

    #[command(flatten)]
    mode: Mode,

    /// Overwrite existing bundles
    #[arg(long, default_value_t = false)]
    force: bool,

    /// Selfsign the resulting application bundles
    #[arg(short, long, default_value_t = false)]
    sign: bool,

    /// Additional arguments to pass to `nix build`
    #[arg(last = true)]
    build_args: Vec<String>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
struct Mode {
    /// Interpret installables as attribute paths of the Nix expression stored in <FILE>.
    #[arg(short, long)]
    file: Option<String>,

    /// Interpret installables as nixpkgs programs, equivalent to `--file <nixpkgs>`
    #[arg(short, long, default_value_t = false, requires = "installables")]
    programs: bool,
}

fn main() -> Result<()> {
    let mut args = Parameters::parse();
    if args.mode.programs {
        args.mode.file = Some("<nixpkgs>".into());
    }
    if let Some(file) = args.mode.file {
        args.build_args.extend(["--file".into(), file]);
    }
    let outputs = nix::build(args.installables, args.build_args);

    let results_path = std::env::current_dir()?.join("results");
    std::fs::create_dir_all(&results_path)?;

    for output in outputs.iter().flatten() {
        println!("Looking for applications in {}", output.display());

        let applications_dir = output.join("Applications");
        if applications_dir.is_dir() {
            println!("Source contains applications");
            for entry in fs::read_dir(&applications_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.extension().is_some_and(|ext| ext == "app") {
                    bundle_application(&path, &results_path, args.force, args.sign)?;
                }
            }
        }
    }

    Ok(())
}

fn bundle_application(app_path: &Path, results_path: &Path, force: bool, sign: bool) -> Result<()> {
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
    let target_store = target_path.join("Contents").join("nix");
    println!(
        "Copying app and dependencies from {} to {}",
        app_path.display(),
        target_path.display(),
    );
    recursive_writable_copy(app_path, &target_path, &target_store)?;

    if sign {
        println!("Signing {}", target_path.display());
        let settings = SigningSettings::default();
        let mut signer = BundleSigner::new_from_path(&target_path)?;
        signer.collect_nested_bundles()?;
        signer.write_signed_bundle(&target_path, &settings)?;
    }

    Ok(())
}
