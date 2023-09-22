use home::home_dir;
use anyhow::anyhow;

use std::fs::{File, OpenOptions, remove_file};
use std::io::{self, Write, BufRead, BufReader};
use std::path::PathBuf;

const SAVED_DATA_FILENAME: &'static str = "AppData\\Local\\filesync";

fn file_path() -> PathBuf {
    home_dir().expect("Error getting home directory.").join(SAVED_DATA_FILENAME)
}

pub fn read_refresh_token() -> Option<String> {
    // read first line
    let file = File::open(file_path()).ok()?;
    let buf = BufReader::new(file);
    let mut lines = buf.lines();
    Some(lines.next()?.ok()?)
}

pub fn write_refresh_token(refresh_token: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path())?;

    file.set_len(0)?;
    file.write(refresh_token.as_bytes())?;
    Ok(())
}

pub fn delete_refresh_token() -> anyhow::Result<()> {
    let result = remove_file(file_path());
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!("Error deleting token. You can manually delete from {}", file_path().display())),
    }
}
