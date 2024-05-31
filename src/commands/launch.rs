use bytes::Buf;
use clap::Parser;
use flate2::read::GzDecoder;
use octocrab::models::repos::Release;
use std::{
    error::Error,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::PathBuf,
};
use tar::Archive;
use tokio::process::Command;

const OPENSSL_FIX_FILENAME: &'static str = "openssl_fix.cnf";
const XLCORE_VERSIONDATA_FILENAME: &'static str = "versiondata";

#[derive(Debug, Clone, Parser)]
pub struct LaunchCommand {
    /// The name of the GitHub repository owner for XIVLauncher.
    #[clap(default_value = "goatcorp", long = "repo-owner")]
    repo_owner: String,

    /// The name of the GitHub repository for XIVLauncher.
    #[clap(default_value = "XIVLauncher.Core", long = "repo-name")]
    repo_name: String,

    /// The name of the release asset that contains a self-contained XIVLauncher.
    #[clap(default_value = "XIVLauncher.Core.tar.gz", long = "release-asset-name")]
    release_asset_name: String,

    /// The location where the XIVLauncher should be installed.
    #[clap(default_value = dirs::data_local_dir().unwrap().join("xlcore").into_os_string(), long = "install-directory")]
    data_directory: PathBuf,

    /// Do not check to see if XIVLauncher is out of date on startup. This will not prevent XIVLauncher from installing if the update check fails.
    #[clap(default_value_t = false, long = "skip-update")]
    skip_update: bool,
}

impl LaunchCommand {
    pub async fn run(self) {
        // Query the GitHub API for release information.
        let octocrab = octocrab::instance();
        let repo = octocrab.repos(&self.repo_owner, &self.repo_name);
        let release = match repo.releases().get_latest().await {
            Ok(release) => release,
            Err(err) => {
                eprintln!(
                    "XLCM: Failed to obtain release information for {}/{}: {:?}",
                    self.repo_name,
                    self.repo_name,
                    err.source()
                );
                return;
            }
        };

        // Install XIVLauncher or do an update check if version data already exists.
        match fs::read_to_string(self.data_directory.join(XLCORE_VERSIONDATA_FILENAME)) {
            Ok(ver) => {
                if !self.skip_update {
                    if ver == release.tag_name {
                        println!("XLCM: Installed XIVLauncher is up to date!");
                    } else {
                        println!("XLCM: Installed XIVLauncher is out of date, starting updater...");
                        install_or_update_xlcore(
                            release,
                            &self.release_asset_name,
                            &self.data_directory,
                        )
                        .await
                        .unwrap();
                        println!("XLCM: Successfully updated XIVLauncher to the latest version.")
                    }
                } else {
                    println!("XLCM: Skip update enabled, not attempting to update XIVLauncher.")
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    println!("XLCM: Unable to obtain local version data for XIVLauncher, installing from latest tag...");
                    install_or_update_xlcore(
                        release,
                        &self.release_asset_name,
                        &self.data_directory,
                    )
                    .await
                    .unwrap();
                    println!("Successfully installed XIVLauncher.")
                } else {
                    eprint!(
                        "Something went wrong whilst checking for XIVLauncher: {:?}",
                        err
                    );
                }
            }
        };

        println!("XLCM: Starting XIVLauncher");
        let cmd = Command::new(self.data_directory.join("XIVLauncher.Core"))
            .env_remove("LD_PRELOAD")
            .env("OPENSSL_CONF", &self.data_directory.join("openssl_fix.cnf"))
            .spawn()
            .unwrap()
            .wait()
            .await
            .unwrap();
        println!(
            "XLCM: XIVLauncher process exited with exit code {:?}",
            cmd.code()
        );
    }
}

async fn install_or_update_xlcore(
    release: Release,
    release_asset_name: &String,
    install_location: &PathBuf,
) -> Result<(), ()> {
    for asset in release.assets {
        if asset.name != *release_asset_name {
            continue;
        }

        // Download the release.
        println!(
            "XLCM: Downloading release from {}",
            asset.browser_download_url,
        );
        let response = reqwest::get(asset.browser_download_url).await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let mut archive = Archive::new(GzDecoder::new(bytes.reader()));

        let _ = fs::remove_dir_all(install_location);
        fs::create_dir_all(install_location).unwrap();

        // Unpack the archive.
        println!("XLCM: Unpacking release tarball");
        archive.unpack(install_location).unwrap();

        // Write version info into the release.
        let mut file = File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(install_location.join(XLCORE_VERSIONDATA_FILENAME))
            .unwrap();
        file.write_all(release.tag_name.as_bytes()).unwrap();
        println!("XLCM: Wrote versiondata with {}", release.tag_name);

        // Write the OpenSSL fix into the release.
        let mut file = File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(install_location.join(OPENSSL_FIX_FILENAME))
            .unwrap();
        file.write_all(include_bytes!("../../static/openssl_fix.cnf"))
            .unwrap();
        println!("XLCM: Wrote openssl_fix.cnf");
    }
    Ok(())
}
