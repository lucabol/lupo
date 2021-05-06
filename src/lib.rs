#![recursion_limit = "1024"]
use std::cell::RefCell;
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
pub struct Stocks {
    pub name: String,
    pub asset: String,
    pub group: String,
    pub tags: String,
    pub riskyness: String,
    pub ticker: Option<String>,
    pub tradedcurrency: String,
    pub currencyunderlying: String,
}

#[derive(Debug)]
pub struct PortLine {
    pub ticker: Option<String>,
    pub name: String,
    pub currency: String,
    pub asset: String,
    pub group: String,
    pub tags: String,
    pub riskyness: String,
    pub units: f64,
    pub cost_usd: f64,
    pub revenue_usd: f64,
    pub divs_usd: f64,
    pub fees_usd: f64,
}

impl PortLine {
    fn from(s: &Stocks) -> PortLine {
        PortLine {
            ticker: s.ticker.as_ref().map(|s| s.to_owned()),
            name: s.name.to_owned(),
            currency: s.currencyunderlying.to_owned(),
            asset: s.asset.to_owned(),
            group: s.group.to_owned(),
            tags: s.tags.to_owned(),
            riskyness: s.riskyness.to_owned(),
            units: 0.0,
            cost_usd: 0.0,
            revenue_usd: 0.0,
            divs_usd: 0.0,
            fees_usd: 0.0,
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
            "{:<10}\t{:<10}\t{:<7}\t{:>10.2}\t{:<25}\t{:>8.2}\t{:>8.2}",
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

impl fmt::Display for PortLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:<10}\t{:<25}\t{:<5}\t{:<10}\t{:<15}\t{:<10}\t{:<1}\t{:>10.2}\t{:>10.2}\t{:>10.2}\t{:>10.2}",
            self.ticker
                .as_ref()
                .map_or("<NA>", |t| t.unicode_truncate(10).0),
            self.name.unicode_truncate(25).0,
            self.currency.unicode_truncate(5).0,
            self.asset.unicode_truncate(10).0,
            self.group.unicode_truncate(15).0,
            self.tags.unicode_truncate(10).0,
            self.riskyness.unicode_truncate(1).0,
            self.units,
            self.revenue_usd,
            self.cost_usd,
            self.fees_usd,
        )
    }
}

impl Store<'_> {
    pub fn load_stocks(&self) -> Result<HashMap<String, Stocks>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .trim(csv::Trim::All)
            .comment(Some(b'#'))
            .from_path(self.home_dir.join(STOCKS_FILE))
            .chain_err(|| "Cannot open stocks file")?;

        rdr.deserialize()
            .map(|r: std::result::Result<Stocks, csv::Error>| {
                r.chain_err(|| "Badly formatted csv.")
            })
            .map(|r| r.map(|s| (s.name.clone(), s)))
            .collect::<Result<HashMap<String, Stocks>>>()
    }

    fn trades_fold<R, F>(&self, init: &mut R, f: F) -> Result<()>
    where
        F: Fn(&mut R, Trade) -> (),
    {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .trim(csv::Trim::All)
            .comment(Some(b'#'))
            .from_path(self.home_dir.join(TRADES_FILE))
            .chain_err(|| "Cannot open trades file")?;

        let mut raw_record = csv::StringRecord::new();
        let headers = rdr.headers().chain_err(|| "Can't get headers?")?.clone();

        while rdr
            .read_record(&mut raw_record)
            .chain_err(|| "Csv not well formed")?
        {
            let record: Trade = raw_record
                .deserialize(Some(&headers))
                .chain_err(|| "Csv not well formed")?;
            f(init, record);
        }
        Ok(())
    }

    pub fn trades(&self, name_substring: Option<String>) -> Result<()> {
        let s = name_substring.unwrap_or_default().to_lowercase();
        let mut k = ();
        let f = |_: &mut _, t: Trade| {
            if t.stock.to_lowercase().contains(&s) {
                println!("{}", t)
            }
        };
        self.trades_fold(&mut k, f)?;
        Ok(())
    }

    pub fn stocks(&self, name_substring: Option<String>) -> Result<()> {
        let s = name_substring.unwrap_or_default().to_lowercase();
        let stocks = self.load_stocks()?;

        stocks
            .values()
            .map(|st| PortLine::from(st))
            .filter(|st| st.name.to_lowercase().contains(&s))
            .for_each(|st| println!("{}", st));

        Ok(())
    }

    pub fn check(&self) -> Result<(usize, usize)> {
        let stocks = self.load_stocks()?;
        let cs = stocks.iter().count();

        let f = |c: &mut usize, _: Trade| *c = *c + 1; // if you remove Trade from this line it doesn't work???
        let mut ct = 0;
        self.trades_fold(&mut ct, f)?;

        Ok((ct, cs))
    }

    pub fn port(&self, all: bool) -> Result<Vec<PortLine>> {
        let stocks = self.load_stocks()?;

        let mut lines: HashMap<_, _> = stocks
            .values()
            .map(|s| (s.name.clone(), RefCell::new(PortLine::from(s))))
            .collect();

        let f = |llines: &mut HashMap<String, RefCell<PortLine>>, t: Trade| {
            println!("1:{}", t.stock);
            let mut line = llines.get(t.stock).unwrap().borrow_mut();

            let cash = if !t.stock.contains("Cash") {
                println!("2:{}\t{}", t.stock, t.account);
                Some(
                    llines
                        .get(&format!("Cash{}", t.account)[..])
                        .unwrap()
                        .borrow_mut(),
                )
            } else {
                None
            };

            match t.r#type {
                TradeType::Div => {
                    line.divs_usd += t.units * t.price.unwrap_or_default() * t.currency
                }
                TradeType::Split => line.units = line.units * t.split,
                TradeType::TrIn => {
                    line.units += t.units;
                    line.cost_usd += t.units * t.price.unwrap_or_default() * t.currency
                }
                TradeType::TrOut => {
                    line.units -= t.units;
                    line.revenue_usd += t.units * t.price.unwrap_or_default() * t.currency
                }
                TradeType::Buy => {
                    line.units += t.units;
                    line.cost_usd += t.units * t.price.unwrap_or_default() * t.currency;
                    line.fees_usd += t.fees.unwrap_or_default() * t.currency
                }
                TradeType::Sell => {
                    line.units -= t.units;
                    line.revenue_usd += t.units * t.price.unwrap_or_default() * t.currency;
                    line.fees_usd += t.fees.unwrap_or_default() * t.currency
                }
            }
        };

        self.trades_fold(&mut lines, f)?;

        let ll = lines.into_iter().map(move |(_, v)| v.into_inner());
        if !all {
            Ok(ll
                .filter(|l| !(l.units < 0.01 && l.units > -0.01))
                .collect::<Vec<PortLine>>())
        } else {
            Ok(ll.collect::<Vec<PortLine>>())
        }
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
