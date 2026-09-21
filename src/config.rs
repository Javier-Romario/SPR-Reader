use color_eyre::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_border_color")]
    pub border_color: String,
    #[serde(default = "default_progress_bar_color")]
    pub progress_bar_color: String,
    /// Focus letter color. When absent, inherits the border color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_color: Option<String>,
    #[serde(default = "default_show_border")]
    pub show_border: bool,
    #[serde(default = "default_show_progress_bar")]
    pub show_progress_bar: bool,
    #[serde(default = "default_enable_animations")]
    pub enable_animations: bool,
    #[serde(default = "default_inline")]
    pub inline: bool,
    #[serde(default = "default_seek_step")]
    pub seek_step: usize,
    /// Number of upcoming words to preview below the current word (0 = disabled).
    #[serde(default = "default_preview_words")]
    pub preview_words: usize,
}

fn default_border_color() -> String {
    "60,100,100".to_string() // Muted cyan RGB
}

fn default_progress_bar_color() -> String {
    "60,100,100".to_string() // Muted cyan RGB to match border
}

fn default_show_border() -> bool {
    true
}

fn default_show_progress_bar() -> bool {
    true
}

fn default_enable_animations() -> bool {
    true
}

fn default_inline() -> bool {
    true
}

fn default_seek_step() -> usize {
    10
}

fn default_preview_words() -> usize {
    0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_color: default_border_color(),
            progress_bar_color: default_progress_bar_color(),
            focus_color: None,
            show_border: default_show_border(),
            show_progress_bar: default_show_progress_bar(),
            enable_animations: default_enable_animations(),
            inline: default_inline(),
            seek_step: default_seek_step(),
            preview_words: default_preview_words(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
        Ok(PathBuf::from(home).join(".config").join("SPR-Reader"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn parse_border_color(&self) -> Color {
        Self::parse_color_string(&self.border_color)
    }

    pub fn parse_progress_bar_color(&self) -> Color {
        Self::parse_color_string(&self.progress_bar_color)
    }

    /// Returns the focus letter color. Falls back to `border_color` when unset.
    pub fn parse_focus_color(&self) -> Color {
        match &self.focus_color {
            Some(s) if !s.is_empty() => Self::parse_color_string(s),
            _ => self.parse_border_color(),
        }
    }

    fn parse_color_string(color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "darkgray" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            // Try to parse RGB format like "#ff0000" or "255,128,0"
            s if s.starts_with('#') && s.len() == 7 => {
                let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
                Color::Rgb(r, g, b)
            }
            s if s.contains(',') => {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].trim().parse().unwrap_or(0);
                    let g = parts[1].trim().parse().unwrap_or(0);
                    let b = parts[2].trim().parse().unwrap_or(0);
                    Color::Rgb(r, g, b)
                } else {
                    Color::Cyan // fallback
                }
            }
            _ => Color::Cyan, // fallback to default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    // --- defaults ---

    #[test]
    fn default_config_values() {
        let c = Config::default();
        assert_eq!(c.border_color, "60,100,100");
        assert_eq!(c.progress_bar_color, "60,100,100");
        assert!(c.focus_color.is_none());
        assert!(c.show_border);
        assert!(c.show_progress_bar);
        assert!(c.enable_animations);
        assert!(c.inline);
        assert_eq!(c.seek_step, 10);
        assert_eq!(c.preview_words, 0);
    }

    // --- parse_color_string: named ---

    #[test]
    fn parse_named_colors() {
        assert_eq!(Config::parse_color_string("cyan"), Color::Cyan);
        assert_eq!(Config::parse_color_string("red"), Color::Red);
        assert_eq!(Config::parse_color_string("green"), Color::Green);
        assert_eq!(Config::parse_color_string("blue"), Color::Blue);
        assert_eq!(Config::parse_color_string("black"), Color::Black);
        assert_eq!(Config::parse_color_string("white"), Color::White);
        assert_eq!(Config::parse_color_string("gray"), Color::Gray);
        assert_eq!(Config::parse_color_string("darkgray"), Color::DarkGray);
        assert_eq!(Config::parse_color_string("lightcyan"), Color::LightCyan);
        assert_eq!(Config::parse_color_string("lightblue"), Color::LightBlue);
    }

    #[test]
    fn parse_named_colors_case_insensitive() {
        assert_eq!(Config::parse_color_string("CYAN"), Color::Cyan);
        assert_eq!(Config::parse_color_string("Red"), Color::Red);
        assert_eq!(Config::parse_color_string("LightBlue"), Color::LightBlue);
        assert_eq!(Config::parse_color_string("DARKGRAY"), Color::DarkGray);
    }

    // --- parse_color_string: hex ---

    #[test]
    fn parse_hex_primary_colors() {
        assert_eq!(Config::parse_color_string("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(Config::parse_color_string("#00ff00"), Color::Rgb(0, 255, 0));
        assert_eq!(Config::parse_color_string("#0000ff"), Color::Rgb(0, 0, 255));
    }

    #[test]
    fn parse_hex_default_color() {
        assert_eq!(
            Config::parse_color_string("#3c6464"),
            Color::Rgb(60, 100, 100)
        );
    }

    #[test]
    fn parse_hex_black_and_white() {
        assert_eq!(Config::parse_color_string("#000000"), Color::Rgb(0, 0, 0));
        assert_eq!(
            Config::parse_color_string("#ffffff"),
            Color::Rgb(255, 255, 255)
        );
    }

    #[test]
    fn parse_hex_invalid_chars_fall_back_to_zero_components() {
        // invalid hex chars → unwrap_or(0) per component
        assert_eq!(Config::parse_color_string("#xxyyzz"), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn parse_hex_wrong_length_falls_back() {
        assert_eq!(Config::parse_color_string("#fff"), Color::Cyan); // 4 chars
        assert_eq!(Config::parse_color_string("#ffffffff"), Color::Cyan); // 9 chars
    }

    // --- parse_color_string: decimal RGB ---

    #[test]
    fn parse_rgb_decimal() {
        assert_eq!(Config::parse_color_string("255,0,0"), Color::Rgb(255, 0, 0));
        assert_eq!(
            Config::parse_color_string("60,100,100"),
            Color::Rgb(60, 100, 100)
        );
        assert_eq!(Config::parse_color_string("0,0,0"), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn parse_rgb_decimal_with_spaces() {
        assert_eq!(
            Config::parse_color_string("255, 0, 0"),
            Color::Rgb(255, 0, 0)
        );
        assert_eq!(
            Config::parse_color_string(" 60 , 100 , 100 "),
            Color::Rgb(60, 100, 100)
        );
    }

    #[test]
    fn parse_rgb_wrong_part_count_falls_back() {
        assert_eq!(Config::parse_color_string("1,2"), Color::Cyan); // 2 parts
        assert_eq!(Config::parse_color_string("1,2,3,4"), Color::Cyan); // 4 parts
    }

    // --- parse_color_string: fallback ---

    #[test]
    fn parse_unknown_string_falls_back_to_cyan() {
        assert_eq!(Config::parse_color_string("notacolor"), Color::Cyan);
        assert_eq!(Config::parse_color_string(""), Color::Cyan);
        assert_eq!(Config::parse_color_string("42"), Color::Cyan);
    }

    // --- parse_border_color / parse_progress_bar_color ---

    #[test]
    fn parse_border_color_delegates_to_parse_color_string() {
        let c = Config {
            border_color: "#ff0000".to_string(),
            ..Default::default()
        };
        assert_eq!(c.parse_border_color(), Color::Rgb(255, 0, 0));
    }

    #[test]
    fn parse_progress_bar_color_delegates() {
        let c = Config {
            progress_bar_color: "green".to_string(),
            ..Default::default()
        };
        assert_eq!(c.parse_progress_bar_color(), Color::Green);
    }

    // --- parse_focus_color fallback ---

    #[test]
    fn focus_color_absent_falls_back_to_border() {
        let c = Config {
            border_color: "red".to_string(),
            ..Default::default()
        };
        assert_eq!(c.parse_focus_color(), Color::Red);
    }

    #[test]
    fn focus_color_empty_string_falls_back_to_border() {
        let c = Config {
            border_color: "red".to_string(),
            focus_color: Some("".to_string()),
            ..Default::default()
        };
        assert_eq!(c.parse_focus_color(), Color::Red);
    }

    #[test]
    fn focus_color_explicit_overrides_border() {
        let c = Config {
            border_color: "red".to_string(),
            focus_color: Some("blue".to_string()),
            ..Default::default()
        };
        assert_eq!(c.parse_focus_color(), Color::Blue);
    }

    // --- TOML round-trip ---

    #[test]
    fn config_toml_round_trip() {
        let original = Config {
            border_color: "#ff0000".to_string(),
            progress_bar_color: "cyan".to_string(),
            focus_color: Some("lightgreen".to_string()),
            show_border: false,
            show_progress_bar: false,
            enable_animations: false,
            inline: false,
            seek_step: 5,
            preview_words: 3,
        };
        let toml_str = toml::to_string_pretty(&original).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.border_color, "#ff0000");
        assert_eq!(deserialized.focus_color, Some("lightgreen".to_string()));
        assert!(!deserialized.show_border);
        assert_eq!(deserialized.seek_step, 5);
        assert_eq!(deserialized.preview_words, 3);
    }

    #[test]
    fn config_toml_partial_uses_defaults() {
        let config: Config = toml::from_str(r#"border_color = "green""#).unwrap();
        assert_eq!(config.border_color, "green");
        assert_eq!(config.progress_bar_color, "60,100,100");
        assert!(config.show_border);
        assert_eq!(config.seek_step, 10);
        assert_eq!(config.preview_words, 0);
    }

    #[test]
    fn config_toml_empty_uses_all_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.border_color, "60,100,100");
        assert!(config.inline);
        assert!(config.enable_animations);
    }
}
