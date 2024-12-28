use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Result};
use schnauzer::LcVariant;

// May be set at compile time to use LLVM's variant or to harcode an absolute path
// into the binary, so we don't need a wrapper for our Nix package.
const INSTALL_NAME_TOOL: &'static str = match option_env!("INSTALL_NAME_TOOL") {
    Some(v) => v,
    None => "install_name_tool",
};

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

pub(crate) fn get_dylibs(macho_path: &Path) -> Result<Vec<String>> {
    let parser = schnauzer::Parser::build(&macho_path)?.parse()?;
    let dylibs: Vec<String> = parser
        .mach_objects()
        .iter()
        .flat_map(|o| o.load_commands_iterator())
        .flat_map(|cmd| match cmd.variant {
            LcVariant::LoadDylib(dylib) => dylib.name.load_string().ok(),
            _ => None,
        })
        .collect();
    Ok(dylibs)
}

pub(crate) fn add_rpath_and_change_libraries(
    macho_path: &Path,
    rpath: &Path,
    changes: Vec<(PathBuf, PathBuf)>,
) -> Result<()> {
    let mut command = Command::new(INSTALL_NAME_TOOL);
    for (from, to) in changes {
        command.arg("-change").arg(from).arg(to);
    }
    let output = command
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
