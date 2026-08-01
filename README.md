# Ti

Ti is a small terminal text editor inspired by vi. It is a toy project, but it
already supports the core editing loop: normal mode, insert mode, visual
selection, movement, saving, quitting, undo, and basic yank/delete/paste
operations.

## Status

This project is experimental. It is useful for learning and hacking on a
minimal editor, but it is not intended to replace a production editor yet.

Current limitations:

- Linux/Unix-like terminal support is the practical target.
- Files are read into memory instead of streamed.
- Window resize handling is not complete.
- Viewport scrolling is still incomplete.

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

- [X] Refactor editor internals
- [X] Implement viewport rendering
- [X] Implement visual mode
- [X] Implement line joining in normal and visual modes
- [X] Support copy, delete, and paste
- [X] Support Unicode input
- [X] Implement number column
- [X] Solve bug: cant delete the last empty line
- [X] Solve bug: It doesn't show selection on empty lines
- [X] Solve bug: It can't select more than one viewport size content
- [X] Support Alt + key
- [X] Implement content movement with Alt + j/k
- [X] Implement colors with themes
- [ ] Improve UI (number column, top and bottom bars)
- [ ] Implement number commands

- [ ] Handle terminal window resize
- [ ] Stream large files instead of reading the whole document at once
