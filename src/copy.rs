use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::macho::{add_rpath_and_change_libraries, get_dylibs, is_mach_object};

const NIX_STORE: &'static str = "/nix/store";

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

        if is_mach_object(&src_path) {
            let mut changes = vec![];

            // Copy all dependencies from nix store
            for dep in get_dylibs(&dst_path)? {
                let dep_src: PathBuf = dep.into();
                if !dep_src.starts_with(NIX_STORE) || !dep_src.try_exists()? {
                    continue;
                }
                let dep_dst = dependency_path(&dep_src, target_store)?;
                copy_path(&dep_src, &dep_dst, target_store)?;
                let rpath_store = PathBuf::from_str("@rpath")?;
                let rpath_dep = dependency_path(&dep_src, &rpath_store)?;
                changes.push((dep_src, rpath_dep));
            }

            if !changes.is_empty() {
                let rel_store = pathdiff::diff_paths(target_store, parent)
                    .context("could not determine relative path")?;
                let rpath = PathBuf::from_str("@loader_path")?.join(rel_store);
                add_rpath_and_change_libraries(&dst_path, &rpath, changes)?;
            }
        }
    }

    Ok(())
}
