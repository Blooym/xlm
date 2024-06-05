use crate::includes::{COMPATIBILITYTOOL_VDF, TOOLMANIFEST_VDF};
use clap::Parser;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

const XLCM_COMPAT_FOLDER_NAME: &'static str = "XLCM";
const XLCM_BINARY_NAME: &'static str = "xlcm";

/// Setup the XLCM steam compatibility tool.
#[derive(Debug, Clone, Parser)]
pub struct SetupCommand {
    /// The path to the 'compatibilitytools.d' folder in your steam installation directory.
    #[clap(long = "steam-compat-path")]
    steam_compat_path: PathBuf,

    /// Extra arguments to pass to the launch command when launching from the compatibility tool.
    #[clap(long = "extra-launch-args")]
    extra_launch_args: Option<String>,
}

impl SetupCommand {
    pub async fn run(self) {
        let compat_dir = self.steam_compat_path.join(XLCM_COMPAT_FOLDER_NAME);
        println!(
            "Setting up the XLCM compatibility tool inside of {:?}",
            compat_dir
        );

        fs::create_dir_all(&compat_dir).unwrap();
        Self::write_compatibilitytool_vdf(&compat_dir);
        Self::write_toolmanifest_vdf(&compat_dir);
        Self::write_script(&compat_dir, self.extra_launch_args);
        fs::copy(
            std::env::current_exe().unwrap(),
            compat_dir.join(XLCM_BINARY_NAME),
        )
        .unwrap();

        println!(
        "Successfully set up compatibility tool- please restart steam for it to correctly appear."
    );
        println!();
        println!("Note: you are now free to delete this executable as it has been safely copied to the compatibility tool folder.");
    }

    fn write_compatibilitytool_vdf(dir: &PathBuf) {
        File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("compatibilitytool.vdf"))
            .unwrap()
            .write_all(COMPATIBILITYTOOL_VDF.as_bytes())
            .unwrap();
    }

    fn write_toolmanifest_vdf(dir: &PathBuf) {
        File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("toolmanifest.vdf"))
            .unwrap()
            .write_all(TOOLMANIFEST_VDF.as_bytes())
            .unwrap();
    }

    fn write_script(dir: &PathBuf, extra_launch_args: Option<String>) {
        // Write the launcher script and ensure it's executable.
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .append(false)
            .open(dir.join("xlcm.sh"))
            .unwrap();
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        file.set_permissions(permissions).unwrap();
        file.write_all(format!(r#"#!/bin/env bash

# Prevents launching twice.
if [ $1 == "run" ]; then sleep 1; exit; fi

tooldir="$(realpath "$(dirname "$0")")"

XL_SECRET_PROVIDER=FILE PATH=$PATH:$tooldir/xlcore $tooldir/xlcm launch --install-directory $tooldir/xlcore {}
"#, extra_launch_args.unwrap_or_default()).as_bytes()).unwrap();
    }
}
