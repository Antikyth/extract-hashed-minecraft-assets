use clap::Parser;
use crossterm::{cursor, QueueableCommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser)]
struct Args {
    /// The path to the `.minecraft/assets` directory to extract assets from.
    ///
    /// Defaults to the default `.minecraft/assets` folder location on your OS.
    input_directory: Option<PathBuf>,
    /// The path to the `assets` directory to extract assets to (not the `minecraft` folder inside).
    ///
    /// Defaults to the current directory.
    #[arg(short, long = "output")]
    output_directory: Option<PathBuf>,

    /// The version file in `indexes` to use (without the `.json` suffix).
    ///
    /// Defaults to the last file in the `indexes` folder (which is OS- and filesystem-dependent).
    #[arg(short, long)]
    version: Option<OsString>,
}

#[derive(Deserialize)]
struct IndexJson {
    objects: HashMap<String, Object>,
}

/// The hashed name of a file and its size.
#[derive(Deserialize)]
struct Object {
    /// The hashed name of the file.
    #[serde(rename = "hash")]
    hashed_file: String,
    /// The size of the file in bytes.
    #[serde(rename = "size")]
    _size: usize,
}

impl Object {
    /// Returns the name of the folder the hashed file is within inside the `objects` folder.
    pub fn parent_dir(&self) -> &str {
        &self.hashed_file[..2]
    }
}

// Windows
#[cfg(target_os = "windows")]
fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join(".minecraft"))
        .filter(|path| path.is_dir())
}

// Mac
#[cfg(target_os = "macos")]
fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join("minecraft"))
        .filter(|path| path.is_dir())
}

// Linux
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn minecraft_dir() -> Option<PathBuf> {
    dirs::home_dir()
        .map(|path| path.join(".minecraft"))
        .filter(|path| path.is_dir())
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args {
        input_directory,
        output_directory,
        version,
    } = Args::parse();

    let input_directory = input_directory
        .or_else(|| minecraft_dir().map(|path| path.join("assets")))
        .filter(|path| path.is_dir())
        .expect("No input directory found");
    let output_directory = output_directory
        .or_else(|| std::env::current_dir().ok())
        .filter(|path| path.is_dir())
        .expect("No output directory found");

    let indexes_dir = input_directory.join("indexes");
    let objects_dir = input_directory.join("objects");

    let indexes: IndexJson = version
        .map(|mut version| {
            version.push(".json");

            version
        })
        .or_else(|| {
            let mut version_files: Vec<_> = indexes_dir
                .read_dir()
                .expect("Failed to read indexes")
                .filter_map(Result::ok)
                .map(|entry| entry.file_name())
                .collect();

            // Use the last version file
            version_files.pop()
        })
        .map(|file_name| indexes_dir.join(file_name))
        .map(|path| fs::read_to_string(path).expect("Failed to read index file"))
        .map(|contents| serde_json::from_str(&contents).expect("Failed to parse index file"))
        .expect("No index file found");
    let objects_len = indexes.objects.len();

    let mut stdout = io::stdout();

    for (i, (file_path, object)) in indexes.objects.iter().enumerate() {
        let file_path = PathBuf::from(&file_path);
        let file_name = file_path.display();

        // Print extraction progress (overwriting the previous progress message)
        // The cursor position is saved and restored to ensure it doesn't move all over the place.
        stdout.queue(cursor::SavePosition)?;
        stdout.write_all(format!("Extracting {}/{objects_len}", i + 1).as_bytes())?;
        stdout.queue(cursor::RestorePosition)?;

        stdout.flush()?;

        let hashed_file_path = objects_dir
            .join(object.parent_dir())
            .join(&object.hashed_file);

        // Read the hashed file
        match fs::read(hashed_file_path) {
            Ok(contents) => {
                let output_file = output_directory.join(&file_path);

                // Fill in parent directories of the file, since Windows doesn't do that.
                if let Some(Err(error)) = output_file.parent().map(fs::create_dir_all) {
                    eprintln!("Failed to create parent directories for '{file_name}': {error}");
                }

                // Copy the file contents
                if let Err(error) = fs::write(output_file, contents) {
                    eprintln!("Failed to write file '{file_name}': {error}");
                }
            }

            Err(error) => eprintln!("Skipping '{file_name}': failed to read hashed file: {error}"),
        }
    }

    Ok(())
}
