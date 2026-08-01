# Ti

Ti is a small terminal text editor inspired by vi. It is a toy project, but it
already supports the core editing loop: normal mode, insert mode, visual
selection, movement, saving, quitting, undo, and basic yank/delete/paste
operations.

(Demo)[./assets/demo.gif]

## Requirements

- Rust toolchain with Cargo.
- A terminal that supports ANSI escape sequences.

Install Rust with [rustup](https://rustup.rs/) if you do not already have
`cargo`.

## Install

Clone the repository and run the installer:

```sh
git clone https://github.com/tiagoGottardo/ti.git
cd ti
./install.sh
```

The script builds a release binary and installs it to:

```sh
~/.local/bin/ti
```

It also creates the default color configuration when it does not already exist:

```sh
~/.config/ti/colors.toml
```

Make sure `~/.local/bin` is in your `PATH`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

For a permanent setup, add that line to your shell profile.

### Nix

You can also install and run it using nix

```bash
nix run github:tiagoGottardo/ti#ti -- <file-to-edit> # running without installing permanently
# or
nix profile add github:tiagoGottardo/ti#ti # install into the nix profile (permanent install)
ti <file-to-edit> # to run
```

## Colors

Ti loads colors from:

```sh
~/.config/ti/colors.toml
```

If `XDG_CONFIG_HOME` is set, Ti uses:

```sh
$XDG_CONFIG_HOME/ti/colors.toml
```

Create or refresh the default color file by running the installer:

```sh
./install.sh
```

Edit the color file with Ti:

```sh
ti ~/.config/ti/colors.toml
```

Reset the color file to the latest defaults:

```sh
rm ~/.config/ti/colors.toml
./install.sh
```

Supported values are `"#RRGGBB"`, `"default"`, and `"NONE"`. Supported keys are
`accent`, `cursor`, `foreground`, `background`, `selection_foreground`,
`selection_background`, `current_line`, `current_line_number`,
`bar_foreground`, `bar_background`, `file_name`, `whitespace`, and `color0`
through `color15`.

## Build

```sh
cargo build --release
```

Run the built binary directly:

```sh
./target/release/ti path/to/file.txt
```

## Usage

Open a file:

```sh
ti path/to/file.txt
```

Ti starts in normal mode.

Common keys:

| Key | Action |
| --- | --- |
| `h` / `j` / `k` / `l` | Move left, down, up, right |
| Number + `h` / `j` / `k` / `l` | Move multiple times, like `5j` |
| Arrow keys | Move vertically or right |
| `w` / `b` / `e` | Move by word |
| `gg` | Go to first line |
| `G` | Go to end of document |
| `i` | Insert before cursor |
| `I` | Insert at start of line |
| `a` | Insert after cursor |
| `A` | Insert at end of line |
| `o` | Open a new line below |
| `Esc` | Return to normal mode |
| `v` | Start visual selection |
| `x` | Delete character |
| `d` | Start delete operation |
| `dd` | Delete current line |
| `D` | Delete to end of line |
| `y` | Start yank operation |
| `yy` | Yank current line |
| `Y` | Yank to end of line |
| `p` / `P` | Paste after or before cursor |
| `J` | Join lines |
| `r` | Replace one character |
| `u` | Undo |
| `W` | Save |
| `Q` | Quit |

## Development

Run checks and tests:

```sh
cargo check
cargo test
```

Format the code:

```sh
cargo fmt --all
```

## Next steps

- Handle terminal window resize
- Stream large files instead of reading the whole document at once
- Add LSP support
- Integrate Lua
