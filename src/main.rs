mod hashed;
mod jar;
mod util;

use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;
use std::{env, io};

/// Extracts hashed Minecraft assets.
///
/// Most Minecraft assets are located within a version's jar file (e.g.
/// `.minecraft/versions/1.20.1.jar`), but certain assets (like sounds or
/// non-US-English languages) are found in `.minecraft/assets/`, with hashed
/// file names instead of their file path within Minecraft's assets. This tool
/// extracts all those hashed files based on the file path they should have.
#[derive(Parser)]
struct ExtractCommand {
    #[command(subcommand)]
    subcommand: ExtractSubcommand,

    /// The path to the directory into which to extract assets.
    ///
    /// Defaults to the current directory.
    #[arg(short, long = "output", value_name = "DIRECTORY")]
    output_dir: Option<PathBuf>,
    /// Whether to extract the contents directly into the output directory.
    ///
    /// If this is not set, `assets`/`data` directories will be created in the
    /// output directory.
    ///
    /// If it is set, the contents of those directories will be placed directly
    /// into the output directory.
    ///
    /// You probably don't want to use this if you're using the jar subcommand
    /// to extract both `assets` and `data` at the same time, as their contents
    /// would get mixed up.
    #[arg(long)]
    ignore_top_level: bool,
}

#[derive(Subcommand)]
enum ExtractSubcommand {
    /// Extracts hashed Minecraft assets (e.g. from `.minecraft/assets/`).
    Hashed(hashed::HashedSubcommand),
    /// Extracts non-hashed Minecraft `assets`, or `data`, from a Minecraft jar (or zip) file.
    Jar(jar::JarSubcommand),
}

impl ExtractSubcommand {
    fn execute(self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        match self {
            Self::Hashed(subcommand) => subcommand.execute(output_dir, ignore_top_level),
            Self::Jar(subcommand) => subcommand.execute(output_dir, ignore_top_level),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ExtractCommand {
        subcommand,
        output_dir,
        ignore_top_level,
    } = ExtractCommand::parse();

    let output_dir = output_dir.unwrap_or(env::current_dir()?);
    if output_dir.is_dir() {
        subcommand.execute(output_dir, ignore_top_level)?;
    } else {
        panic!(
            "'{}' does not exist or is not a directory",
            output_dir.display()
        );
    }

    Ok(())
}
