use crate::includes::{COMPATIBILITYTOOL_VDF, TOOLMANIFEST_VDF, XLCM_SHELL_SCRIPT};
use clap::Parser;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

const XLCM_COMPAT_FOLDER_NAME: &'static str = "XLCM";
const XLCM_BINARY_NAME: &'static str = "xlcm";

#[derive(Debug, Clone, Parser)]
pub struct SetupCommand {
    /// The path to the 'compatibilitytools.d' folder in your steam installation directory.
    #[clap(long = "steam-compat-path")]
    steam_compat_path: PathBuf,
}

impl SetupCommand {
    pub async fn run(self) {
        let compat_dir = self.steam_compat_path.join(XLCM_COMPAT_FOLDER_NAME);
        println!(
            "Setting up the XLCM compatibility tool inside of {:?}",
            compat_dir
        );

        fs::create_dir_all(&compat_dir).unwrap();

        // Write manifest files.
        File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(compat_dir.join("compatibilitytool.vdf"))
            .unwrap()
            .write_all(COMPATIBILITYTOOL_VDF.as_bytes())
            .unwrap();
        File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(compat_dir.join("toolmanifest.vdf"))
            .unwrap()
            .write_all(TOOLMANIFEST_VDF.as_bytes())
            .unwrap();

        // Write the launcher script and ensure it's executable.
        let mut file = File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(compat_dir.join("xlcm.sh"))
            .unwrap();
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o755);
        file.set_permissions(permissions).unwrap();
        file.write_all(XLCM_SHELL_SCRIPT.as_bytes()).unwrap();

        // Write a copy of the current executable into the directory.
        fs::copy(
            std::env::current_exe().unwrap(),
            compat_dir.join(XLCM_BINARY_NAME),
        )
        .unwrap();

        println!();
        println!(
            "Successfully set up XLCM. Please ensure you restart steam for it to correctly appear."
        );
        println!("Note: you are now free to delete this executable from the current directory as it has been safely copied to the compatibility tool folder.");
    }
}
