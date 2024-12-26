mod copy;
mod macho;
mod nix;
mod paths;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser};
use copy::recursive_writable_copy;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// A darwin-compatible alternative to nix-bundle
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Parameters {
    /// What to bundle, interpretation depends on mode.  
    /// Default: must be a path, defaults to "default.nix";
    /// --flake: must be a flake installable, defaults to ".#default";
    /// --program: must be a program name, no default value.
    target: Option<String>,

    #[command(flatten)]
    mode: Mode,

    /// Overwrite existing bundles
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
struct Mode {
    /// Which attribute path of TARGET to build
    #[arg(short = 'A', long)]
    attr: Option<String>,

    /// Treat TARGET as program, e.g., teeworlds
    #[arg(short, long, default_value_t = false, requires = "target")]
    program: bool,

    /// Treat TARGET as flake installable, e.g., nixpkgs#teeworlds
    #[arg(short = 'F', long, default_value_t = false)]
    flake: bool,
}

fn main() -> Result<()> {
    let args = Parameters::parse();
    let outputs = if args.mode.flake {
        let installable = args.target.as_deref().unwrap_or(".#default");
        println!("Building {installable}");
        nix::build_flake(installable)?
    } else if args.mode.program {
        let program = args.target.unwrap();
        println!("Building {program}");
        let nixpkgs = nix::find_file("nixpkgs/default.nix")?;
        nix::build(&nixpkgs, Some(&program))?
    } else {
        let target = args.target.as_deref().unwrap_or("default.nix");
        println!("Building {target}");
        let path: PathBuf = target.into();
        nix::build(&path, args.mode.attr.as_deref())?
    };

    let results_path = std::env::current_dir()?.join("results");
    std::fs::create_dir_all(&results_path)?;

    for output in outputs {
        println!("Looking for applications in {output}");

        let applications_dir = PathBuf::from_str(&output)?.join("Applications");
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
    let target_store = target_path.join("Contents").join("nix");
    println!(
        "Copying app and dependencies from {} to {}",
        target_path.display(),
        app_path.display()
    );
    recursive_writable_copy(app_path, &target_path, &target_store)?;
    Ok(())
}
