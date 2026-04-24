use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

use crate::obfuscate;

// File system commands

//pwd 
pub fn pwd() -> String {
    match std::env::current_dir() {
        Ok(current_dir) => current_dir.to_string_lossy().to_string(),
        Err(e) => {
            let err = format!("{}{}", obfuscate!("Failed to get current directory: "), e);
            err
        }
    }
}   

//ls <path>
pub fn ls(path: &str) -> String {

    let mut output = format!("#{}\n", path);

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return format!("{e}"),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => return format!("{e}"),
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(e) => return format!("Failed to get metadata: {}", e),
        };

        output += &format!(" + {}\n", entry.file_name().to_string_lossy());
    }

    output
}

// cd <path>
pub fn change_dir(path: &str) -> String {
    if let Err(e) = std::env::set_current_dir(path) {
        let err = format!("{}{}", obfuscate!("Failed to change directory: "), e);
        return err;
    }

	pwd()
}

pub fn search(directory: &PathBuf, file_name: &str) -> Result<String> {

    match std::fs::read_dir(directory) {
        Ok(entries) => {

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                // search the entry if it is a directory
                if path.is_dir(){
                    if let Ok(result) = search(&path, file_name) {
                        return Ok(result);
                    }
                };
                if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(file_name) {
                    return Ok(path.to_string_lossy().to_string());
                }

            }
        },
        Err(e) => bail!(e),
    }

    let e = format!("No results found for {} inside {}", directory.display(), file_name);
    bail!(e);
}