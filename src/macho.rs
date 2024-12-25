use std::{fs::File, io::Read, path::Path, process::Command};

use anyhow::{bail, Result};

pub(crate) fn is_mach_object(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut buffer = Vec::with_capacity(4);
    let Ok(n) = file.take(4).read_to_end(&mut buffer) else {
        return false;
    };
    // 0xFEEDFACE for 32-bit and 0xFEEDFACF for 64-bit
    n == 4
        && buffer[3] == 0xFE
        && buffer[2] == 0xED
        && buffer[1] == 0xFA
        && (buffer[0] == 0xCE || buffer[0] == 0xCF)
}

pub(crate) fn add_rpath(macho_path: &Path, rpath: &Path) -> Result<()> {
    let output = Command::new("install_name_tool")
        .arg("-add_rpath")
        .arg(rpath)
        .arg(macho_path)
        .output()?;
    if !output.status.success() {
        bail!(
            "install_name_tool failed: {}",
            String::from_utf8(output.stderr).unwrap_or("stderr cannot be parsed".into())
        );
    }
    Ok(())
}

pub(crate) fn change_library_path(macho_path: &Path, from: &Path, to: &Path) -> Result<()> {
    let output = Command::new("install_name_tool")
        .arg("-change")
        .arg(from)
        .arg(to)
        .arg(macho_path)
        .output()?;
    if !output.status.success() {
        bail!(
            "install_name_tool failed: {}",
            String::from_utf8(output.stderr).unwrap_or("stderr cannot be parsed".into())
        );
    }
    Ok(())
}
