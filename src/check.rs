use crate::args::Opts;
use crate::errors::Result;

pub fn check(opts: &Opts) -> Result<()> {
    println!("{:?}", opts.directory);
    //let _ = load_trades(&trades_path(&opts)).unwrap();
    Ok(())
}
