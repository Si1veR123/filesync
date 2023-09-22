pub mod zip;
pub mod upload;
pub mod download;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use google_drive::{Client, types::File};

pub const TEMP_ZIP_LOCATION: &'static str = "filesync_temp.zip";

pub fn current_dir() -> PathBuf {
    match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => panic!("Error getting current working directory: {:?}", e)
    }
}

pub fn get_directory_name(dir: &Path) -> anyhow::Result<&str> {
    let file_name = dir.file_name().unwrap();
    file_name.to_str().ok_or_else(|| anyhow!("Invalid characters in directory name."))
}

pub async fn get_all_files(client: &Client) -> Result<Vec<File>, ()> {
    let files = client.files().list_all(
        "user",
        "",
        false,
        "",
        false,
        "",
        "",
        "appDataFolder",
        false,
        false,
        ""
    ).await;

    match files {
        Ok(files) => Ok(files),
        Err(_) => Err(())
    }
}

pub async fn get_file_id(client: &Client, dir_name: &str) -> Option<String> {
    let files = get_all_files(client).await;

    match files {
        Ok(files) => {
            let mut id = None;
            for file in &files {
                if file.name == dir_name {
                    id = Some(file.id.clone());
                }
            }
            id
        }
        Err(_) => {
            panic!("Error getting files from drive.");
        }
    }
}

pub async fn delete_from_cloud(client: &Client) -> anyhow::Result<()> {
    let cwd = current_dir();
    let directory_name = get_directory_name(&cwd)?;
    let file_id = get_file_id(client, directory_name).await;

    if let Some(id) = file_id {
        let result = client.files().delete(&id, false, false).await;

        match result {
            Ok(_) => {
                println!("Deleted from cloud...")
            }
            Err(_) => {
                return Err(anyhow!("Couldn't delete from cloud."));
            }
        }
    }

    Ok(())
}
