use std::io::{Cursor, stdin};
use std::env::current_dir;
use std::fs::create_dir_all;

use super::{get_current_file_id, directory_name};

use google_drive::{Client, traits::FileOps};
use zip::read;

pub async fn download_from_drive(client: &Client, overwrite: bool) {
    let id = get_current_file_id(client).await;

    println!("Downloading file...");
    let bytes = match id {
        Some(id) => {
            let download = client.files().download_by_id(&id).await;
            match download {
                Ok(download) => {
                    println!("Downloaded.");
                    download
                }
                Err(_) => panic!("Error downloading file")
            }
        }
        None => panic!("No file named {} to download.", directory_name())
    };

    let raw_bytes: Vec<u8> = bytes.into_iter().collect();
    let mut buffer = Cursor::new(raw_bytes);

    let mut zip = read::ZipArchive::new(&mut buffer).expect("Error reading downloaded data.");

    let current_dir = current_dir().unwrap();
    let dir_name = directory_name();

    let current_dir_clone = current_dir.clone();
    let parent = current_dir_clone.parent().unwrap();
    
    // path of the new extracted folder if not overwriting
    let new_name = parent.clone().join(format!("{}_new", dir_name));
    
    let input;
    if overwrite {
        input = "y".to_string()
    } else {
        println!("Overwrite current files (y) or extract to {} (n)?", new_name.file_name().unwrap().to_str().unwrap());
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        input = buf.trim().to_lowercase();
    }

    let res = match input.as_str() {
        "y" => {
            zip.extract(current_dir)
        }
        "n" => {
            create_dir_all(new_name.clone()).unwrap();
            zip.extract(new_name)
        }
        _ => panic!("Invalid input")
    };
    
    match res {
        Ok(_) => {
            println!("Extracted succesfully.");
        }
        Err(e) => {
            println!("Error extracting zip. Some files may be corrupted. Error: {e}")
        }
    }
}
