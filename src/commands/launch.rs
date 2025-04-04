use crate::{includes::EMBEDDED_ARIA2C_TARBALL, ui::LaunchUI};
use anyhow::{Context, Result, bail};
use bytes::{Buf, Bytes};
use clap::Parser;
use flate2::bufread::GzDecoder;
use log::{debug, error, info};
use reqwest::Url;
use std::{
    env,
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::PathBuf,
    primitive,
    str::FromStr,
};
use tar::Archive;
use tokio::process::Command;

const XIVLAUNCHER_BIN_FILENAME: &str = "XIVLauncher.Core";
const XIVLAUNCHER_VERSION_REMOTE_FILENAME: &str = "version";
const XIVLAUNCHER_VERSIONDATA_LOCAL_FILENAME: &str = "versiondata";
const ARIA2C_BIN_FILENAME: &str = "aria2c";

/// Install or update XIVLauncher and then open it.
#[derive(Debug, Clone, Parser)]
pub struct LaunchCommand {
    /// The name of the GitHub repository owner for XIVLauncher.
    #[clap(default_value = "goatcorp", long = "xlcore-repo-owner")]
    xlcore_repo_owner: String,

    /// The name of the GitHub repository for XIVLauncher.
    #[clap(default_value = "XIVLauncher.Core", long = "xlcore-repo-name")]
    xlcore_repo_name: String,

    /// The name of the release tar.gz archive that contains a self-contained XIVLauncher.
    #[clap(
        default_value = "XIVLauncher.Core.tar.gz",
        long = "xlcore-release-asset"
    )]
    xlcore_release_asset: String,

    /// The URL to a release of XIVLauncher.Core. This conflicts with `xlcore-repo-owner` and `xlcore-repo-name`
    /// as it overrides the default git-based release system.
    ///
    /// This should be a URL base that contains the following under it:
    ///
    /// - A plaintext file named `version` that contains only a version number.
    ///
    /// - A tar.gz archive with the name of `--xlcore-release-asset` that contains XIVLauncher files.
    #[clap(
        long = "xlcore-web-release-url-base",
        conflicts_with = "xlcore_repo_name",
        conflicts_with = "xlcore_repo_owner"
    )]
    xlcore_web_release_url_base: Option<Url>,

    /// Source of an aria2c tarball containing a statically compiled `aria2c` binary.
    /// By default an embedded tarball will be used.
    ///
    /// The supported source types are `file:path`, `url:url` or `embedded`.
    #[clap(long = "aria-source", default_value_t = AriaSource::Embedded)]
    aria_source: AriaSource,

    /// The path to where XIVLauncher should be installed.
    #[clap(default_value = dirs::data_local_dir().unwrap().join("xlcore").into_os_string(), long = "install-directory")]
    install_directory: PathBuf,

    /// Use XIVLauncher's fallback secrets provider instead of the system's `libsecret` provider.
    ///
    /// This should be used when no compatible system secrets provider is available where
    /// credential saving is still desirable.
    #[clap(long = "use-fallback-secret-provider")]
    use_fallback_secret_provider: bool,

    /// Run the launcher in Steam compatibility tool mode.
    ///
    /// This should be disabled if launching standalone instead of from a Steam compatibility tool.
    #[clap(default_value_t = true, long = "run-as-steam-compat-tool")]
    run_as_steam_compat_tool: primitive::bool,

    /// Skip checking for & installing new XIVLauncher versions.
    ///
    /// Note: this will not prevent XIVLauncher from installing when not present.
    #[clap(long = "skip-update")]
    skip_update: bool,
}

impl LaunchCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        info!("Attempting launch with: {self:?}");

        // Query the GitHub API or Web Release URL for release information.
        let release = match self.xlcore_web_release_url_base {
            Some(url) => {
                ReleaseAssetInfo::from_url(
                    url,
                    &self.xlcore_release_asset,
                    XIVLAUNCHER_VERSION_REMOTE_FILENAME,
                )
                .await?
            }
            None => {
                ReleaseAssetInfo::from_github(
                    &self.xlcore_repo_owner,
                    &self.xlcore_repo_name,
                    &self.xlcore_release_asset,
                )
                .await?
            }
        };

        // Conditionally run update check/install depending on flags and versions.
        let xl_installed = fs::exists(self.install_directory.join(XIVLAUNCHER_BIN_FILENAME))?;
        if xl_installed && self.skip_update {
            info!(
                "XIVLauncher already installed & version checks are disabled, skipping the update process"
            );
        } else {
            match fs::read_to_string(
                self.install_directory
                    .join(XIVLAUNCHER_VERSIONDATA_LOCAL_FILENAME),
            ) {
                Ok(local_ver) => {
                    if xl_installed && local_ver == release.version {
                        info!(
                            "XIVLauncher is up to date (local: {local_ver} == remote: {})",
                            release.version
                        );
                    } else {
                        let launch_ui = LaunchUI::new();
                        info!(
                            "XIVLauncher is out of date or missing files (local {local_ver} != remote: {}, bin present: {xl_installed}) - starting update",
                            release.version
                        );
                        install_or_update_xlcore(
                            release,
                            self.aria_source,
                            &self.install_directory,
                            true,
                            |txt| {
                                debug!("setting progress text to '{txt}'");
                                launch_ui.set_progress_text(txt);
                            },
                        )
                        .await?;
                        info!("Successfully updated XIVLauncher to the latest version")
                    }
                }
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        let launch_ui = LaunchUI::new();
                        info!(
                            "Unable to obtain local version data for XIVLauncher - installing latest release"
                        );
                        install_or_update_xlcore(
                            release,
                            self.aria_source,
                            &self.install_directory,
                            false,
                            |txt| {
                                debug!("setting progress text to '{txt}'");
                                launch_ui.set_progress_text(txt);
                            },
                        )
                        .await?;
                        info!("Successfully installed XIVLauncher");
                    } else {
                        error!(
                            "Something went wrong whilst checking for XIVLauncher: {:?}",
                            err
                        );
                    }
                }
            };
        }

        info!("Starting XIVLauncher");
        let mut cmd = Command::new(self.install_directory.join(XIVLAUNCHER_BIN_FILENAME));
        if self.use_fallback_secret_provider {
            cmd.env("XL_SECRET_PROVIDER", "FILE");
        }
        if self.run_as_steam_compat_tool {
            cmd.env("XL_SCT", "1"); // Needed to trigger compatibility tool mode in XIVLauncher. Otherwise XL_PRELOAD will be ignored.
        }
        cmd.env("XL_PRELOAD", env::var("LD_PRELOAD").unwrap_or_default()) // Write XL_PRELOAD so it can maybe be passed to the game later.
            .env_remove("LD_PRELOAD") // Completely remove LD_PRELOAD otherwise steam overlay will break the launcher text.
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }
}

/// Create/Overwrite an XLCore installation.
pub async fn install_or_update_xlcore<F: Fn(&str)>(
    release: ReleaseAssetInfo,
    aria_source: AriaSource,
    install_location: &PathBuf,
    is_update: bool,
    progress_msg_cb: F,
) -> anyhow::Result<()> {
    // Download and create archive readers for required files.
    let mut xlcore_archive = {
        match is_update {
            true => {
                info!("Downloading XIVLauncher from {}", release.download_url);
                progress_msg_cb(&format!("Downloading XIVLauncher (v{})", release.version));
            }
            false => {
                info!("Updating XIVLauncher from {}", release.download_url);
                progress_msg_cb(&format!("Updating XIVLauncher (v{})", release.version));
            }
        }

        let response = reqwest::get(release.download_url).await?;
        let bytes = response.bytes().await?;
        Archive::new(GzDecoder::new(bytes.reader()))
    };
    let mut aria_archive = {
        match aria_source {
            AriaSource::Embedded => {
                info!("Using embedded aria2c tarball");
                Archive::new(GzDecoder::new(
                    Bytes::from_static(EMBEDDED_ARIA2C_TARBALL).reader(),
                ))
            }
            AriaSource::Url(url) => {
                info!("Downloading remote aria2c tarball from {url}");
                progress_msg_cb("Downloading aria2c");
                let response: reqwest::Response = reqwest::get(url).await?;
                Archive::new(GzDecoder::new(response.bytes().await?.reader()))
            }
            AriaSource::File(path) => {
                info!("Using local aria2c tarball at path: {path:?}");
                Archive::new(GzDecoder::new(Bytes::from(fs::read(path)?).reader()))
            }
        }
    };

    // Cleanup old install.
    let _ = fs::remove_dir_all(install_location);
    fs::create_dir_all(install_location)?;

    // Unpack XLCore
    info!("Unpacking XIVLauncher tarball");
    progress_msg_cb("Extracting XIVLauncher");
    xlcore_archive.unpack(install_location)?;
    drop(xlcore_archive);
    info!("Ensuring XIVLauncher tarball contained compatible files");
    progress_msg_cb("Validating XIVLauncher files");
    if !fs::exists(install_location.join(XIVLAUNCHER_BIN_FILENAME))? {
        bail!(
            "XIVLauncher tarball does not contain a file named '{}' and is incompatible with XLM.",
            XIVLAUNCHER_BIN_FILENAME
        )
    }
    info!("Successfully extracted and wrote XIVLauncher files");

    // Unpack Aria2c
    info!("Unpacking aria2c tarball");
    progress_msg_cb("Unpacking aria2c");
    aria_archive.unpack(install_location)?;
    drop(aria_archive);
    info!("Ensuring aria2c tarball contained compatible files");
    progress_msg_cb("Validating aria2c files");
    if !fs::exists(install_location.join(ARIA2C_BIN_FILENAME))? {
        bail!(
            "aria2c tarball does not contain a file named '{}' and is incompatible with XLM.",
            ARIA2C_BIN_FILENAME
        )
    }
    info!("Successfully extracted and wrote aria2c files");

    // Complete installation by writing version information.
    progress_msg_cb("Writing version data");
    let mut file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(install_location.join(XIVLAUNCHER_VERSIONDATA_LOCAL_FILENAME))?;
    file.write_all(release.version.as_bytes())?;
    info!("Wrote version data (version {})", release.version);
    progress_msg_cb("Finishing up");

    Ok(())
}

#[derive(Default, Clone, Debug)]
pub enum AriaSource {
    #[default]
    Embedded,
    Url(Url),
    File(PathBuf),
}

impl FromStr for AriaSource {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "embedded" => Ok(Self::Embedded),
            _ if s.starts_with("url:") => Ok(Self::Url(
                Url::parse(&s.chars().skip(4).collect::<String>()).unwrap(),
            )),
            _ if s.starts_with("file:") => {
                let s = s.chars().skip(5).collect::<String>();
                if !fs::exists(&s)
                    .context("exists check operation failed")
                    .unwrap()
                {
                    return Err("unable to find file at given path");
                }
                Ok(Self::File(PathBuf::from(s)))
            }
            _ => Err("valid sources are 'embedded', 'url:' or 'file:'"),
        }
    }
}

impl Display for AriaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            AriaSource::Embedded => write!(f, "embedded"),
            AriaSource::File(_) => write!(f, "file:"),
            AriaSource::Url(_) => write!(f, "url:"),
        }
    }
}

pub struct ReleaseAssetInfo {
    pub download_url: Url,
    pub version: String,
}

impl ReleaseAssetInfo {
    /// Obtain [`ReleaseAssetInfo`] from the GitHub API.
    pub async fn from_github(
        repo_owner: &String,
        repo_name: &String,
        release_asset: &String,
    ) -> Result<Self> {
        let release = {
            match octocrab::instance()
                .repos(repo_owner, repo_name)
                .releases()
                .get_latest()
                .await
            {
                Ok(release) => release,
                Err(err) => {
                    bail!(
                        "Failed to obtain release information for {}/{}: {:?}",
                        repo_owner,
                        repo_name,
                        err.source()
                    );
                }
            }
        };

        match release
            .assets
            .into_iter()
            .find(|asset| &asset.name == release_asset)
        {
            Some(asset) => Ok(Self {
                download_url: asset.browser_download_url,
                version: release.tag_name,
            }),
            None => {
                bail!(
                    "Failed to find asset {} in release {}",
                    release_asset,
                    release.tag_name
                );
            }
        }
    }

    /// Obtain [`ReleaseAssetInfo`] from a web URL.
    pub async fn from_url(base_url: Url, release_asset: &str, version_asset: &str) -> Result<Self> {
        let (release_url, version_url) =
            (base_url.join(release_asset)?, base_url.join(version_asset)?);

        debug!("release asset url:{}", release_url);
        debug!("release version url: {}", version_url);

        let response = reqwest::get(version_url).await?;
        if !response.status().is_success() {
            bail!("{}", format!("{:?}", response.status().canonical_reason()))
        }

        Ok(Self {
            download_url: release_url,
            version: response.text().await?,
        })
    }
}
