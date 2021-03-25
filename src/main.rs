#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {}
}

use errors::*;
use log::error;

mod args;
mod common;
mod check;
mod init;


use args::*;
use check::check;
use init::init;

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
        .quiet(opts.quiet)
        .verbosity(opts.verbose + 1)
        .timestamp(opts.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();
    
    match &opts.subcmd {
        SubCommand::Init  { force} =>
            init(&opts.directory.unwrap(), force),
        SubCommand::Check {} => check(&opts),
        SubCommand::List  {} => todo!() 
    }
}
