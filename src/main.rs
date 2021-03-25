#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {}
}

use errors::*;

mod args;
mod common;
mod check;
mod init;


use args::*;
use check::check;
use init::init;

quick_main!(run);

fn run() -> Result<()> {
    let opts = parse_args();

    match &opts.subcmd {
        SubCommand::Init  { force} => init(opts).chain_err(|| "Error in Init"),
        SubCommand::Check {} => check(&opts),
        SubCommand::List  {} => todo!() 
    }
}
