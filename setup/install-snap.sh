#!/bin/bash

if [[ $EUID -eq 0 ]]; then
    echo 'Error: this script cannot be ran as the root user or with sudo. Please run it as a regular user.'
    exit 1
fi

echo "-- XLM Snap Auto-Installer --"
echo ""

echo "[Step: 1] Downloading XLM"
curl --fail -L https://github.com/Blooym/xlm/releases/latest/download/xlm-x86_64-unknown-linux-gnu > /tmp/xlm

if [[ $? -ne 0 ]]; then
    echo "Error: XLM did not appear to download correctly, this may be an issue with your network."
    exit 1
fi

echo "[Step: 2] Configuring XLM as a Steam Tool"
chmod +x /tmp/xlm
/tmp/xlm install-steam-tool --xlm-updater-disable --steam-compat-path ~/snap/steam/common/.steam/root/compatibilitytools.d/

echo "[Step: 3] Cleanup XLM binary"
rm /tmp/xlm

echo ""
echo "-- Auto Installer Complete: Restart Steam and follow the README to continue! --"