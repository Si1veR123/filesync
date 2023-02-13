use home::home_dir;

use std::fs::{File, OpenOptions};
use std::io::{Read, self, Write};
use std::path::PathBuf;

const SAVED_DATA_FILENAME: &'static str = "AppData/Local/filesync";

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
        .read(true)
        .write(true)
        .create(true)
        .open(file_path())?;
    
    let mut string_buf = String::new();
    let _ = file.read_to_string(&mut string_buf)?;
    let lines = string_buf.split("\r\n");
    let lines_left = lines.skip(1);

    let remaining_lines = lines_left.fold(String::new(), |acc, x| format!("{acc}{x}\r\n"));

    let new_data = format!("{}\r\n{}", refresh_token, remaining_lines);
    let _ = file.set_len(0);
    let _ = file.write(new_data.as_bytes())?;
    Ok(())
}
