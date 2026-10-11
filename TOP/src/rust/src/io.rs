//! Files: listing and loading the instances of a folder, writing text outputs.

use crate::model::Instance;
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Instance files (*.txt) of a folder, sorted by name.
pub fn instance_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    Ok(files)
}

/// Load every instance of a folder (in parallel), sorted by name.
pub fn load_all(dir: &Path) -> Result<Vec<Instance>, String> {
    instance_files(dir)?.par_iter().map(|p| Instance::load(p)).collect()
}

/// Write a text file, creating its folder if needed.
pub fn write_file(out: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::File::create(out)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .map_err(|e| format!("{}: {e}", out.display()))
}
