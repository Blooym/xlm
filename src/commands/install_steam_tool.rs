use crate::includes::{
    COMPATIBILITYTOOL_VDF_CONTENT, COMPATIBILITYTOOL_VDF_FILENAME, TOOLMANIFEST_VDF_CONTENT,
    TOOLMANIFEST_VDF_FILENAME, XLM_BINARY_FILENAME, XLM_COMPATDIR_DIRNAME,
    XLM_LAUNCHSCRIPT_FILENAME, get_launch_script,
};
use anyhow::{Context, Result, bail};
use clap::Parser;
use log::{debug, info};
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

/// Install the XLM Steam compatibility tool to the chosen path.
#[derive(Debug, Clone, Parser)]
pub struct InstallSteamToolCommand {
    /// Path to Steam's `compatibilitytools.d` directory.
    ///
    /// This is typically located in the following location depending on install:
    ///
    ///  - Native/Steamdeck: `~/.steam/root/compatibilitytools.d`
    ///
    ///  - Flatpak: `~/.var/app/com.valvesoftware.Steam/.steam/root/compatibilitytools.d/`
    ///
    ///  - Snap: `~/snap/steam/common/.steam/root/compatibilitytools.d/`
    #[clap(long = "steam-compat-path")]
    steam_compat_path: PathBuf,

    /// Additional flags to pass to the launch command when started from the compatibility tool.
    #[clap(long = "extra-launch-args")]
    extra_launch_args: Option<String>,

    /// Additional environment variables to pass to the launch command when started from the compatibility tool.
    #[clap(long = "extra-env-vars")]
    extra_env_vars: Option<String>,
}

impl InstallSteamToolCommand {
    pub async fn run(self) -> Result<()> {
        // Ensure the parent of "compatibilitytools.d/" exists.
        let compat_dir_parent = self
            .steam_compat_path
            .parent()
            .context("unable to obtain parent folder to compat path.")?;
        if !fs::exists(compat_dir_parent)? {
            bail!(
                "Unable to obtain information for the parent directory of `--steam-compat-path` ({compat_dir_parent:?}). This is likely because you have not ran Steam for the first time or are using the wrong type of install method."
            );
        };

        // Install compatibility tool.
        let xlm_compat_dir = self.steam_compat_path.join(XLM_COMPATDIR_DIRNAME);
        info!(
            "Installing XLM compatibility tool to {:?}\nExtra launch args: {:?}, Extra env vars: {:?}",
            xlm_compat_dir, self.extra_launch_args, self.extra_env_vars
        );
        fs::create_dir_all(&xlm_compat_dir)?;
        Self::write_compatibilitytool_vdf(&xlm_compat_dir)?;
        Self::write_toolmanifest_vdf(&xlm_compat_dir)?;
        Self::write_launch_script(&xlm_compat_dir, self.extra_launch_args, self.extra_env_vars)?;
        fs::copy(
            std::env::current_exe()?,
            xlm_compat_dir.join(XLM_BINARY_FILENAME),
        )?;
        info!(
            "Successfully set up the XLM compatibility tool - please restart Steam for it to correctly appear."
        );

        Ok(())
    }

    fn write_compatibilitytool_vdf(dir: &Path) -> Result<()> {
        debug!("Writing compatibilitytool.vdf");
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dir.join(COMPATIBILITYTOOL_VDF_FILENAME))?
            .write_all(COMPATIBILITYTOOL_VDF_CONTENT)?)
    }

    fn write_toolmanifest_vdf(dir: &Path) -> Result<()> {
        debug!("Writing toolmanifest.vdf");
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dir.join(TOOLMANIFEST_VDF_FILENAME))?
            .write_all(TOOLMANIFEST_VDF_CONTENT)?)
    }

    fn write_launch_script(
        dir: &Path,
        extra_launch_args: Option<String>,
        extra_env_vars: Option<String>,
    ) -> Result<()> {
        debug!("Writing script");
        // Write the launcher script and ensure it's executable.
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o755)
            .open(dir.join(XLM_LAUNCHSCRIPT_FILENAME))?;
        file.write_all(get_launch_script(extra_env_vars, extra_launch_args).as_bytes())?;
        Ok(())
    }
}
