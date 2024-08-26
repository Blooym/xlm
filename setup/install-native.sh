#!/bin/bash

echo "-- XLM Native Auto-Installer --"

echo "[Step: 1] Downloading XLM"
wget -q --show-progress -P /tmp https://github.com/Blooym/xlm/releases/latest/download/xlm

echo "[Step: 2] Configuring XLM as a Steam Tool"
chmod +x /tmp/xlm
/tmp/xlm install-steam-tool --steam-compat-path ~/.steam/root/compatibilitytools.d/

echo "[Step: 3] Cleanup XLM binary"
rm /tmp/xlm