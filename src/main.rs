mod hashed;
mod jar;
mod util;
mod version;

use clap::{Parser, Subcommand};
use crossterm::terminal::ClearType;
use crossterm::{terminal, ExecutableCommand};
use std::path::PathBuf;
use std::{env, io};

/// Extracts Minecraft `assets` or `data`.
///
/// Most Minecraft assets are located within a version's jar file, while some
/// (like sounds or non-US-English languages) are found hashed in
/// `.minecraft/assets/`. This tool can extract `assets` or `data` from either
/// location, or both at the same time.
#[derive(Parser)]
struct ExtractCommand {
    #[command(subcommand)]
    subcommand: ExtractSubcommand,

    /// The path to the directory into which to extract assets.
    ///
    /// Defaults to the current directory.
    #[arg(short, long = "output", value_name = "DIRECTORY", global = true)]
    output_dir: Option<PathBuf>,
    /// Whether to extract the contents directly into the output directory.
    ///
    /// If this is not set, `assets`/`data` directories will be created in the
    /// output directory.
    ///
    /// If it is set, the contents of those directories will be placed directly
    /// into the output directory.
    ///
    /// You probably don't want to use this if you're extracting both `assets`
    /// and `data` at the same time, as their contents would get mixed up.
    #[arg(long, global = true)]
    ignore_top_level: bool,
}

#[derive(Subcommand)]
enum ExtractSubcommand {
    /// Extracts hashed Minecraft assets.
    ///
    /// The usual location for hashed assets is `.minecraft/assets`.
    Hashed(hashed::HashedSubcommand),
    /// Extracts non-hashed Minecraft `assets`, or `data`.
    ///
    /// `assets` and/or `data` are extracted from a Minecraft version zip archive
    /// file (usually a `.jar` file in `.minecraft/versions/<version>/<version>.jar`).
    Jar(jar::JarSubcommand),
    /// Extracts both hashed and non-hashed Minecraft `assets`, or `data`.
    Version(version::VersionSubcommand),
}

trait ExtractCmd {
    /// Executes and consumes the subcommand.
    fn execute(self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()>;
}

impl ExtractCmd for ExtractSubcommand {
    fn execute(self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        match self {
            Self::Hashed(subcommand) => subcommand.execute(output_dir, ignore_top_level),
            Self::Jar(subcommand) => subcommand.execute(output_dir, ignore_top_level),
            Self::Version(subcommand) => subcommand.execute(output_dir, ignore_top_level),
        }
    }
}

fn main() -> io::Result<()> {
    let ExtractCommand {
        subcommand,
        output_dir,
        ignore_top_level,
    } = ExtractCommand::parse();

    let output_dir = output_dir.map(Ok).unwrap_or_else(|| env::current_dir())?;

    if output_dir.is_dir() {
        let result = subcommand.execute(output_dir, ignore_top_level);

        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(ClearType::FromCursorDown))?;

        match &result {
            Ok(_) => println!("Extraction complete"),
            Err(error) => eprintln!("Extraction failed: {error}"),
        }

        result
    } else {
        panic!(
            "'{}' does not exist or is not a directory",
            output_dir.display()
        );
    }
}
