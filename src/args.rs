use clap::Clap;
use std::path::PathBuf;

/// A CLI portfolio manager. Supports multiple currencies and automatic
/// download of quotes.
#[derive(Clap)]
#[clap(version = "1.0", author)]
pub struct Opts {
    /// Directory for portfolio files
    #[clap(short, long)]
    pub directory: Option<PathBuf>,

    #[clap(short, long)]
    pub quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,
    /// Timestamp (sec, ms, ns, none)
    #[clap(short, long)]
    pub ts: Option<stderrlog::Timestamp>,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    /// Initialize the portfolio directory
    Init {
        /// Wipes out existing directory
        #[clap(short, long)]
        force: bool,
    },
    /// Check that all portfolio files are well formed
    Check {},
    /// List all trades in the portfolio
    Trades {
        /// Includes just trades with name containing the string
        name_substring: Option<String>,
    },
    /// List all stocks known to the program
    Stocks {
        /// Includes just stocks with name containing the string
        name_substring: Option<String>,
    },
    /// List all portfolio's positions
    Port {
        /// Includes closed positions in the portfolio
        #[clap(short, long)]
        include_closed_pos: bool,
    },
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
