# XLM - XIVLauncher Manager

> [!IMPORTANT]  
> The code in this repository is considered experimental and is not finished. While I personally use this tool daily without issue, you may run into problems that I have not.

An alternative method for launching XIVLauncher.Core on Linux, primarily built to avoid the pitfalls of using Flatpak XIVLauncher & Steam together. It allows for launching standalone or via a steam compatibility tool while providing nice features like launcher auto-updates.

## Setup

The download and setup process has yet to be streamlined and as such may not be a smooth experience. The steps listed in this guide should get you where you need to go though.

### Getting the binary

To use this tool it's recommended to download the latest release from the [GitHub Releases](https://github.com/Blooym/xlm/releases/latest) page. You can also compile it yourself using Rust if you so wish, however no guide is offered for that.

Please note that at this time the binary does not auto-update from GitHub releases and as such will need to be updated by hand if you wish to take advantage of new features or receive critical bugfixes. There are future plans to implement an auto-updater so this is not necessary.

## Setting up as a steam compatibility tool

Once you have the XLM binary installed make sure it's set as executable. Open up a terminal and navigate to the directory where you installed the binary and run `chmod +x ./xlm`.

Afterwards, run one of the following commands to install XLM as a Steam compatibility tool. What command you need to run depends on how you have Steam installed.

**Steamdeck**:
```
./xlm install-steam-tool --extra-launch-args="--use-fallback-secret-provider" --steam-compat-path ~/.steam/root/compatibilitytools.d/
```

**Steam (Native)**
```
./xlm install-steam-tool --steam-compat-path ~/.steam/root/compatibilitytools.d/
```

**Steam (Flatpak)**
```
./xlm install-steam-tool --extra-launch-args="--use-fallback-secret-provider" --steam-compat-path ~/.var/app/com.valvesoftware.Steam/.steam/root/compatibilitytools.d/
```

After you've ran this do the following:
- Switch back to gaming mode (Steamdeck) or restart Steam 
- Navigate to your library and select "FINAL FANTASY XIV Online" 
- Open the game properties menu and switch to the "compatibility" tab
- Enable the "Force the use of a specific Steam Play compatibility tool" checkbox
- From the dropdown that appears select "XLCore [XLM]" (if this does not show, please make sure you restarted Steam first)
- You can now launch the game. XIVLauncher will be automatically installed to the compatibilitytools.d directory and start as usual. When you close the game, Steam will recognise this.

### Passing extra arguments or environment variables to the XIVLauncher (Advanced & Optional)

When installing the compatibility tool you have the option to pass extra launch arguments via the `--extra-launch-args` flag and to pass extra environment variables via the `--extra-env-vars` flag. This will allow you to, for example, override the version of XIVLauncher you're using. More information on launch flags can be found by running `xlm launch --help`.

### "No secrets provider installed or configured"

This means that XIVLauncher was unable to find a secure way to store your passwords. This is usually because you don't have a secrets manager like GNOME Keyring or KDE Wallet installed on your system. It's recommended you install a recognised and well known secrets manager to solve this problem.

If you still run into this issue even with a secrets manager installed on your system, use the fallback file storage provider offered by XIVLauncher; You can tell XLM to ask XIVLauncher to enable this by running the `install-steam-tool` command again and including the following flag: `--extra-launch-args="--use-fallback-secret-provider"`.