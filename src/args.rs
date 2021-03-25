use clap::Clap;
use std::path::PathBuf;

/// A CLI portfolio manager. Supports multiple currencies and automatic
/// download of quotes.
#[derive(Clap)]
#[clap(version = "1.0", author)]
pub struct Opts {
    /// Directory that stores portfolio files
    #[clap(short, long)]
    pub directory: Option<PathBuf>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Init(Init),
    Check(Check),
    List(List)
}

/// Check that all portfolio files are well formed
#[derive(Clap)]
pub struct Check {
}

/// List all trades in the portfolio
#[derive(Clap)]
pub struct List {
}

/// Initialize the portfolio directory
#[derive(Clap)]
pub struct Init {
}

pub fn parse_args() -> Opts {
    let opts = Opts::parse();
    if opts.directory.is_none() {
        let mut dd = dirs::data_dir().expect("Cannot find an home directory on this system");
        dd.push("lupo");
        Opts {
            directory: Some(dd),
            ..opts
        }
    } else {
        opts
    }
}