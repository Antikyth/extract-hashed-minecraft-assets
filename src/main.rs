use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// The path to the `.minecraft/assets` directory to extract assets from
    input_directory: PathBuf,
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
    hash: String,
    /// The size of the file in bytes.
    _size: usize,
}

impl Object {
    /// Returns the name of the folder the hashed file is within inside the `objects` folder.
    pub fn parent_dir(&self) -> &str {
        &self.hash[..2]
    }
}

fn main() {
    let Args {
        input_directory,
        output_directory,
        version,
    } = Args::parse();

    let input_directory = input_directory.to_path_buf();
    if !input_directory.is_dir() {
        panic!("Expected input to be a directory");
    }

    let output_directory = output_directory
        .or_else(|| std::env::current_dir().ok())
        .expect("No output directory found");
    if !output_directory.is_dir() {
        panic!("Expected output to be a directory");
    }

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
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .expect("No index file found");

    for (file_name, object) in indexes.objects {
        let hashed_file_path = objects_dir.join(object.parent_dir()).join(object.hash);

        // Read the hashed file
        match fs::read(hashed_file_path) {
            Ok(contents) => {
                let output_file = output_directory.join(&file_name);

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
}
