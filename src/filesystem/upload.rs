use crate::filesystem::current_dir;

use super::zip::compress_dir;
use super::{TEMP_ZIP_LOCATION, get_directory_name, delete_from_cloud};

use std::path::Path;
use std::io::Read;
use std::fs::{File, remove_file};

use zip::CompressionMethod;
use google_drive::{AccessToken, Client};
use hyper::{self, Request, Body, StatusCode};
use hyper_tls::HttpsConnector;
use serde_json;
use anyhow::anyhow;

const URL: &'static str = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";

fn create_multipart_data(metadata: serde_json::Value, data_bytes: Vec<u8>) -> Vec<u8> {
    let part1_header = "Content-Type: application/json; charset=UTF-8\r\n\r\n".as_bytes();
    let metadata_str = metadata.to_string();
    let part1_middle = metadata_str.as_bytes();
    let part1_end = "\r\n".as_bytes();

    let part2_header = "Content-Type: application/zip\r\n\r\n".as_bytes();
    let part2_data = data_bytes.as_slice();
    let part2_end = "\r\n".as_bytes();

    let body_boundary = "--filesync_boundary\r\n".as_bytes();
    let end_boundary = "--filesync_boundary--\r\n".as_bytes();

    [body_boundary, part1_header, part1_middle, part1_end, body_boundary, part2_header, part2_data, part2_end, end_boundary].concat()
}

fn create_json_metadata() -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    let cwd = current_dir();

    let mut map = serde_json::Map::with_capacity(2);
    map.insert(
        "name".to_string(),
        serde_json::Value::String(
            get_directory_name(&cwd)?.to_string()
        )
    );

    map.insert(
        "parents".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("appDataFolder".to_string())
        ])
    );

    Ok(map)
}

async fn upload_zip<P: AsRef<Path>>(token: &AccessToken, path: P) -> anyhow::Result<()> {
    let mut file = File::open(path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build(https);

    println!("Uploading...");

    let map = create_json_metadata()?;
    let metadata = serde_json::Value::Object(map);
    let multipart_data = create_multipart_data(metadata, bytes);

    let request = Request::post(URL)
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Content-Type", "multipart/related; boundary=filesync_boundary")
        .header("Content-Length", multipart_data.len().to_string())
        .body(Body::from(multipart_data))?;
    
    let response = client.request(request).await?;

    match response.status() {
        StatusCode::OK => {
            println!("Uploaded!");
            Ok(())
        },
        _ => {
            Err(anyhow!("Error uploading file: {}", response.status()))
        }
    }
}

pub async fn upload_to_drive(client: &Client, token: &AccessToken) -> anyhow::Result<()> {
    let dir = current_dir();

    let zip_path = dir.join(TEMP_ZIP_LOCATION);
    let zip_path_str = zip_path.to_str().unwrap();

    compress_dir(
        dir.as_os_str().to_str().unwrap(),
        zip_path_str,
        CompressionMethod::Deflated
    )?;
    
    delete_from_cloud(client).await?;
    upload_zip(token, zip_path_str).await?;

    println!("Deleting temporary zip...");
    let res = remove_file(zip_path_str);
    match res {
        Ok(_) => {
            println!("Deleted!");
            Ok(())
        }
        Err(_) => {
            Err(anyhow!("Failed to delete temporary zip."))
        }
    }
}
