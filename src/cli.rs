use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Command to execute from ptrace
    #[arg(short, long)]
    pub command: Option<String>,

    /// Attach the tracing to an existing process ID. We're using the `-p` short flag because
    /// strace uses it
    #[arg(short = 'p', long)]
    pub attach: Option<i32>,

    /// Write the output to a file instead of the standard output
    #[arg(short = 'f', long = "file")]
    pub file_to_print: Option<String>,

    /// If defined, it hides the TUI
    #[arg(long = "no-tui", default_value_t = false)]
    pub no_tui: bool,
}
