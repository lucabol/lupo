use std::{fs, io, path};
use log::{warn, info};

use crate::errors::*;

fn create_file_if_not_exist(home_dir: &path::Path, file_name: &str) -> crate::errors::Result<()> {

    let res = fs::OpenOptions::new().write(true)
                                .create_new(true)
                                .open(home_dir.join(file_name));
    match &res {
        Ok(_) => info!("{}: file created", file_name),
        Err(e) => {
            if e.kind() == io::ErrorKind::AlreadyExists {
                warn!("{}: file already exists", file_name);
            } else {
                res.chain_err(
                    || format!("{}: error creating the file", file_name))?;
            }
        }
    }
    Ok(())
}

pub fn init(home_dir: &path::Path, force: &bool) -> Result<()> {
    
    if *force && home_dir.is_dir() {
        fs::remove_dir_all(home_dir)
            .chain_err(|| "Could not remove portfolio directory")?;
    }
    let home_dir_str = home_dir.to_string_lossy();

    let _ = fs::create_dir_all(&home_dir)
        .chain_err(||
            format!("Can't create porfolio directory at {}", home_dir_str));
    
    info!("data dir: {}", home_dir_str);

    let _ = create_file_if_not_exist(&home_dir, "stocks.tsv")?;
    let _ = create_file_if_not_exist(&home_dir, "trades.tsv")?;
    Ok(())
}
