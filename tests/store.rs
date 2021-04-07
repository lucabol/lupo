use lupo::errors::*;

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
    store.check()?;
    Ok(())
}
#[test]
fn can_init_existing_store() -> Result<()> {
    temp_store!(_store, home, false);

    let store = lupo::Store::new(home.as_ref(), false)?;
    store.check()?;
    Ok(())
}
