use clap::Parser;
use color_eyre::Result;
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Text to read
    #[arg(short, long, conflicts_with = "file")]
    pub text: Option<String>,

    /// File to read text from
    #[arg(short, long)]
    pub file: Option<String>,

    /// Words per minute
    #[arg(long, default_value = "300")]
    pub wpm: u64,

    /// Inline mode
    #[arg(short, long, default_value = "false")]
    pub inline: Option<bool>,
}

pub fn get_content(args: &Args) -> Result<String> {
    match (&args.file, &args.text) {
        (Some(file), None) => Ok(fs::read_to_string(file)?),
        (None, Some(text)) => Ok(text.clone()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Either --text or --file must be provided",
        )
        .into()),
    }
}
