use crate::includes::{
    get_launch_script, COMPATIBILITYTOOL_VDF_CONTENT, COMPATIBILITYTOOL_VDF_FILENAME,
    TOOLMANIFEST_VDF_CONTENT, TOOLMANIFEST_VDF_FILENAME, XLM_BINARY_FILENAME,
    XLM_COMPATDIR_DIRNAME, XLM_LAUNCHSCRIPT_FILENAME,
};
use anyhow::Result;
use clap::Parser;
use log::{debug, info};
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// Install the XLM steam compatibility tool for easier launching via Steam.
#[derive(Debug, Clone, Parser)]
pub struct InstallSteamToolCommand {
    /// The path to the 'compatibilitytools.d' folder in your steam installation directory.
    /// Please read the manual if you don't know where this is.
    #[clap(long = "steam-compat-path")]
    steam_compat_path: PathBuf,

    /// Extra arguments to pass to the launch command when launching from the compatibility tool.
    /// This can usually be left blank.
    #[clap(long = "extra-launch-args")]
    extra_launch_args: Option<String>,

    /// Extra environment variables to pass to the launch command & XIVLauncher when launching from the compatibility tool.
    /// This can usually be left blank.
    #[clap(long = "extra-env-vars")]
    extra_env_vars: Option<String>,
}

impl InstallSteamToolCommand {
    pub async fn run(self) -> Result<()> {
        let compat_dir = self.steam_compat_path.join(XLM_COMPATDIR_DIRNAME);

        // Write files
        info!(
            "Setting up the XLM compatibility tool inside of {:?}",
            compat_dir
        );
        info!(
            "Extra launch args: {:?}, Extra env vars: {:?}",
            self.extra_launch_args, self.extra_env_vars
        );
        fs::create_dir_all(&compat_dir)?;
        Self::write_compatibilitytool_vdf(&compat_dir)?;
        Self::write_toolmanifest_vdf(&compat_dir)?;
        Self::write_script(&compat_dir, self.extra_launch_args, self.extra_env_vars)?;
        fs::copy(
            std::env::current_exe()?,
            compat_dir.join(XLM_BINARY_FILENAME),
        )?;

        info!("Successfully set up compatibility tool please restart steam for it to correctly appear.");

        Ok(())
    }

    fn write_compatibilitytool_vdf(dir: &Path) -> Result<()> {
        debug!("Writing compatibilitytool.vdf");
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join(COMPATIBILITYTOOL_VDF_FILENAME))?
            .write_all(COMPATIBILITYTOOL_VDF_CONTENT)?)
    }

    fn write_toolmanifest_vdf(dir: &Path) -> Result<()> {
        debug!("Writing toolmanifest.vdf");
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join(TOOLMANIFEST_VDF_FILENAME))?
            .write_all(TOOLMANIFEST_VDF_CONTENT)?)
    }

    fn write_script(
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
            .append(false)
            .open(dir.join(XLM_LAUNCHSCRIPT_FILENAME))?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o755);
        file.set_permissions(permissions)?;
        file.write_all(get_launch_script(&extra_env_vars, &extra_launch_args).as_bytes())?;
        Ok(())
    }
}
