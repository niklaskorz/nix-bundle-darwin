use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

const NIX_STORE: &'static str = "/nix/store";

pub(crate) fn copy_dependencies(deps: &Vec<PathBuf>, target_store: &Path) -> Result<()> {
    for src_dir in deps {
        copy_dependency(src_dir, target_store)?;
    }
    Ok(())
}

fn copy_dependency(src_dir: &Path, target_store: &Path) -> Result<()> {
    let pattern = format!(r"{}\/[0-9a-z]{{32}}-", regex::escape(NIX_STORE));
    let re = regex::Regex::new(&pattern)?;
    let dst_name = re.replace(src_dir.to_str().unwrap(), "").to_string();
    let dst_dir = target_store.join(dst_name);
    // TODO: We might also need other paths that are referenced by wrappers,
    // e.g. bin (especially for Node or Python applications).
    recursive_writable_copy_with_filter(src_dir, &dst_dir, |rel_path| {
        rel_path.starts_with("Library") || rel_path.starts_with("lib")
    })?;

    Ok(())
}

pub(crate) fn recursive_writable_copy(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    recursive_writable_copy_with_filter(src_dir, dst_dir, |_| true)
}

pub(crate) fn recursive_writable_copy_with_filter<F>(
    src_dir: &Path,
    dst_dir: &Path,
    rel_path_filter: F,
) -> Result<()>
where
    F: Fn(&Path) -> bool,
{
    for entry in walkdir::WalkDir::new(src_dir).follow_links(true) {
        let entry = entry?;
        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(src_dir)?;
        let dst_path = dst_dir.join(rel_path);

        if rel_path.as_os_str().is_empty() || rel_path_filter(rel_path) {
            println!("Copying {} to {}", entry_path.display(), dst_path.display());
            let file_type = entry.file_type();
            if file_type.is_dir() {
                fs::create_dir(&dst_path)?;
            } else if file_type.is_file() {
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
    }
    Ok(())
}
