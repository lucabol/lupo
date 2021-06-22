use clap::Clap;
use std::path::PathBuf;

/// Provides portfolio services: tracks trades and position, automatically downloads prices
/// & reports on portfolio risk factors.
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
        /// Edit the trades file by opening the default editor
        #[clap(short, long)]
        edit: bool,

        /// Includes just trades with name containing the string
        name_substring: Option<String>,
    },
    /// List all stocks known to the program
    Stocks {
        /// Edit the stocks file by opening the default editor
        #[clap(short, long)]
        edit: bool,

        /// Includes just stocks with name containing the string
        name_substring: Option<String>,
    },
    /// List all portfolio's positions
    Port {
        /// Includes closed positions in the portfolio
        #[clap(short, long)]
        all: bool,

        /// Separate cash positions by account
        #[clap(short, long)]
        separate_cash: bool,

        /// Field to sort positions on
        #[clap(subcommand)]
        sort_by: Option<SortField>,
    },
    /// Report on the portfolio exposure to various risks
    Report {
        /// Type of report to generate
        #[clap(subcommand)]
        report_type: ReportType,
    },
    /// Update prices of all stock owned using the Yahoo finance API
    UpdatePrices {},
    /// Total value of the portfolio
    Total {},
}

#[derive(Clap)]
pub enum SortField {
    Ticker,
    Name,
    Amount,
    Currency,
    Asset,
    Group,
    Tags,
    Riskyness,
    Gain,
    Tax,
}

#[derive(Clap)]
pub enum ReportType {
    /// By currency
    Currency,
    /// By asset type
    Asset,
    /// By group
    Group,
    /// By level of risk
    Riskyness,
    /// By tags (user defined)
    Tags,
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
