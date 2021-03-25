use std::{fs::{OpenOptions, create_dir_all}};
use std::io;

use crate::args::Opts;
use crate::errors::*;

fn create_file_if_not_exist(home_dir: &std::path::Path, file_name: &str) -> crate::errors::Result<()> {

    let res = OpenOptions::new().write(true)
                                .create_new(true)
                                .open(home_dir.join(file_name));
    match &res {
        Ok(_) => println!("{}: file created", file_name),
        Err(e) => {
            if e.kind() == io::ErrorKind::AlreadyExists {
                println!("{}: file already exists", file_name);
            } else {
                res.chain_err(
                    || format!("{}: error creating the file", file_name))?;
            }
        }
    }
    Ok(())
}

pub fn init(opts: Opts) -> Result<()> {
    let home_dir = opts.directory.unwrap();
    let home_dir_str = home_dir.to_string_lossy();

    let _ = create_dir_all(&home_dir)
        .chain_err(||
            format!("Can't create porfolio directory at {}", home_dir_str));
    
    println!("data dir: {}", home_dir_str);

    let _ = create_file_if_not_exist(&home_dir, "stocks.tsv")?;
    let _ = create_file_if_not_exist(&home_dir, "trades.tsv")?;
    Ok(())
}
