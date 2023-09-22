use std::ffi::OsStr;
use std::io::{Cursor, stdin};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use super::{get_file_id, get_directory_name, current_dir};

use google_drive::{Client, traits::FileOps};
use anyhow::anyhow;
use zip::read;

async fn download_bytes_from_id(client: &Client, id: String) -> anyhow::Result<hyper::body::Bytes> {
    let download = client.files().download_by_id(&id).await;
    if download.is_ok() {
        println!("Downloaded.")
    }

    download
}

fn new_download_path<P: AsRef<Path>, S: AsRef<OsStr>>(parent: P, old_name: S) -> PathBuf {
    parent.as_ref().join(format!("{:?}_new", old_name.as_ref()))
}

pub async fn download_from_drive(client: &Client, overwrite: bool) -> anyhow::Result<()> {
    let cwd = current_dir();
    let directory_name = get_directory_name(&cwd)?;

    println!("Downloading file...");
    let id = get_file_id(client, directory_name).await;
    let bytes = match id {
        Some(id) => {
            download_bytes_from_id(client, id).await?
        }
        None => {
            return Err(anyhow!("No file named {} to download.", directory_name))
        }
    };

    let buffer = Cursor::new(bytes);
    let mut zip = read::ZipArchive::new(buffer)?;

    let parent = cwd.parent().unwrap();
    let new_name = new_download_path(parent, directory_name);
    
    if overwrite {
        zip.extract(cwd)?;
    } else {
        println!("Overwrite current files (y) or extract to {} (n)?", get_directory_name(&new_name)?);
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        
        let res = match buf.trim() {
            "y" | "Y" => {
                zip.extract(cwd)
            }
            "n" | "N" => {
                create_dir_all(new_name.clone())?;
                zip.extract(new_name)
            }
            _ => return Err(anyhow!("Invalid input"))
        };

        match res {
            Ok(_) => println!("Extracted succesfully."),
            Err(e) => println!("Error extracting zip. Some files may be corrupted. Error: {e}")
        };
    };

    Ok(())
}
