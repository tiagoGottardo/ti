#!/usr/bin/env bash
set -euo pipefail

APP_NAME="ti"
INSTALL_DIR="${HOME}/.local/bin"
BIN_PATH="target/release/${APP_NAME}"
CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/ti"
COLOR_PATH="${CONFIG_DIR}/colors.toml"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build ${APP_NAME}" >&2
  echo "install Rust from https://rustup.rs/ and run this script again" >&2
  exit 1
fi

echo "Building ${APP_NAME}..."
cargo build --release

if [[ ! -x "${BIN_PATH}" ]]; then
  echo "error: expected binary was not produced at ${BIN_PATH}" >&2
  exit 1
fi

mkdir -p "${INSTALL_DIR}"

if [[ -e "${INSTALL_DIR}/${APP_NAME}" ]]; then
  echo "Replacing existing ${INSTALL_DIR}/${APP_NAME}..."
else
  echo "Installing to ${INSTALL_DIR}/${APP_NAME}..."
fi

cp "${BIN_PATH}" "${INSTALL_DIR}/${APP_NAME}"
chmod 755 "${INSTALL_DIR}/${APP_NAME}"

echo "Installed ${APP_NAME} to ${INSTALL_DIR}/${APP_NAME}"

mkdir -p "${CONFIG_DIR}"

if [[ -e "${COLOR_PATH}" ]]; then
  echo "Keeping existing ${COLOR_PATH}"
else
  echo "Installing default colors to ${COLOR_PATH}..."
  cat >"${COLOR_PATH}" <<'EOF'
accent = "#7f7fff"
cursor = "default"
foreground = "#ebdbb2"
background = "#282828"
selection_foreground = "#fbf1c7"
selection_background = "#504945"
current_line = "#3c3836"
current_line_number = "#fabd2f"
bar_foreground = "#ebdbb2"
bar_background = "#504945"
file_name = "#fabd2f"
whitespace = "#665c54"

color0 = "#282828"
color1 = "#cc241d"
color2 = "#98971a"
color3 = "#d79921"
color4 = "#458588"
color5 = "#b16286"
color6 = "#689d6a"
color7 = "#a89984"
color8 = "#928374"
color9 = "#fb4934"
color10 = "#b8bb26"
color11 = "#fabd2f"
color12 = "#83a598"
color13 = "#d3869b"
color14 = "#8ec07c"
color15 = "#ebdbb2"
EOF
fi

case ":${PATH:-}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "warning: ${INSTALL_DIR} is not in your PATH" >&2
    echo 'add this to your shell profile: export PATH="$HOME/.local/bin:$PATH"' >&2
    ;;
esac
