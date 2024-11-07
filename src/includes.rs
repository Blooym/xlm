pub const COMPATIBILITYTOOL_VDF_FILENAME: &str = "compatibilitytool.vdf";
pub const TOOLMANIFEST_VDF_FILENAME: &str = "toolmanifest.vdf";
pub const XLM_LAUNCHSCRIPT_FILENAME: &str = "xlm.sh";
pub const XLM_BINARY_FILENAME: &str = "xlm";
pub const XLM_COMPATDIR_DIRNAME: &str = "XLM";

/// toolmanifest.vdf content as a collection of bytes.
pub const TOOLMANIFEST_VDF_CONTENT: &[u8] = include_bytes!("../static/toolmanifest.vdf");
/// compatibilitytool.vdf content as a collection of bytes.
pub const COMPATIBILITYTOOL_VDF_CONTENT: &[u8] = include_bytes!("../static/compatibilitytool.vdf");
/// aria2c tarball content as a collection of bytes.
pub const ARIA2C_TARBALL_CONTENT: &[u8] = include_bytes!("../static/aria2c-static.tar.gz");

/// Get the xlm.sh launch script as a pre-formatted string.
pub fn get_launch_script(
    extra_env_vars: &Option<String>,
    extra_launch_args: &Option<String>,
) -> String {
    format!(
        r#"#!/bin/env bash

# Prevents launching twice.
if [[ "$1" == "run" ]]; then sleep 1; exit; fi

tooldir="$(realpath "$(dirname "$0")")"

PATH=$PATH:$tooldir/xlcore {} $tooldir/xlm launch {} --install-directory $tooldir/xlcore $4
"#,
        extra_env_vars.as_deref().unwrap_or_default(),
        extra_launch_args.as_deref().unwrap_or_default()
    )
}
