use crate::{util, ExtractCmd};
use clap::Args;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, QueueableCommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Args)]
pub struct HashedSubcommand {
    /// The path to the `.minecraft/assets/` directory to extract assets from.
    ///
    /// Defaults to the default `.minecraft/assets/` folder location on your OS.
    #[arg(value_name = "ASSETS DIRECTORY")]
    hashed_assets_dir: Option<PathBuf>,
    /// The index file to use.
    ///
    /// Can either be a file path to the index file itself, or the name of
    /// that version (e.g. `24` instead of `.minecraft/assets/indexes/24.json`).
    #[arg(short, long, value_name = "FILE or VERSION", value_parser = IndexFileLocation::parse)]
    index: IndexFileLocation,
}

/// The location of the index file to use.
#[derive(Clone)]
enum IndexFileLocation {
    /// A file path to the index file itself.
    File(PathBuf),
    /// The name of the index file's version (e.g. `24` instead of `.minecraft/assets/indexes/24.json).
    Version(String),
}

impl IndexFileLocation {
    fn parse(input: &str) -> Result<Self, Infallible> {
        let path = Path::new(input);

        Ok(if path.is_file() {
            IndexFileLocation::File(path.to_owned())
        } else {
            IndexFileLocation::Version(input.to_owned())
        })
    }
}

/// Represents the contents of an index file in `.minecraft/assets/indexes`.
#[derive(Deserialize)]
pub struct IndexFile {
    /// A map of file paths within `assets` and the associated [`Object`].
    objects: HashMap<PathBuf, Object>,
}

/// Information about a hashed file.
#[derive(Deserialize)]
struct Object {
    /// The hashed name of the file.
    #[serde(rename = "hash")]
    hashed_file_name: String,
    /// The size of the file in bytes.
    #[serde(rename = "size")]
    _size: usize,
}

impl Object {
    /// Returns the name of the folder the hashed file is within inside the `objects` folder.
    ///
    /// The name of that folder will be the same as the first two characters of
    /// [the hashed file's name](#field.hashed_file).
    fn parent_dir(&self) -> &Path {
        Path::new(&self.hashed_file_name[..2])
    }

    /// Returns the path to the hashed file within the `objects` folder.
    pub fn hashed_file_path(&self) -> PathBuf {
        [self.parent_dir(), self.hashed_file_name.as_ref()]
            .iter()
            .collect()
    }
}

impl ExtractCmd for HashedSubcommand {
    fn execute(self, mut output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        let input_dir = self
            .hashed_assets_dir
            .or_else(|| util::minecraft_dir().map(|path| path.join("assets")))
            .filter(|path| path.is_dir())
            .expect("No input directory found");

        let output_dir = {
            if !ignore_top_level {
                output_dir.push("assets");
            }
            output_dir
        };

        let objects_dir = input_dir.join("objects");
        let indexes_dir = input_dir.join("indexes");

        let index_file = match self.index {
            IndexFileLocation::File(file) => file,

            IndexFileLocation::Version(version) => {
                let path = indexes_dir.join(format!("{version}.json"));

                if !path.is_file() {
                    panic!("No index file found at {}", path.display());
                }

                path
            }
        };

        let index: IndexFile = serde_json::from_str(&fs::read_to_string(index_file)?)?;
        let objects_len = index.objects.len();

        let mut stdout = io::stdout();

        for (i, (file_path, object)) in index.objects.iter().enumerate() {
            let file_name = file_path.display();

            // Print extraction progress (overwriting the previous progress message)
            // The cursor position is saved and restored to ensure it doesn't move all over the place.
            stdout.queue(cursor::SavePosition)?;
            stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
            stdout.write_all(format!("Extracting {}/{objects_len}", i + 1).as_bytes())?;
            stdout.queue(cursor::RestorePosition)?;

            stdout.flush()?;

            // Read the hashed file
            match fs::read(objects_dir.join(object.hashed_file_path())) {
                Ok(contents) => {
                    let output_file = output_dir.join(&file_path);

                    // Fill in parent directories of the file, since Windows doesn't do that.
                    if let Some(Err(error)) = output_file.parent().map(fs::create_dir_all) {
                        eprintln!("Failed to create parent directories for '{file_name}': {error}");
                    }

                    // Copy the file contents
                    if let Err(error) = fs::write(output_file, contents) {
                        eprintln!("Failed to write file '{file_name}': {error}");
                    }
                }

                Err(error) => {
                    eprintln!("Skipping '{file_name}': failed to read hashed file: {error}")
                }
            }
        }

        Ok(())
    }
}
