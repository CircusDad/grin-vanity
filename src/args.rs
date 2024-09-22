use clap::Parser;

/// A vanity slatepack address generator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Desired prefix pattern
    #[arg(short, long, default_value_t = String::from("grin1234"))]
    pub prefix: String,

    /// Desired suffix pattern
    #[arg(short = 's', long)]
    pub suffix: Option<String>,  // Optional suffix pattern

    /// Threads
    #[arg(short, long, default_value_t = 1)]
    pub threads: usize,

    /// Refresh Interval
    #[arg(short, long, default_value_t = 1)]
    pub interval: usize,
}
