# SPR

Concentration has never been my strong point, that being said; programming requires concentration and reading, 
and I am a programmer, so why not create tooling which aides me in my work and makes my life easier.

That is how SPR came about, influenced by [Stutter](), [Ratatui and Orhun's amazing work]() 
along with the likes of [TJ De'vries]() and [Neovim]() workflows.
The idea is that we need to read **a lot**, why not make it easier to do so on the command line.

<div align="center">
  <img src="demo.gif" alt="SPR demo" />
</div>

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap Javier-Romario/SPR-Reader https://github.com/Javier-Romario/SPR-Reader
brew install spr
```

### Nix (NixOS / nix-darwin / any Linux with Nix)

Run without installing:

```bash
nix run github:Javier-Romario/SPR-Reader -- --text "your text here"
```

Install into your profile:

```bash
nix profile install github:Javier-Romario/SPR-Reader
```

Or add to your NixOS / home-manager flake:

```nix
inputs.spr.url = "github:Javier-Romario/SPR-Reader";

# then in your packages:
inputs.spr.packages.${system}.default
```

### From source (requires Rust)

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
```

### CLI Flags

| Flag | Short | Default | Description |
|---|---|---|---|
| `--text <TEXT>` | `-t` | — | Text string to read |
| `--file <FILE>` | `-f` | — | Path to a text file to read |
| `--wpm <N>` | | `300` | Reading speed in words per minute |
| `--inline [bool]` | `-i` | config value | Inline mode (5-line viewport). Omit value for `true` |
| `--preview-words <N>` | `-p` | config value | Upcoming words shown dimly below current word |

`--text` and `--file` are mutually exclusive. One must be provided.

---

## Configuration

SPR creates a config file on first run at `~/.config/SPR-Reader/config.toml`.

### Config File Options

| Key | Type | Default | Description |
|---|---|---|---|
| `border_color` | string | `"60,100,100"` | Color of the UI border |
| `progress_bar_color` | string | `"60,100,100"` | Color of the progress bar |
| `focus_color` | string | *(inherits `border_color`)* | Color of the Spritz focus letter. Omit to match the border |
| `show_border` | bool | `true` | Show or hide the UI border |
| `show_progress_bar` | bool | `true` | Show or hide the progress bar |
| `enable_animations` | bool | `true` | Enable tachyonfx transition animations |
| `inline` | bool | `true` | `true` = compact 5-line viewport, `false` = fullscreen |
| `seek_step` | integer | `10` | Number of words to jump when rewinding or fast-forwarding |
| `preview_words` | integer | `0` | Upcoming words to display dimly below the current word |

### Color Formats

All color fields accept any of the following formats:

| Format | Example | Notes |
|---|---|---|
| Named color | `"cyan"` | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `darkgray`, `white`, and their `light*` variants |
| Hex RGB | `"#3c6464"` | Standard 6-digit hex |
| Decimal RGB | `"60,100,100"` | Comma-separated R,G,B values (0–255) |

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

CLI flags `--inline` and `--preview-words` override the config file values for that invocation.

---

## Keybindings

| Key | Action |
|---|---|
| `Space` | Pause / Resume |
| `l` / `→` | Fast-forward (`seek_step` words) |
| `h` / `←` | Rewind (`seek_step` words) |
| `j` / `↓` | Scroll help down |
| `k` / `↑` | Scroll help up |
| `?` | Toggle help overlay |
| `q` / `Esc` | Quit |

---

**Key Features:**

- 📖 Speed reading with configurable WPM
- 🎯 Focus point highlighting (Spritz algorithm)
- 🖥️ Dual modes: fullscreen and inline (5-line viewport)
- 📊 Visual progress tracking
- ⏸️ Interactive pause/resume controls
- 🎨 Customizable colors and animations
- ⚙️ TOML-based configuration system
