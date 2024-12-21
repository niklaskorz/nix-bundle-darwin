use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

const NIX_STORE: &'static str = "/nix/store";
const NIX_HASH_LEN: usize = 32;

pub(crate) fn copy_dependencies(deps: &Vec<PathBuf>, target_store: &Path) -> Result<()> {
    for src_dir in deps {
        copy_dependency(src_dir, target_store)?;
    }
    Ok(())
}

fn copy_dependency(src_dir: &Path, target_store: &Path) -> Result<()> {
    let dst_dir = dependency_path(src_dir, target_store)?;
    recursive_writable_copy(src_dir, &dst_dir, target_store)?;

    Ok(())
}

fn dependency_path(src_path: &Path, target_store: &Path) -> Result<PathBuf> {
    let dst_name = src_path.strip_prefix(NIX_STORE)?;
    let dst_path = target_store.join(dst_name);
    Ok(dst_path)
}

pub(crate) fn recursive_writable_copy(
    src_dir: &Path,
    dst_dir: &Path,
    target_store: &Path,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(src_dir).follow_root_links(false) {
        let entry = entry?;
        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(src_dir)?;
        let dst_path = dst_dir.join(rel_path);

        let file_type = entry.file_type();
        if file_type.is_symlink() {
            let mut real_path = entry_path.read_link()?;
            if real_path.starts_with(NIX_STORE) {
                let dep_path = dependency_path(&real_path, target_store)?;
                real_path =
                    pathdiff::diff_paths(dep_path, &dst_path.parent().context("no parent dir")?)
                        .context("could not determine relative path")?;
            }
            println!(
                "Symlinking {} to {}",
                dst_path.display(),
                real_path.display(),
            );
            std::os::unix::fs::symlink(real_path, dst_path)?;
            continue;
        }

        if file_type.is_dir() {
            println!("Creating dir {}", dst_path.display());
            if !dst_path.is_dir() {
                fs::create_dir(&dst_path)?;
            }
        } else if file_type.is_file() {
            println!(
                "Copying file {} to {}",
                entry_path.display(),
                dst_path.display()
            );
            fs::copy(entry_path, &dst_path)?;
        } else {
            println!("Neither file or directory, skipping");
            continue;
        };
        let mut permissions = fs::metadata(entry_path)?.permissions();
        // Make owner-writable (similar to chmod u+w)
        permissions.set_mode(permissions.mode() | 0b010000000);
        fs::set_permissions(&dst_path, permissions)?;
    }
    Ok(())
}
