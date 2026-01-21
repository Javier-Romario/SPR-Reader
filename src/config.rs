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
    #[serde(default = "default_show_border")]
    pub show_border: bool,
    #[serde(default = "default_show_progress_bar")]
    pub show_progress_bar: bool,
    #[serde(default = "default_enable_animations")]
    pub enable_animations: bool,
    #[serde(default = "default_inline")]
    pub inline: bool,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            border_color: default_border_color(),
            progress_bar_color: default_progress_bar_color(),
            show_border: default_show_border(),
            show_progress_bar: default_show_progress_bar(),
            enable_animations: default_enable_animations(),
            inline: default_inline(),
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

    pub fn first_use_marker_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(".first_use_complete"))
    }

    pub fn is_first_use() -> Result<bool> {
        let marker_path = Self::first_use_marker_path()?;
        Ok(!marker_path.exists())
    }

    pub fn mark_first_use_complete() -> Result<()> {
        let marker_path = Self::first_use_marker_path()?;

        // Ensure directory exists
        if let Some(parent) = marker_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&marker_path, "")?;
        Ok(())
    }

    pub fn parse_border_color(&self) -> Color {
        Self::parse_color_string(&self.border_color)
    }

    pub fn parse_progress_bar_color(&self) -> Color {
        Self::parse_color_string(&self.progress_bar_color)
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
