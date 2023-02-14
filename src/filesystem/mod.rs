pub mod zip;
pub mod upload;
pub mod download;

use std::env::current_dir;
use google_drive::{Client, types::File};

pub const TEMP_ZIP_LOCATION: &'static str = "filesync_temp.zip";

pub fn directory_name() -> String {
    current_dir().unwrap().file_name().unwrap().to_os_string().into_string().unwrap()
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

pub async fn get_current_file_id(client: &Client) -> Option<String> {
    let files = get_all_files(client).await;

    let dir_name = directory_name();
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

pub async fn delete_from_cloud(client: &Client) {
    let file_id = get_current_file_id(client).await;

    if let Some(id) = file_id {
        let result = client.files().delete(&id, false, false).await;

        match result {
            Ok(_) => {
                println!("Deleted from cloud...")
            }
            Err(_) => {
                panic!("Couldn't delete from cloud.");
            }
        }
    }
}
