use lupo::errors::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

//use pretty_assertions::{assert_eq, assert_ne};
use tempfile::tempdir;

// Can't create this as a standard function because 'store' borrows 'home'
macro_rules! temp_store {
    ($var:ident, $home:ident, $force:expr) => {
        let $home = tempdir().chain_err(|| "Can't create temporary dir")?;
        let $var = lupo::Store::new($home.as_ref(), $force)?;
    };
}

#[test]
fn can_init_not_existing_store() -> Result<()> {
    temp_store!(store, home, false);
    let (ct, cs) = store.check()?;
    assert_eq!(0, ct);
    assert_eq!(0, cs);
    Ok(())
}
#[test]
fn can_init_existing_store() -> Result<()> {
    temp_store!(_store, home, false);

    // add trade
    let new_trade = "IB	2015/04/27	TrIn	CashIB	1335387	1	0	1	1";
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(home.path().join("trades.tsv"))
        .chain_err(|| "Can't open trade file")?;
    writeln!(file, "{}", new_trade).chain_err(|| "Can't print to trade file")?;
    let (ct, cs) = _store.check()?;
    assert_eq!(1, ct);
    assert_eq!(0, cs);

    let store = lupo::Store::new(home.as_ref(), false)?;
    let (ct, cs) = store.check()?;
    assert_eq!(1, ct);
    assert_eq!(0, cs);
    Ok(())
}
#[test]
fn can_init_forcefully_existing_store() -> Result<()> {
    temp_store!(_store, home, false);

    // add trade
    let new_trade = "IB	2015/04/27	TrIn	CashIB	1335387	1	0	1	1";
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(home.path().join("trades.tsv"))
        .chain_err(|| "Can't open trade file")?;
    writeln!(file, "{}", new_trade).chain_err(|| "Can't print to trade file")?;
    let (ct, cs) = _store.check()?;
    assert_eq!(1, ct);
    assert_eq!(0, cs);

    // open the store forcefully, should reset the trades
    let store = lupo::Store::new(home.as_ref(), true)?;
    let (ct, cs) = store.check()?;
    assert_eq!(0, ct);
    assert_eq!(0, cs);
    Ok(())
}
#[test]
fn check_err_if_invalid_trade() -> Result<()> {
    temp_store!(_store, home, false);

    // add trade
    let new_trade = "IB	2015/04/27	XTrIn	CashIB	1335387	1	0	1	1";
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(home.path().join("trades.tsv"))
        .chain_err(|| "Can't open trade file")?;
    writeln!(file, "{}", new_trade).chain_err(|| "Can't print to trade file")?;
    let r = _store.check();
    assert_eq!(true, r.is_err());
    Ok(())
}
