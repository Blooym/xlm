use crate::includes::{COMPATIBILITYTOOL_VDF, TOOLMANIFEST_VDF};
use anyhow::Result;
use clap::Parser;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

const XLM_COMPAT_FOLDER_NAME: &'static str = "XLM";
const XLM_BINARY_NAME: &'static str = "xlm";

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
}

impl InstallSteamToolCommand {
    pub async fn run(self) -> Result<()> {
        let compat_dir = self.steam_compat_path.join(XLM_COMPAT_FOLDER_NAME);
        println!(
            "Setting up the XLM compatibility tool inside of {:?}",
            compat_dir
        );

        fs::create_dir_all(&compat_dir)?;
        Self::write_compatibilitytool_vdf(&compat_dir)?;
        Self::write_toolmanifest_vdf(&compat_dir)?;
        Self::write_script(&compat_dir, self.extra_launch_args)?;
        fs::copy(std::env::current_exe()?, compat_dir.join(XLM_BINARY_NAME))?;

        println!(
        "Successfully set up compatibility tool- please restart steam for it to correctly appear."
    );
        println!();
        println!("Note: you are now free to delete this executable as it has been safely copied to the compatibility tool folder.");

        Ok(())
    }

    fn write_compatibilitytool_vdf(dir: &PathBuf) -> Result<()> {
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("compatibilitytool.vdf"))?
            .write_all(COMPATIBILITYTOOL_VDF.as_bytes())?)
    }

    fn write_toolmanifest_vdf(dir: &PathBuf) -> Result<()> {
        Ok(File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("toolmanifest.vdf"))?
            .write_all(TOOLMANIFEST_VDF.as_bytes())?)
    }

    fn write_script(dir: &PathBuf, extra_launch_args: Option<String>) -> Result<()> {
        // Write the launcher script and ensure it's executable.
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("xlm.sh"))?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o755);
        file.set_permissions(permissions)?;
        file.write_all(
            format!(
                r#"#!/bin/env bash

# Prevents launching twice.
if [ $1 == "run" ]; then sleep 1; exit; fi

tooldir="$(realpath "$(dirname "$0")")"

PATH=$PATH:$tooldir/xlcore $tooldir/xlm launch --use-fallback-secret-provider --install-directory $tooldir/xlcore {}
"#,
                extra_launch_args.unwrap_or_default()
            )
            .as_bytes(),
        )?;

        Ok(())
    }
}
