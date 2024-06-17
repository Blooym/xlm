use crate::includes::OPENSSL_FIX_CNF;
use bytes::Buf;
use clap::Parser;
use eframe::egui::{self, Layout};
use flate2::read::GzDecoder;
use octocrab::models::repos::Release;
use reqwest::Url;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::PathBuf,
    sync::RwLock,
};
use tar::Archive;
use tokio::process::Command;
use winit::platform::wayland::EventLoopBuilderExtWayland;

const OPENSSL_FIX_FILENAME: &'static str = "openssl_fix.cnf";
const XLCORE_VERSIONDATA_FILENAME: &'static str = "versiondata";

/// Whether all egui windows should close when they next redraw.
static UI_SHOULD_CLOSE: RwLock<bool> = RwLock::new(false);

/// Install/Update XIVLauncher and launch it.
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

    #[clap(
        default_value = "https://github.com/rankynbass/aria2-static-build/releases/download/v1.37.0-2/aria2-static.tar.gz",
        long = "aria-download-url"
    )]
    aria_download_url: Url,

    /// The location where the XIVLauncher should be installed.
    #[clap(default_value = dirs::data_local_dir().unwrap().join("xlcore").into_os_string(), long = "install-directory")]
    install_directory: PathBuf,

    /// Do not check to see if XIVLauncher is out of date on startup. This will not prevent XIVLauncher from installing if it's not present at all.
    #[clap(default_value_t = false, long = "skip-update")]
    skip_update: bool,
}

impl LaunchCommand {
    pub async fn run(self) {
        {
            // Query the GitHub API for release information.
            let octocrab = octocrab::instance();
            let repo = octocrab.repos(&self.xlcore_repo_owner, &self.xlcore_repo_name);
            let release = match repo.releases().get_latest().await {
                Ok(release) => release,
                Err(err) => {
                    eprintln!(
                        "XLCM: Failed to obtain release information for {}/{}: {:?}",
                        self.xlcore_repo_owner,
                        self.xlcore_repo_name,
                        err.source()
                    );
                    return;
                }
            };

            // Install XIVLauncher or do an update check if version data already exists.
            match fs::read_to_string(self.install_directory.join(XLCORE_VERSIONDATA_FILENAME)) {
                Ok(ver) => {
                    if !self.skip_update {
                        if ver == release.tag_name {
                            println!("XLCM: Installed XIVLauncher is up to date!");
                        } else {
                            Self::open_xlcm_wait_ui();
                            println!(
                                "XLCM: Installed XIVLauncher is out of date, starting updater..."
                            );
                            Self::install_or_update_xlcore(
                                release,
                                &self.xlcore_release_asset,
                                self.aria_download_url,
                                &self.install_directory,
                            )
                            .await
                            .unwrap();
                            println!(
                                "XLCM: Successfully updated XIVLauncher to the latest version."
                            )
                        }
                    } else {
                        println!("XLCM: Skip update enabled, not attempting to update XIVLauncher.")
                    }
                }
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        Self::open_xlcm_wait_ui();
                        println!("XLCM: Unable to obtain local version data for XIVLauncher, installing from latest tag...");
                        Self::install_or_update_xlcore(
                            release,
                            &self.xlcore_release_asset,
                            self.aria_download_url,
                            &self.install_directory,
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
        }

        Self::close_xlcm_wait_ui();

        println!("XLCM: Starting XIVLauncher");
        let cmd = Command::new(self.install_directory.join("XIVLauncher.Core"))
            .env("XL_PRELOAD", env::var("LD_PRELOAD").unwrap_or_default()) // Write XL_PRELOAD so it can maybe be passed to the game later.
            .env("XL_SCT", "1") // Needed to trigger compatibility tool mode in XIVLauncher. Otherwise XL_PRELOAD will be ignored.
            .env(
                "OPENSSL_CONF",
                &self.install_directory.join("openssl_fix.cnf"),
            ) // Apply the OpenSSL fix for downloads.
            .env_remove("LD_PRELOAD") // Completely remove LD_PRELOAD otherwise steam overlay will break the launcher text.
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

    /// Creates a new XLCore installation or overwrites an existing XLCore installion with a new one.
    async fn install_or_update_xlcore(
        release: Release,
        release_asset_name: &String,
        aria_download_url: Url,
        install_location: &PathBuf,
    ) -> Result<(), ()> {
        for asset in release.assets {
            if asset.name != *release_asset_name {
                continue;
            }

            // Download and decompress XLCore.
            {
                println!(
                    "XLCM: Downloading release from {}",
                    asset.browser_download_url,
                );
                let response = reqwest::get(asset.browser_download_url).await.unwrap();
                let bytes = response.bytes().await.unwrap();
                let mut archive = Archive::new(GzDecoder::new(bytes.reader()));
                let _ = fs::remove_dir_all(install_location);
                fs::create_dir_all(install_location).unwrap();
                println!("XLCM: Unpacking release tarball");
                archive.unpack(install_location).unwrap();
                println!("XLCM: Wrote xivlauncher files");
            }

            {
                // Download and write aria2c
                let response = reqwest::get(aria_download_url).await.unwrap();
                let bytes = response.bytes().await.unwrap();
                let mut archive = Archive::new(GzDecoder::new(bytes.reader()));
                println!("XLCM: Unpacking aria2c tarball");
                archive.unpack(install_location).unwrap();
                println!("XLCM: Wrote aria2c binary");
            }

            {
                // Write the OpenSSL fix into the release.
                let mut file = File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .append(false)
                    .open(install_location.join(OPENSSL_FIX_FILENAME))
                    .unwrap();
                file.write_all(OPENSSL_FIX_CNF.as_bytes()).unwrap();
                println!("XLCM: Wrote openssl_fix.cnf");

                // Write version info into the release.
                let mut file = File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .append(false)
                    .open(install_location.join(XLCORE_VERSIONDATA_FILENAME))
                    .unwrap();
                file.write_all(release.tag_name.as_bytes()).unwrap();
                println!("XLCM: Wrote versiondata with {}", release.tag_name);
            }

            break;
        }

        Ok(())
    }

    /// Starts a new Tokio task and displays an egui window displaying a "XIVLauncher is starting" message.
    /// The egui window blocks inside of the task meaning it cannot be killed by aborting the thread.
    /// To close the window you can call [`Self::close_xlcm_wait_ui`] which will close all existing windows.
    fn open_xlcm_wait_ui() {
        *UI_SHOULD_CLOSE.write().unwrap() = false;
        tokio::task::spawn(async move {
            eframe::run_simple_native(
                "XLCM",
                eframe::NativeOptions {
                    event_loop_builder: Some(Box::new(|event_loop_builder| {
                        event_loop_builder.with_any_thread(true);
                    })),
                    viewport: egui::ViewportBuilder::default()
                        .with_inner_size([800.0, 500.0])
                        .with_resizable(false)
                        .with_decorations(false),
                    default_theme: eframe::Theme::Dark,
                    ..Default::default()
                },
                move |ctx, _frame| {
                    if *UI_SHOULD_CLOSE.read().unwrap() {
                        std::process::exit(0);
                    }

                    ctx.set_pixels_per_point(1.5);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.with_layout(
                            Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.heading("Preparing XIVLauncher, please wait patiently.");
                            },
                        );
                    });
                },
            )
            .unwrap();
        });
    }

    // Closes any running egui windows regardless of the thread they're running on.
    fn close_xlcm_wait_ui() {
        *UI_SHOULD_CLOSE.write().unwrap() = true;
    }
}
