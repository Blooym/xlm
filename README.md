# XLM - XIVLauncher Manager

> [!IMPORTANT]  
> While I personally use this tool daily without issue, you may run into problems that I have not. Please report any issues you experience on the issues tab.

An alternative method for launching XIVLauncher.Core on Linux, primarily built to avoid the pitfalls of using Flatpak XIVLauncher & Steam together. It allows for launching standalone or via a steam compatibility tool while providing nice features like launcher auto-updates (and steam overlay support!)

## Setup

Auto installers for the steam compatibility tool part of XLM are provided for the `Steam Deck`, `Flatpak` and `Native` versions of Steam. For any other use-case you will need to manually download the XLM binary from the [GitHub Releases Page](https://github.com/Blooym/xlm/releases/latest).

### Auto Installers

Run one of the following commands to install XLM as a Steam compatibility tool. What command you need to run depends on how you have Steam installed.

**Steamdeck**:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Blooym/xlm/main/setup/install-steamdeck.sh)"
```

**Steam (Native)**
```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Blooym/xlm/main/setup/install-native.sh)"
```

**Steam (Snap)**
```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Blooym/xlm/main/setup/install-snap.sh)"
```

**Steam (Flatpak)**
```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/Blooym/xlm/main/setup/install-flatpak.sh)"
```

After the auto-installer has finished running, follow these steps to use it in Steam:
- Switch back to gaming mode (Steam Deck) or restart Steam.
- Navigate to your library and select "FINAL FANTASY XIV Online" or "FINAL FANTASY XIV Online Free Trial" (trial and non-steam users) 
- Open the game properties menu and switch to the "compatibility" tab.
- Enable the "Force the use of a specific Steam Play compatibility tool" checkbox.
- From the dropdown that appears select "XLCore [XLM]" (if this does not show, please make sure you restarted Steam first).
- You can now launch the game. XIVLauncher will be automatically installed to the compatibilitytools.d directory and start as usual. When you close the game, Steam will recognise this.

### Passing extra arguments or environment variables on startup (Advanced & Optional)

> [!NOTE]  
> This is not available when using an auto-installer. Please manually run the XLM binary to pass extra options.

When installing the compatibility tool you have the option to pass extra launch arguments via the `--extra-launch-args` flag and to pass extra environment variables via the `--extra-env-vars` flag. This will allow you to, for example, override the version of XIVLauncher you're using. More information on launch flags can be found by running `xlm launch --help` or [viewing the code (advanced)](https://github.com/Blooym/xlm/blob/01150849865eb436bbb9477be17c40752488f463/src/commands/launch.rs#L29).

#### Using a fork of XIVLauncher

To use a fork of XIVLauncher you can add the flags `--xlcore-repo-owner` and `--xlcore-repo-name` to the `--extra-launch-args` when installing the steam compatibility tool to fetch the XIVLauncher release binary from a different source on GitHub; You can also manually edit the `xlm.sh` script in the Steam compatibility tools folder if you know how to do this. Forks of XIVLauncher can also offer their own install scripts for XLM that automate this process for you so you don't have to manually tinker, do check to see if one exists for the fork you want to use!

### "No secrets provider installed or configured"

This means that XIVLauncher was unable to find a secure way to store your passwords. This is usually because you don't have a secrets manager like GNOME Keyring or KDE Wallet installed on your system. It's recommended you install a recognised and well known secrets manager to solve this problem.

If you still run into this issue even with a secrets manager installed on your system, use the fallback file storage provider offered by XIVLauncher; You can tell XLM to ask XIVLauncher to enable this by running the `install-steam-tool` command again and including the following flag: `--extra-launch-args="--use-fallback-secret-provider"`.
