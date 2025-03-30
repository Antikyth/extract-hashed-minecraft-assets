use crate::ExtractCmd;
use clap::Args;
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, QueueableCommand};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};
use zip::ZipArchive;

#[derive(Args)]
pub struct JarSubcommand {
    /// The jar or zip file to extract from.
    ///
    /// Minecraft version jar files can be found in `.minecraft/versions/`.
    #[arg(value_name = "FILE")]
    jar_file: PathBuf,
    /// Which contents to extract.
    #[command(flatten)]
    extracted_contents: ExtractedContents,
}

#[derive(Args, Clone, PartialEq, Eq, Hash, Debug)]
#[group(multiple = true, required = true)]
pub struct ExtractedContents {
    /// Extract the `assets` folder.
    ///
    /// Can be combined with --data.
    #[arg(short, long)]
    pub assets: bool,
    /// Extract the `data` folder.
    ///
    /// Can be combined with --assets.
    #[arg(short, long)]
    pub data: bool,
}

impl Default for ExtractedContents {
    fn default() -> Self {
        Self {
            assets: true,
            data: false,
        }
    }
}

impl Display for ExtractedContents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.assets, self.data) {
            (false, false) => write!(f, "nothing"),
            (true, false) => write!(f, "assets"),
            (false, true) => write!(f, "data"),
            (true, true) => write!(f, "assets and data"),
        }
    }
}

pub fn extract_jar(
    jar_file: &Path,
    output_dir: &Path,
    extracted_contents: ExtractedContents,
    ignore_top_level: bool,
) -> io::Result<()> {
    let assets = extracted_contents.assets.then(|| Path::new("assets"));
    let data = extracted_contents.data.then(|| Path::new("data"));

    if assets.is_none() && data.is_none() {
        return Ok(());
    }

    let mut archive = ZipArchive::new(File::open(&jar_file)?)?;
    let top_level_dir = archive.root_dir(zip::read::root_dir_common_filter)?;

    let mut stdout = io::stdout();

    // Why does ZipArchive not implement an iterator...?
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let path = match &top_level_dir {
            Some(top_level) => path.strip_prefix(top_level).unwrap_or(&path),
            None => &path,
        };
        let path = match (assets, data) {
            (Some(assets), _) if path.starts_with(assets) => {
                if ignore_top_level {
                    path.strip_prefix(assets).unwrap_or(path)
                } else {
                    path
                }
            }
            (_, Some(data)) if path.starts_with(data) => {
                if ignore_top_level {
                    path.strip_prefix(data).unwrap_or(path)
                } else {
                    path
                }
            }

            (_, _) => continue,
        };
        let output_path = output_dir.join(path);

        // Print file being extracted
        stdout.queue(cursor::SavePosition)?;
        stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
        stdout.write_all(format!("Extracting {}", path.display()).as_bytes())?;
        stdout.queue(cursor::RestorePosition)?;

        stdout.flush()?;

        if file.is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy the file
            let mut output_file = File::create(&output_path)?;
            io::copy(&mut file, &mut output_file)?;
        }

        // Set the file permissions on unix
        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&output_path, Permissions::from_mode(mode))?;
        }
    }

    Ok(())
}

impl ExtractCmd for JarSubcommand {
    fn execute(self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        extract_jar(
            &self.jar_file,
            &output_dir,
            self.extracted_contents,
            ignore_top_level,
        )
    }
}
