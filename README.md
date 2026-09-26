<div align="center">
<pre>
┏━┓┏━┓┏━┓   ┏━┓┏━╸┏━┓╺┳┓┏━╸┏━┓
┗━┓┣━┛┣┳┛   ┣┳┛┣╸ ┣━┫ ┃┃┣╸ ┣┳┛
┗━┛╹  ╹┗╸   ╹┗╸┗━╸╹ ╹╺┻┛┗━╸╹┗╸
</pre>
</div>

A terminal speed-reader using the Spritz ORP technique — one focused word at a time, right in your shell.

Influenced by [ratatui](https://github.com/ratatui/ratatui), [TJ DeVries](https://github.com/tjdevries), and the [Neovim](https://neovim.io) workflow ethos: read more, read faster, stay in the terminal.

<div align="center">
  <img src="demo.gif" alt="SPR demo" />
</div>

## Features

- Speed reading at configurable WPM
- Focus point highlighting (Spritz ORP)
- Inline (5-line) and fullscreen modes
- Word preview — upcoming words shown dimly below the current
- Visual progress bar with fast-forward / rewind navigation
- Customizable colors and animations via TOML config

---

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap Javier-Romario/SPR-Reader https://github.com/Javier-Romario/SPR-Reader
brew install spr
```

### Nix

Run without installing:

```bash
nix run github:Javier-Romario/SPR-Reader -- --text "your text here"
```

Install into your profile:

```bash
nix profile install github:Javier-Romario/SPR-Reader
```

Add to a NixOS / home-manager flake:

```nix
inputs.spr.url = "github:Javier-Romario/SPR-Reader";

# then in your packages:
inputs.spr.packages.${system}.default
```

### From source

```bash
cargo install --git https://github.com/Javier-Romario/SPR-Reader
```

---

## Usage

```bash
spr --text "Your text here"
spr --file path/to/file.txt
spr --file notes.txt --wpm 450 --inline
spr --file article.txt --preview-words 3

# Piped stdin
echo "The quick brown fox" | spr
cat article.txt | spr --wpm 500
pbpaste | spr --inline
curl -s https://example.com/article.txt | spr
```

### Flags

| Flag | Short | Default | Description |
|:-----|:-----:|:-------:|:------------|
| `--text <TEXT>` | `-t` | — | Text string to read |
| `--file <FILE>` | `-f` | — | Path to a text file |
| `--wpm <N>` | | `300` | Reading speed (words per minute) |
| `--inline` | `-i` | config | Compact 5-line viewport. Flag alone sets `true` |
| `--preview-words <N>` | `-p` | config | Upcoming words shown below current |

`--text` and `--file` are mutually exclusive. If neither is given, `spr` reads from stdin.

---

## Configuration

Config is created on first run at `~/.config/SPR-Reader/config.toml`.

| Key | Type | Default | Description |
|:----|:----:|:-------:|:------------|
| `border_color` | string | `"60,100,100"` | UI border color |
| `progress_bar_color` | string | `"60,100,100"` | Progress bar color |
| `focus_color` | string | *(inherits `border_color`)* | Spritz focus letter color |
| `show_border` | bool | `true` | Show/hide the UI border |
| `show_progress_bar` | bool | `true` | Show/hide the progress bar |
| `enable_animations` | bool | `true` | Enable tachyonfx transition animations |
| `inline` | bool | `true` | `true` = compact 5-line view, `false` = fullscreen |
| `seek_step` | integer | `10` | Words to jump per fast-forward / rewind |
| `preview_words` | integer | `0` | Upcoming words to preview below current (`0` = off) |

### Color formats

| Format | Example |
|:-------|:--------|
| Named | `"cyan"`, `"lightcyan"`, `"darkgray"`, … |
| Hex | `"#3c6464"` |
| Decimal RGB | `"60,100,100"` |

### Example `config.toml`

```toml
border_color       = "#3c6464"
progress_bar_color = "#3c6464"
focus_color        = "lightcyan"
show_border        = true
show_progress_bar  = true
enable_animations  = true
inline             = true
seek_step          = 10
preview_words      = 2
```

CLI flags `--inline` and `--preview-words` override config for that invocation.

---

## Neovim plugin

Speed-read text without leaving Neovim. The plugin grabs the visual selection (or buffer/line), writes it to a temp file, and runs `spr` in a floating terminal window.

Layout lives under [`nvim/`](nvim/) in this repo — point your plugin manager at it, or symlink into `~/.config/nvim`.

### lazy.nvim

```lua
{
  dir = "~/path/to/SPR-Reader/nvim",
  name = "spr.nvim",
  opts = {
    wpm = 450,
    inline = true,
    preview_words = 3,
  },
}
```

Requires `spr` on your `PATH` (or set `opts.bin = "/path/to/spr"`).

### Commands

| Command | Action |
|:--------|:-------|
| `:SprRead` | Read the visual selection (also `:'<,'>SprRead`) |
| `:SprReadBuffer` | Read the whole buffer |
| `:SprReadLine` | Read the current line |

### Default keymaps

| Key | Action |
|:----|:-------|
| `<leader>sr` (visual) | Read selection |
| `<leader>sb` | Read buffer |
| `<leader>sl` | Read line |

Disable with `vim.g.spr_no_default_maps = true`.

### Configuration

```lua
require("spr").defaults = {
  bin = "spr",
  wpm = 300,
  inline = true,
  preview_words = 0,
  window = {
    width = 42,   -- columns (>1) or fraction of editor width (<=1)
    height = 5,   -- rows (>1) or fraction of editor height (<=1)
    border = "none",
  },
}
```

---

## Keybindings

| Key | Action |
|:----|:-------|
| `Space` | Pause / Resume |
| `l` / `→` | Fast-forward (`seek_step` words) |
| `h` / `←` | Rewind (`seek_step` words) |
| `j` / `↓` | Scroll help down |
| `k` / `↑` | Scroll help up |
| `?` | Toggle help overlay |
| `q` / `Esc` | Quit |
