use super::zip::compress_dir;
use super::{TEMP_ZIP_LOCATION, directory_name, delete_from_cloud};

use std::path::PathBuf;
use std::env::current_dir;
use std::io::Read;
use std::fs::{File, remove_file};
use zip::CompressionMethod;
use google_drive::{AccessToken, Client};
use hyper::{self, Request, Body, StatusCode};
use hyper_tls::HttpsConnector;
use serde_json;

const UPLOAD_ENDPOINT: &'static str = "https://www.googleapis.com/upload/drive/v3/files";

fn create_multipart_data(metadata: serde_json::Value, data_bytes: Vec<u8>) -> Vec<u8> {
    let part1_header = "Content-Type: application/json; charset=UTF-8\r\n\r\n".as_bytes();
    let metadata_str = metadata.to_string();
    let part1_middle = metadata_str.as_bytes();
    let part1_end = "\r\n".as_bytes();

    let part2_header = "Content-Type: application/zip\r\n\r\n".as_bytes();
    let part2_data = data_bytes.as_slice();
    //let part2_data = "data".as_bytes();
    let part2_end = "\r\n".as_bytes();

    let body_boundary = "--filesync_boundary\r\n".as_bytes();
    let end_boundary = "--filesync_boundary--\r\n".as_bytes();

    [body_boundary, part1_header, part1_middle, part1_end, body_boundary, part2_header, part2_data, part2_end, end_boundary].concat()
}

async fn upload_zip(token: &AccessToken, path: &str) {
    let url = format!("{UPLOAD_ENDPOINT}?uploadType=multipart");

    let path = PathBuf::try_from(path).unwrap();

    let mut file = File::open(path).unwrap();
    let mut bytes = vec![];
    let _ = file.read_to_end(&mut bytes);

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let metadata = serde_json::Value::Object(
        serde_json::Map::from_iter([
            ("name".to_string(), serde_json::Value::String(directory_name())),
            ("parents".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("appDataFolder".to_string())]))
        ].into_iter())
    );

    let multipart_data = create_multipart_data(metadata, bytes);

    println!("Uploading...");

    let request = Request::post(url)
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Content-Type", "multipart/related; boundary=filesync_boundary")
        .header("Content-Length", multipart_data.len().to_string())
        .body(Body::from(multipart_data))
        .unwrap();
    
    let response = client.request(request).await.unwrap();

    match response.status() {
        StatusCode::OK => {
            println!("Uploaded!");
        },
        _ => {
            panic!("Error uploading file: {}", response.status());
        }
    }
}

pub async fn upload_to_drive(client: &Client, token: &AccessToken) {
    // use client for the google drive crate's api methods
    // use token for more custom api calls
    let dir = current_dir().unwrap();

    let zip_path = dir.join(TEMP_ZIP_LOCATION);
    let zip_path_str = zip_path.to_str().unwrap();

    let res = compress_dir(
        dir.as_os_str().to_str().unwrap(),
        zip_path_str,
        CompressionMethod::Deflated
    );

    if res.is_err() {
        panic!("Failed to compress folder");
    }

    delete_from_cloud(client).await;
    upload_zip(token, zip_path_str).await;
    println!("Deleting temporary zip...");
    let res = remove_file(zip_path_str);
    match res {
        Ok(_) => {
            println!("Deleted!");
        }
        Err(_) => {
            panic!("Failed to delete temporary zip.");
        }
    }
}
