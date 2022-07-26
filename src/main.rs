use log::error;

use lupo::errors::*;
use lupo::*;

use itertools::Itertools;
use lupo::args::*;

// Rust doesn't trap a unix signal appropriately occasionally: https://github.com/rust-lang/rust/issues/46016
fn reset_signal_pipe_handler() -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)
                .chain_err(|| "Internal error: cannot trap signal")?;
        }
    }

    Ok(())
}
#[tokio::main]
async fn main() {
    reset_signal_pipe_handler().unwrap();

    if let Err(ref e) = run().await {
        let mut s = e.to_string();

        for e in e.iter().skip(1) {
            s.push_str(&format!("\n\tcaused by: {}", e));
        }

        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            s.push_str(&format!("\n\tbacktrace:\n{:?}", backtrace));
        }

        error!("{}", s);

        ::std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let opts = parse_args();

    stderrlog::new()
        .module(module_path!())
        .show_level(false)
        .quiet(opts.quiet)
        .verbosity(opts.verbose + 1) // The user needs warnings
        .timestamp(opts.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let home_dir = &opts.directory.unwrap();

    match opts.subcmd {
        SubCommand::Init { force } => {
            let store = Store::new(home_dir, force)?;
            println!("Data directory: {}", store.home_dir.to_string_lossy());
            Ok(())
        }
        SubCommand::Check {} => {
            let store = Store::open(home_dir)?;
            let (ct, cs) = store.check()?;
            println!("{} trades processed correctly.", ct);
            println!("{} stocks processed correctly.", cs);
            Ok(())
        }
        SubCommand::Trades {
            name_substring,
            edit,
        } => {
            let store = Store::open(home_dir)?;

            if edit {
                store.edit_trades()
            } else {
                println!(
                    fmt_trade!(),
                    "ACCOUNT", "DATE", "TYPE", "UNITS", "NAME", "PRICE", "FEES"
                );
                store.trades(name_substring)?;
                Ok(())
            }
        }
        SubCommand::Stocks {
            name_substring,
            edit,
        } => {
            let store = Store::open(home_dir)?;
            if edit {
                store.edit_stocks()
            } else {
                store.stocks(name_substring)?;
                Ok(())
            }
        }
        SubCommand::Port {
            all,
            separate_cash,
            sort_by,
        } => {
            let store = Store::open(home_dir)?;
            let mut v = store.port(all, separate_cash)?;

            if let Some(sort_by_field) = sort_by {
                match sort_by_field {
                    SortField::Account => v.sort_by(|a, b| a.account.cmp(&b.account)),
                    SortField::Amount => {
                        v.sort_by(|a, b| b.amount_usd.partial_cmp(&a.amount_usd).unwrap())
                    }
                    SortField::Ticker => v.sort_by(|a, b| a.ticker.cmp(&b.ticker)),
                    SortField::Name => v.sort_by(|a, b| a.name.cmp(&b.name)),
                    SortField::Currency => v.sort_by(|a, b| a.currency.cmp(&b.currency)),
                    SortField::Asset => v.sort_by(|a, b| a.asset.cmp(&b.asset)),
                    SortField::Group => v.sort_by(|a, b| a.group.cmp(&b.group)),
                    SortField::Tags => v.sort_by(|a, b| a.tags.cmp(&b.tags)),
                    SortField::Riskyness => v.sort_by(|a, b| a.riskyness.cmp(&b.riskyness)),
                    SortField::Gain => v.sort_by(|a, b| b.gain.partial_cmp(&a.gain).unwrap()),
                    SortField::Tax => v.sort_by(|a, b| a.tax_status.cmp(&b.tax_status)),
                }
            } else {
                // By default sorts by name.
                v.sort_by(|a, b| a.name.cmp(&b.name))
            }
            println!(
                fmt_portline!(),
                "ACCOUNT",
                "%",
                "TICKER",
                "NAME",
                "CUR",
                "ASSET",
                "GROUP",
                "TAGS",
                "R",
                "UNITS",
                "PRICE",
                "AMOUNT",
                "GAIN",
                "TAX",
                "ER"
            );
            v.iter().for_each(|l| println!("{}", l));
            Ok(())
        }
        SubCommand::Report { report_type } => {
            let store = Store::open(home_dir)?;
            let rll = store
                .report(report_type)?
                .sorted_by(|a, b| b.amount_usd.partial_cmp(&a.amount_usd).unwrap());
            println!(fmt_report!(), "GROUP", "AMOUNT", "% TOT");
            rll.for_each(|rl| println!("{}", rl));
            Ok(())
        }
        SubCommand::Total {} => {
            let store = Store::open(home_dir)?;
            let tot = store.total()?;
            println!("USD\t{:<10}", tot.sep());
            Ok(())
        }
        SubCommand::UpdatePrices {} => {
            let store = Store::open(home_dir)?;
            store.update_prices().await
        }
    }
}
