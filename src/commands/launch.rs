use crate::{includes::ARIA2C_TARBALL_CONTENT, ui::LaunchUI};
use anyhow::{bail, Context, Result};
use bytes::{Buf, Bytes};
use clap::Parser;
use flate2::read::GzDecoder;
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

#[derive(Default, Clone, Debug)]
enum AriaSource {
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
    /// This should be a URL prefix that contains:
    ///
    /// - A file called `version` that contains the version of the release.
    ///
    /// - A file with the name of `<xlcore-release-asset>` that contains the tar.gz archive of the release.
    #[clap(
        long = "xlcore-web-release-url-base",
        conflicts_with = "xlcore_repo_name",
        conflicts_with = "xlcore_repo_owner"
    )]
    xlcore_web_release_url_base: Option<Url>,

    /// The source of the aria2c tarball containing a static compiled 'aria2c' binary.
    /// By default an embedded tarball will be used requiring no downloads.
    ///
    /// The supported source types are `file:`, `url:` or `embedded`.
    #[clap(long = "aria-source", default_value_t = AriaSource::Embedded)]
    aria_source: AriaSource,

    /// The location where the XIVLauncher should be installed.
    #[clap(default_value = dirs::data_local_dir().unwrap().join("xlcore").into_os_string(), long = "install-directory")]
    install_directory: PathBuf,

    /// Use a fallback secrets provider with XIVLauncher instead of the system provided.
    /// Used when no system secrets provider is available and credentials should still be saved.
    #[clap(long = "use-fallback-secret-provider")]
    use_fallback_secret_provider: bool,

    /// Run the launcher in Steam compatibility tool mode.
    ///
    /// This should be disabled if launching standalone not from a Steam compatibility tool.
    #[clap(default_value_t = true, long = "run-as-steam-compat-tool")]
    run_as_steam_compat_tool: primitive::bool,

    /// Skip checking for XIVLauncher updates. This will not prevent XIVLauncher from installing if it isn't installed.
    #[clap(long = "skip-update")]
    skip_update: bool,
}

impl LaunchCommand {
    pub async fn run(self) -> anyhow::Result<()> {
        debug!("Attempting launch with args: {self:?}");

        // Query the GitHub API or web release Url for release information.
        let (remote_version, remote_release_url) = match self.xlcore_web_release_url_base {
            Some(url) => Self::get_release_web(url, &self.xlcore_release_asset).await?,
            None => {
                Self::get_release_github(
                    &self.xlcore_repo_owner,
                    &self.xlcore_repo_name,
                    &self.xlcore_release_asset,
                )
                .await?
            }
        };

        // Install XIVLauncher or do an update check if version data already exists locally.
        match fs::read_to_string(
            self.install_directory
                .join(XIVLAUNCHER_VERSIONDATA_LOCAL_FILENAME),
        ) {
            Ok(ver) => {
                if !self.skip_update {
                    if ver == remote_version {
                        info!(
                            "XIVLauncher is up to date! (local: {ver} == remote: {remote_version})"
                        );
                    } else {
                        let mut launch_ui = LaunchUI::new();
                        info!("XIVLauncher is out of date (local {ver} != remote: {remote_version}) - starting update");
                        Self::install_or_update_xlcore(
                            &remote_version,
                            remote_release_url,
                            self.aria_source,
                            &self.install_directory,
                            &mut launch_ui,
                        )
                        .await?;
                        info!("Successfully updated XIVLauncher to the latest version.")
                    }
                } else {
                    info!("Skip update enabled, not attempting to update XIVLauncher.")
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    let mut launch_ui = LaunchUI::new();
                    info!("Unable to obtain local version data for XIVLauncher - installing latest release");
                    Self::install_or_update_xlcore(
                        &remote_version,
                        remote_release_url,
                        self.aria_source,
                        &self.install_directory,
                        &mut launch_ui,
                    )
                    .await?;
                    info!("Successfully installed XIVLauncher")
                } else {
                    error!(
                        "Something went wrong whilst checking for XIVLauncher: {:?}",
                        err
                    );
                }
            }
        };

        info!("Starting XIVLauncher");

        let mut cmd = Command::new(self.install_directory.join(XIVLAUNCHER_BIN_FILENAME));
        if self.use_fallback_secret_provider {
            cmd.env("XL_SECRET_PROVIDER", "FILE");
        }
        if self.run_as_steam_compat_tool {
            cmd.env("XL_SCT", "1"); // Needed to trigger compatibility tool mode in XIVLauncher. Otherwise XL_PRELOAD will be ignored.
        }
        let cmd = cmd
            .env("XL_PRELOAD", env::var("LD_PRELOAD").unwrap_or_default()) // Write XL_PRELOAD so it can maybe be passed to the game later.
            .env_remove("LD_PRELOAD") // Completely remove LD_PRELOAD otherwise steam overlay will break the launcher text.
            .spawn()?
            .wait()
            .await?;

        info!("XIVLauncher process exited with exit code {:?}", cmd.code());

        Ok(())
    }

    async fn get_release_github(
        xlcore_repo_owner: &String,
        xlcore_repo_name: &String,
        xlcore_release_asset: &String,
    ) -> Result<(String, Url)> {
        let octocrab = octocrab::instance();
        let repo = octocrab.repos(xlcore_repo_owner, xlcore_repo_name);
        let release = match repo.releases().get_latest().await {
            Ok(release) => release,
            Err(err) => {
                bail!(
                    "Failed to obtain release information for {}/{}: {:?}",
                    xlcore_repo_owner,
                    xlcore_repo_name,
                    err.source()
                );
            }
        };

        let release_url = release
            .assets
            .iter()
            .find(|asset| &asset.name == xlcore_release_asset);

        if let Some(asset) = release_url {
            Ok((release.tag_name, asset.browser_download_url.clone()))
        } else {
            bail!(
                "Failed to find asset {} in release {}",
                xlcore_release_asset,
                release.tag_name
            );
        }
    }

    async fn get_release_web(base_url: Url, xlcore_release_asset: &str) -> Result<(String, Url)> {
        let version_url = base_url.join(XIVLAUNCHER_VERSION_REMOTE_FILENAME)?;
        let release_url = base_url.join(xlcore_release_asset)?;

        info!("XIVLauncher web release asset url:{}", release_url);
        info!("XIVLauncher web release version url: {}", version_url);

        let response = reqwest::get(version_url).await?;
        if !response.status().is_success() {
            bail!("{}", format!("{:?}", response.error_for_status()))
        }
        Ok((response.text().await?, release_url))
    }

    /// Creates a new XLCore installation or overwrites an existing XLCore installion with a new one.
    async fn install_or_update_xlcore(
        release_version: &String,
        release_url: Url,
        aria_source: AriaSource,
        install_location: &PathBuf,
        launch_ui: &mut LaunchUI,
    ) -> anyhow::Result<()> {
        // Download/extract XLCore.
        {
            info!("Downloading XIVLauncher release from {release_url}");
            launch_ui.set_progress_text("Downloading XIVLauncher");
            let response = reqwest::get(release_url).await?;
            let bytes = response.bytes().await?;
            let mut archive = Archive::new(GzDecoder::new(bytes.reader()));
            let _ = fs::remove_dir_all(install_location);
            fs::create_dir_all(install_location)?;
            info!("Unpacking XIVLauncher release tarball");
            launch_ui.set_progress_text("Extracting XIVLauncher");
            archive.unpack(install_location)?;
            info!("Wrote XIVLauncher files");
        }

        // Download/extract aria2c.
        {
            let aria_archive_bytes = match aria_source {
                AriaSource::Embedded => {
                    info!("Using embedded aria2c tarball");
                    Bytes::from_static(ARIA2C_TARBALL_CONTENT)
                }
                AriaSource::Url(url) => {
                    info!("Downloading remote aria2c tarball from {url}");
                    launch_ui.set_progress_text("Downloading aria2c");
                    let response: reqwest::Response = reqwest::get(url).await?;
                    response.bytes().await?
                }
                AriaSource::File(path) => {
                    info!("Using local aria2c tarball at path: {path:?}");
                    Bytes::from(fs::read(path)?)
                }
            };

            let mut archive = Archive::new(GzDecoder::new(aria_archive_bytes.reader()));

            info!("Unpacking aria2c tarball");
            launch_ui.set_progress_text("Unpacking aria2c");
            archive.unpack(install_location)?;

            info!("Ensuring aria2c tarball contained correct binary");
            launch_ui.set_progress_text("Ensuring aria2c compatibility");
            if !fs::exists(install_location.join("aria2c"))? {
                error!("aria2c tarball does not contain a binary named 'aria2c' and is unusable with XIVLauncher.");
                bail!("aria2c tarball does not contain a binary named 'aria2c' and is unusable with XIVLauncher.")
            }

            info!("Wrote aria2c binary");
        }

        // Write local version info for release.
        {
            launch_ui.set_progress_text("Writing XIVLauncher version data");
            let mut file = File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .append(false)
                .open(install_location.join(XIVLAUNCHER_VERSIONDATA_LOCAL_FILENAME))?;
            file.write_all(release_version.as_bytes())?;
            info!("Wrote versiondata with version {}", release_version);
        }
        launch_ui.set_progress_text("Finishing up");

        Ok(())
    }
}
