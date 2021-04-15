#![recursion_limit = "1024"]
use std::io::Write;
use std::{collections::HashMap, fmt, fs, io, path};

use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::Deserialize;
use unicode_truncate::UnicodeTruncateStr;

use crate::errors::*;

pub mod errors {
    error_chain::error_chain! {}
}

pub const TRADES_FILE: &str = "trades.tsv";
pub const STOCKS_FILE: &str = "stocks.tsv";

pub struct Store<'a> {
    pub home_dir: &'a path::Path,
}

#[derive(Debug, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
    TrIn,
    Div,
    TrOut,
    Split,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trade<'a> {
    pub account: &'a str,
    #[serde(with = "my_date_format")]
    pub date: DateTime<Utc>,
    pub r#type: TradeType,
    pub stock: &'a str,
    pub units: f64,
    pub price: Option<f64>,
    pub fees: Option<f64>,
    pub split: f64,
    pub currency: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Stocks<'a> {
    pub name: &'a str,
    pub asset: &'a str,
    pub group: &'a str,
    pub tags: &'a str,
    pub riskyness: &'a str,
    pub ticker: Option<&'a str>,
    pub tradedcurrency: &'a str,
    pub currencyunderlying: &'a str,
}

#[derive(Debug)]
pub struct PortLine<'a> {
    pub ticker: Option<&'a str>,
    pub name: &'a str,
    pub currency: &'a str,
    pub asset: &'a str,
    pub group: &'a str,
    pub tags: &'a str,
    pub riskyness: &'a str,
    pub units: f64,
    pub gain: f64,
}

impl<'a> PortLine<'a> {
    fn from(s: &'a Stocks) -> PortLine<'a> {
        PortLine {
            ticker: s.ticker,
            name: s.name,
            currency: s.currencyunderlying,
            asset: s.asset,
            group: s.group,
            tags: s.tags,
            riskyness: s.riskyness,
            units: 0.0,
            gain: 0.0,
        }
    }
}

mod my_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y/%m/%d %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let ks = s + " 00:00:00";
        Utc.datetime_from_str(&ks, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TradeType::Buy => "Buy  ",
                TradeType::Sell => "Sell ",
                TradeType::TrIn => "TrIn ",
                TradeType::TrOut => "TrOut",
                TradeType::Split => "Split",
                TradeType::Div => "Div  ",
            }
        )
    }
}

impl fmt::Display for Trade<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:<10}{:<10}\t{:<7}\t{:>10.2}\t{:<25}\t{:>8.2}\t{:>8.2}",
            self.account.unicode_truncate(10).0,
            self.date.format("%Y/%m/%d"),
            self.r#type,
            self.units,
            self.stock.unicode_truncate(25).0,
            self.price.unwrap_or_default(),
            self.fees.unwrap_or_default()
        )
    }
}

impl fmt::Display for PortLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:<10}{:<25}\t{:<5}\t{:<10}\t{:<15}\t{:<15}\t{:<3}\t{:>10.2}",
            self.ticker
                .as_ref()
                .map_or("<NA>", |t| t.unicode_truncate(10).0),
            self.name.unicode_truncate(25).0,
            self.currency.unicode_truncate(5).0,
            self.asset.unicode_truncate(10).0,
            self.group.unicode_truncate(15).0,
            self.tags.unicode_truncate(15).0,
            self.riskyness.unicode_truncate(3).0,
            self.units,
        )
    }
}

impl Store<'_> {
    pub fn load_trades(&self) -> Result<Vec<Trade>> {
        self.load_csv(TRADES_FILE)
    }
    pub fn load_stocks(&self) -> Result<Vec<Stocks>> {
        self.load_csv(STOCKS_FILE)
    }

    fn load_csv<T>(&self, data: &str) -> Result<Vec<T>>
    where
        //T: for<'de> Deserialize<'de>,
        T: serde::de::DeserializeOwned,
    {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .trim(csv::Trim::All)
            .comment(Some(b'#'))
            .from_reader(data.as_bytes());

        /*
        let raw_record = csv::StringRecord::new();
        let headers = rdr.headers().chain_err(|| "Can't get headers?")?.clone();

        let mut res = Vec::with_capacity(1024);
        while rdr
            .read_record(&mut raw_record)
            .chain_err(|| "Csv not well formed")?
        {
            let record: T = raw_record
                .deserialize(Some(&headers))
                .chain_err(|| "Csv not well formed")?;
            res.push(record);
        }
        Ok(res)
        */
        rdr.deserialize()
            .map(|r| r.chain_err(|| "Badly formatted csv."))
            .collect::<Result<Vec<T>>>()
    }

    pub fn trades(&self, name_substring: Option<String>) -> Result<Vec<Trade>> {
        let trades = self.load_trades()?;
        let s = name_substring.unwrap_or_default().to_lowercase();
        Ok(trades
            .into_iter()
            .filter(move |t| t.stock.to_lowercase().contains(&s))
            .collect())
    }

    pub fn check(&self) -> Result<(usize, usize)> {
        let stocks = self.load_stocks()?;
        let trades = self.load_trades()?;

        let ct = trades.iter().count();
        let cs = stocks.iter().count();
        Ok((ct, cs))
    }

    pub fn port(&self) -> Result<Vec<PortLine>> {
        let stocks = self.load_stocks()?;
        let trades = self.load_trades()?;

        let mut lines: HashMap<_, _> = stocks
            .iter()
            .map(|s| (&s.name, PortLine::from(&s)))
            .collect();

        trades.into_iter().for_each(|t| {
            let mut line = lines
                .get_mut(&t.stock)
                .expect("Error getting a line I just addedd??");

            match t.r#type {
                TradeType::Div => (),
                TradeType::Split => (),
                TradeType::TrIn => (),
                TradeType::TrOut => (),
                TradeType::Buy => line.units += t.units,
                TradeType::Sell => line.units -= t.units,
            }
        });
        Ok(lines
            .into_iter()
            .map(move |(_, v)| v)
            .filter(|l| l.units > 0.01)
            .collect::<Vec<PortLine>>())
    }

    fn create_file_if_not_exist(&self, file_name: &str, header: &str) -> crate::errors::Result<()> {
        let full_path = self.home_dir.join(file_name);
        let str_path = full_path.to_string_lossy();

        let mut res = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&full_path);

        match &mut res {
            Ok(file) => {
                info!("{}: file created", str_path);
                Ok(writeln!(file, "{}", header)
                    .chain_err(|| format!("Cannot write to file {}", str_path))?)
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(warn!("{}: file already exists", str_path))
                } else {
                    res.map(|_| ())
                        .chain_err(|| format!("Error opening {}", str_path))
                }
            }
        }
    }

    pub fn open(home_dir: &path::Path) -> Result<Store> {
        if home_dir.is_dir() {
            Ok(Store { home_dir })
        } else {
            error_chain::bail!("Can't find home directory {}", home_dir.to_string_lossy())
        }
    }

    pub fn new(home_dir: &path::Path, force: bool) -> Result<Store> {
        if force && home_dir.is_dir() {
            fs::remove_dir_all(&home_dir).chain_err(|| "Could not remove portfolio directory")?;
        }
        let home_dir_str = home_dir.to_string_lossy();

        let _ = fs::create_dir_all(&home_dir)
            .chain_err(|| format!("Can't create porfolio directory at {}", home_dir_str));

        let store = Store { home_dir };

        let trade_header = "Account	Date	Type	Stock	Units	Price	Fees	Split	Currency";
        let stocks_header = "Name	Asset	Group	Tags	Riskyness	Ticker	Tradedcurrency	Currencyunderlying";

        store.create_file_if_not_exist(STOCKS_FILE, stocks_header)?;
        store.create_file_if_not_exist(TRADES_FILE, trade_header)?;

        Ok(store)
    }
}
