use crate::util::OptionExt;
use crate::{hashed, jar, util, ExtractCmd};
use clap::Args;
use serde::Deserialize;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Args)]
pub struct VersionSubcommand {
    /// The directory containing the version `.jar` file and manifest.
    ///
    /// Can be a path to the directory, or the name of the version to be found
    /// within `.minecraft/versions/`.
    ///
    /// The name of the `.jar` and `.json` manifest file inside must match the
    /// directory name.
    ///
    /// Example: `1.20.1` or `.minecraft/versions/1.20.1`
    #[arg(value_name = "DIRECTORY or VERSION", value_parser = Version::parse)]
    version_dir: Version,
    /// The path to the `.minecraft/assets/` directory to find hashed assets.
    ///
    /// Defaults to the default location on your OS.
    #[arg(long = "hashed-assets", value_name = "DIRECTORY")]
    hashed_assets_dir: Option<PathBuf>,

    /// Which contents to extract.
    #[command(flatten)]
    extracted_contents: jar::ExtractedContents,
}

/// Represents a directory containing the version `.jar` file and manifest.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Version {
    dir: PathBuf,
}

impl Version {
    /// Returns a new [`Version`] wrapping `dir`.
    fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The path to the version's directory.
    fn path(&self) -> &Path {
        &self.dir
    }

    /// The name of the version directory, [jar file](Version::jar_file), and
    /// [manifest file](Version::manifest_file).
    fn name(&self) -> &str {
        self.path()
            .file_name()
            .and_then(OsStr::to_str)
            .expect("Version directory has an invalid name")
    }

    /// The path to the version's jar file.
    fn jar_file(&self) -> PathBuf {
        self.path().join(format!("{}.jar", self.name()))
    }

    /// The path to the version's manifest file.
    fn manifest_file(&self) -> PathBuf {
        self.path().join(format!("{}.json", self.name()))
    }

    /// Parses `input` into a [`Version`].
    ///
    /// If there is neither a directory at the path specified by `input`, nor as
    /// a child of the [default `versions` directory location](util::versions_dir),
    /// an [`InvalidVersion`] error is returned.
    fn parse(input: &str) -> Result<Self, InvalidVersion> {
        let path = Path::new(input);

        if path.is_dir() {
            Ok(Self::new(path.to_owned()))
        } else {
            if let Some(path) = util::versions_dir()
                .inspect_mut(|dir| dir.push(path))
                .filter(|path| path.is_dir())
            {
                Ok(Self::new(path))
            } else {
                Err(InvalidVersion::new(input.to_owned()))
            }
        }
    }
}

/// Represents an error locating a [version directory](Version) during
/// [parsing](Version::parse).
#[derive(Debug)]
pub struct InvalidVersion {
    pub version: String,
}

impl InvalidVersion {
    fn new(version: String) -> Self {
        Self { version }
    }
}

impl Display for InvalidVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid version '{}': no directory exists of that path nor name within `minecraft/versions`",
            self.version
        )
    }
}

impl Error for InvalidVersion {}

/// Represents the manifest file for a version.
///
/// The manifest file has a lot of information: this representation only
/// includes what is necessary for identifying the hashed assets index.
#[derive(Deserialize)]
#[non_exhaustive]
struct ManifestFile {
    /// The name of the index file to be found within `.minecraft/assets/indexes/`,
    /// without the `json` file extension.
    #[serde(rename = "assets")]
    index_version: String,
}

impl ExtractCmd for VersionSubcommand {
    fn execute(self, output_dir: PathBuf, ignore_top_level: bool) -> io::Result<()> {
        if !self.extracted_contents.assets && !self.extracted_contents.data {
            return Ok(());
        }

        let manifest: Option<ManifestFile> = self
            .extracted_contents
            .assets
            .then(|| {
                serde_json::from_str(&fs::read_to_string(self.version_dir.manifest_file())?)
                    .map_err(Into::<io::Error>::into)
            })
            .transpose()?;
        let hashed_assets_dir = self
            .hashed_assets_dir
            .or_else(|| util::hashed_assets_dir())
            .filter(|path| path.is_dir())
            .expect("No hashed assets directory found");
        let index = manifest.map(|ManifestFile { index_version }| {
            let mut path: PathBuf = [
                &hashed_assets_dir,
                Path::new("indexes"),
                index_version.as_ref(),
            ]
            .iter()
            .collect();
            path.set_extension("json");

            assert!(
                path.is_file(),
                "No index file for hashed assets found at '{}'",
                path.display()
            );

            path
        });

        let jar = self.version_dir.jar_file();
        println!(
            "Extracting {} from {}...",
            &self.extracted_contents,
            jar.display()
        );
        jar::extract_jar(&jar, &output_dir, self.extracted_contents, ignore_top_level)?;

        if let Some(path) = index {
            println!("Extracting hashed assets using index {}...", path.display());
            hashed::extract_hashed_assets(&hashed_assets_dir, output_dir, path, ignore_top_level)?;
        }

        Ok(())
    }
}
