#!/usr/bin/env bash
# Explicit opt-in installation. Run this in your Wayland user session.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
source_file="$here/deploy/systemd/blobin-render-bridge.service"
target="$HOME/.config/systemd/user/blobin-render-bridge.service"
if [[ -e "$target" ]] && ! cmp -s "$source_file" "$target"; then
  echo "Different unit already present: $target; will not overwrite" >&2
  exit 2
fi
mkdir -p "$(dirname "$target")"
install -m 0644 "$source_file" "$target"
systemctl --user daemon-reload
systemctl --user enable --now blobin-render-bridge.service
systemctl --user --no-pager status blobin-render-bridge.service
printf '\nFollow logs: journalctl --user -u blobin-render-bridge.service -f\n'
