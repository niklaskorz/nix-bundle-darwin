use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::macho::{add_rpath, change_library_path, is_mach_object};
use crate::paths::get_nix_store_paths;

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
        copy_path(entry_path, &dst_path, target_store)?;
    }
    Ok(())
}

fn copy_path(src_path: &Path, dst_path: &Path, target_store: &Path) -> Result<()> {
    if dst_path.try_exists().unwrap() {
        return Ok(());
    }
    let src_md = fs::symlink_metadata(src_path)?;
    if !(src_md.is_file() || src_md.is_symlink()) {
        return Ok(());
    }
    let parent = dst_path
        .parent()
        .context("cannot copy to path without parent")?;
    if !parent.is_dir() {
        fs::create_dir_all(parent)
            .context(format!("creating parent directory {}", parent.display()))?;
    }

    if src_md.is_symlink() {
        use std::os::unix::fs::symlink;
        let link_target = src_path.canonicalize()?;
        if link_target.starts_with(NIX_STORE) {
            let dep_path = dependency_path(&link_target, target_store)?;
            let rel_link_target = pathdiff::diff_paths(&dep_path, parent)
                .context("could not determine relative path")?;
            symlink(&rel_link_target, dst_path).context(format!(
                "symlinking {} to {}",
                dst_path.display(),
                rel_link_target.display(),
            ))?;
            if link_target.is_dir() {
                recursive_writable_copy(&link_target, &dep_path, target_store)?;
            } else {
                copy_path(&link_target, &dep_path, target_store)?;
            }
        } else {
            symlink(&link_target, dst_path).context(format!(
                "symlinking {} to {}",
                dst_path.display(),
                link_target.display(),
            ))?;
        }
    } else if src_md.is_file() {
        fs::copy(src_path, &dst_path).context(format!(
            "copying file {} to {}",
            src_path.display(),
            dst_path.display(),
        ))?;

        let mut permissions = src_md.permissions();
        // Make owner-writable (similar to chmod u+w)
        permissions.set_mode(permissions.mode() | 0b010000000);
        fs::set_permissions(&dst_path, permissions)?;

        let is_macho = is_mach_object(&src_path);
        if is_macho {
            let rel_store = pathdiff::diff_paths(target_store, parent)
                .context("could not determine relative path")?;
            let rpath = PathBuf::from_str("@loader_path")?.join(rel_store);
            add_rpath(&dst_path, &rpath)?;
        }

        // Copy all dependencies from nix store
        for dep in get_nix_store_paths(src_path)
            .context(format!("nix store path deps for {}", src_path.display()))?
        {
            let dep_dst = dependency_path(&dep, target_store)?;
            if dep.try_exists()? {
                copy_path(&dep, &dep_dst, target_store)?;
                if is_macho {
                    let rpath_store = PathBuf::from_str("@rpath")?;
                    let rpath_dep = dependency_path(&dep, &rpath_store)?;
                    change_library_path(&dst_path, &dep, &rpath_dep)?;
                }
            }
        }
    }

    Ok(())
}
