use log::error;

mod args;
use args::*;

use lupo::errors::*;
use lupo::*;

fn main() {
    if let Err(ref e) = run() {
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

fn run() -> Result<()> {
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
            let _ = Store::new(home_dir, force)?;
            Ok(())
        }
        SubCommand::Check {} => {
            let store = Store::open(home_dir)?;
            store.check()?;
            Ok(())
        }
        SubCommand::Trades { name_substring } => {
            let store = Store::open(home_dir)?;
            store.trades(name_substring)?;
            Ok(())
        }
    }
}
