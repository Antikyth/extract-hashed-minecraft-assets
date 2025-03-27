use clap::Args;
use std::io;
use std::path::PathBuf;

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

#[derive(Args)]
#[group(multiple = true, required = true)]
#[derive(Clone)]
struct ExtractedContents {
    /// Extract the `assets` folder.
    ///
    /// Can be combined with --data.
    #[arg(long)]
    assets: bool,
    /// Extract the `data` folder.
    ///
    /// Can be combined with --assets.
    #[arg(long)]
    data: bool,
}

impl JarSubcommand {
    pub fn execute(&self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        Ok(())
    }
}
