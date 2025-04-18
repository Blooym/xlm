#!/bin/sh -e

echo "-- XLM Steam Deck Auto-Installer --"
echo

if [ "$(id -u)" -eq 0 ]; then
    echo 'This script cannot be ran as the root user or with sudo. Please run it as a regular user.'
    exit 1
fi

echo "[Step: 1] Downloading XLM"
curl --fail -L https://github.com/Blooym/xlm/releases/latest/download/xlm-x86_64-unknown-linux-gnu > /tmp/xlm

echo "[Step: 2] Configuring XLM as a Steam Tool"
chmod +x /tmp/xlm
/tmp/xlm install-steam-tool --xlm-updater-disable --extra-launch-args="--run-as-steam-compat-tool=true --use-fallback-secret-provider" --steam-compat-path ~/.steam/root/compatibilitytools.d/

echo "[Step: 3] Cleanup XLM binary"
rm /tmp/xlm

echo
echo "-- Auto Installer Complete: Go back to gaming mode and follow the guide at https://goatcorp.github.io/faq/steamdeck or https://github.com/Blooym/xlm#readme to continue! --"