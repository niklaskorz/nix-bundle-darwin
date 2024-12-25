use std::path::{Path, PathBuf};

use anyhow::Result;
use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{sinks::Lossy, BinaryDetection, MmapChoice, SearcherBuilder},
};

pub(crate) fn get_nix_store_paths(file_path: &Path) -> Result<Vec<PathBuf>> {
    let matcher = RegexMatcher::new(r"/nix/store/[a-zA-Z0-9/.\-_\\]+")?;
    let mut matches: Vec<PathBuf> = vec![];
    SearcherBuilder::new()
        .binary_detection(BinaryDetection::convert(0))
        // Safety: "the caller guarantees that the underlying file won’t be mutated."
        // As we only read from the the Nix store, this is fine.
        .memory_map(unsafe { MmapChoice::auto() })
        .build()
        .search_path(
            &matcher,
            file_path,
            Lossy(|_lnum, line| {
                // We are guaranteed to find a match, so the unwrap is OK.
                let mymatch = matcher.find(line.as_bytes())?.unwrap();
                matches.push(line[mymatch].into());
                Ok(true)
            }),
        )?;
    Ok(matches)
}
