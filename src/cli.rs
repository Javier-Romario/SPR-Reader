use clap::Parser;
use color_eyre::Result;
use std::{
    fs,
    io::{self, IsTerminal, Read},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Text to read
    #[arg(short, long, conflicts_with = "file")]
    pub text: Option<String>,

    /// File to read text from
    #[arg(short, long)]
    pub file: Option<String>,

    /// Words per minute (at least 1)
    #[arg(long, default_value_t = 300, value_parser = clap::value_parser!(u64).range(1..))]
    pub wpm: u64,

    /// Number of upcoming words to preview below the current word
    #[arg(short = 'p', long)]
    pub preview_words: Option<usize>,

    /// Inline mode (defaults to config value if not specified)
    #[arg(
        short,
        long,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = false,
        action = clap::ArgAction::Set
    )]
    pub inline: Option<bool>,
}

pub fn get_content(args: &Args) -> Result<String> {
    match (&args.file, &args.text) {
        (Some(file), None) => Ok(fs::read_to_string(file)?),
        (None, Some(text)) => Ok(text.clone()),
        (None, None) if !io::stdin().is_terminal() => {
            let mut buf = String::new();
            io::stdin().lock().read_to_string(&mut buf)?;
            Ok(buf)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provide input via --text, --file, or a stdin pipe",
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_text(text: &str) -> Args {
        Args {
            text: Some(text.to_string()),
            file: None,
            wpm: 300,
            preview_words: None,
            inline: None,
        }
    }

    fn args_with_file(path: &str) -> Args {
        Args {
            text: None,
            file: Some(path.to_string()),
            wpm: 300,
            preview_words: None,
            inline: None,
        }
    }

    fn args_empty() -> Args {
        Args {
            text: None,
            file: None,
            wpm: 300,
            preview_words: None,
            inline: None,
        }
    }

    #[test]
    fn get_content_returns_text_arg() {
        assert_eq!(
            get_content(&args_with_text("hello world")).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn get_content_text_empty_string_is_ok() {
        // get_content itself allows empty; main validates non-empty separately
        assert_eq!(get_content(&args_with_text("")).unwrap(), "");
    }

    #[test]
    fn get_content_text_preserves_whitespace() {
        assert_eq!(
            get_content(&args_with_text("  leading and trailing  ")).unwrap(),
            "  leading and trailing  "
        );
    }

    #[test]
    fn get_content_reads_file() {
        let tmp = std::env::temp_dir().join("spr_test_content.txt");
        std::fs::write(&tmp, "file content").unwrap();
        let result = get_content(&args_with_file(tmp.to_str().unwrap())).unwrap();
        std::fs::remove_file(&tmp).ok();
        assert_eq!(result, "file content");
    }

    #[test]
    fn get_content_reads_file_utf8() {
        let tmp = std::env::temp_dir().join("spr_test_utf8.txt");
        std::fs::write(&tmp, "café naïve résumé").unwrap();
        let result = get_content(&args_with_file(tmp.to_str().unwrap())).unwrap();
        std::fs::remove_file(&tmp).ok();
        assert_eq!(result, "café naïve résumé");
    }

    #[test]
    fn get_content_file_not_found_is_error() {
        assert!(get_content(&args_with_file("/nonexistent/path/spr_missing.txt")).is_err());
    }

    #[test]
    fn get_content_no_args_on_tty_is_error() {
        // Skip if stdin is already a pipe (e.g. CI environment)
        if !io::stdin().is_terminal() {
            return;
        }
        let result = get_content(&args_empty());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stdin pipe"));
    }

    #[test]
    fn wpm_zero_is_rejected() {
        use clap::error::ErrorKind;
        let err = Args::try_parse_from(["spr", "--wpm", "0"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }
}
