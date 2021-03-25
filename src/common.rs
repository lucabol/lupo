/*
use std::path::PathBuf;

use serde::Deserialize;

use crate::args::Opts;
use crate::errors::Result;

fn trades_path(opts: &Opts) -> PathBuf {
    [&opts.directory.as_ref().unwrap(), "trades.tsv"]
        .iter().collect()
}

#[derive(Debug, Deserialize)]
struct Trade {
    account: String,
    date: String,
    r#type: String,
    units: f64,
    price: f64,
    fees: f64,
    split: f64,
    currency: f64
}

fn load_trades(path: &PathBuf) -> Result<()> {
    let mut rdr = csv::Reader::from_path(path).expect("Trades file not found");
    for result in rdr.deserialize() {
        let record: Trade = result?;
        println!("{:?}", record);
    }
    Ok(())
}
*/