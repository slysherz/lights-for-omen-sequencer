#!/usr/bin/env bash
set -euo pipefail

LFOS_BIN_SRC="./target/release/lights-for-omen-sequencer"
LFOS_BIN_DST="/usr/local/bin/lights-for-omen-sequencer"
LFOS_SERVICE_PATH="/etc/systemd/system/lights-for-omen-sequencer.service"
LFOS_SLEEP_HOOK_PATH="/etc/systemd/system-sleep/lights-for-omen-sequencer"
LFOS_DEFAULT_COLOR_ARGS="all FFFA710F pkeys FFBF0FFA home FFBF0FFA c FF8AA8"
LFOS_COLOR_ARGS="${LFOS_COLOR_ARGS:-$LFOS_DEFAULT_COLOR_ARGS}"

echo "Setting up lights-for-omen-sequencer on Linux..."

# libusb is bundled at compile time (rusb vendored feature).
# Only a C compiler and pkg-config are needed to build.
if ! command -v cc >/dev/null 2>&1; then
  echo "WARNING: No C compiler found. Install gcc or clang before running cargo build."
fi

echo "Installing udev rule..."
sudo cp 50-omen-keyboard.rules /etc/udev/rules.d/50-omen-keyboard.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

if getent group plugdev >/dev/null 2>&1; then
  if id -nG "$USER" | grep -qw plugdev; then
    echo "User '$USER' is already in plugdev group."
  else
    echo "Adding user '$USER' to plugdev group..."
    sudo usermod -aG plugdev "$USER"
    echo "Log out and log back in for group changes to take effect."
  fi
else
  echo "Group 'plugdev' was not found on this system."
  echo "Create it (or adjust 50-omen-keyboard.rules to an existing group), then re-run this script."
fi

if [ ! -x "$LFOS_BIN_SRC" ]; then
  if command -v cargo >/dev/null 2>&1; then
    echo "Release binary not found, building it with cargo..."
    cargo build --release
  else
    echo "ERROR: release binary not found at $LFOS_BIN_SRC and cargo is not installed."
    echo "Install Rust/cargo and run: cargo build --release"
    exit 1
  fi
fi

echo "Installing binary to $LFOS_BIN_DST..."
sudo install -Dm755 "$LFOS_BIN_SRC" "$LFOS_BIN_DST"

echo "Preparing systemd paths..."
sudo install -d /etc/systemd/system
sudo install -d /etc/systemd/system-sleep

echo "Installing startup service..."
sudo tee "$LFOS_SERVICE_PATH" >/dev/null <<EOF
[Unit]
Description=Apply OMEN Sequencer keyboard colors
After=multi-user.target
StartLimitIntervalSec=20
StartLimitBurst=5

[Service]
Type=oneshot
ExecStart=$LFOS_BIN_DST $LFOS_COLOR_ARGS
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
EOF

echo "Installing resume hook..."
sudo tee "$LFOS_SLEEP_HOOK_PATH" >/dev/null <<EOF
#!/bin/sh
case "\$1/\$2" in
  post/*)
    systemctl restart --no-block lights-for-omen-sequencer.service
    ;;
esac
EOF
sudo chmod +x "$LFOS_SLEEP_HOOK_PATH"

echo "Enabling service at boot..."
sudo systemctl daemon-reload
sudo systemctl enable lights-for-omen-sequencer.service
if ! sudo systemctl restart lights-for-omen-sequencer.service; then
  echo "WARNING: service failed to start right now (keyboard might be unavailable)."
  echo "Check status with: sudo systemctl status lights-for-omen-sequencer.service"
fi

echo "Done."
echo "Colors command: $LFOS_BIN_DST $LFOS_COLOR_ARGS"
echo "Override colors by setting LFOS_COLOR_ARGS, for example:"
echo "LFOS_COLOR_ARGS='all ff0000 pkeys 00ff00' ./install-linux.sh"
