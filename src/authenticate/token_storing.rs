use home::home_dir;

use std::fs::{File, OpenOptions, remove_file};
use std::io::{Read, self, Write};
use std::path::PathBuf;

const SAVED_DATA_FILENAME: &'static str = "AppData\\Local\\filesync";

fn file_path() -> PathBuf {
    home_dir().unwrap().join(SAVED_DATA_FILENAME)
}

pub fn read_refresh_token() -> Option<String> {
    // read first line
    let mut file = File::open(file_path()).ok()?;
    let mut string_buf = String::new();
    file.read_to_string(&mut string_buf).ok()?;
    let mut lines = string_buf.split("\r\n");
    Some(lines.nth(0)?.to_string())
}

pub fn write_refresh_token(refresh_token: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path())?;

    let _ = file.set_len(0);
    let _ = file.write(refresh_token.as_bytes())?;
    Ok(())
}

pub fn delete_refresh_token() -> Result<(), String> {
    let result = remove_file(file_path());
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Error deleting token. You can manually delete from {}", file_path().display())),
    }
}
