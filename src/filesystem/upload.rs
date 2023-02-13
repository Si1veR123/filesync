use super::zip::compress_dir;
use super::TEMP_ZIP_LOCATION;

use std::env::current_dir;
use std::io::Read;
use std::fs::File;
use zip::CompressionMethod;
use google_drive::AccessToken;
use hyper::{Request, Body, Client, StatusCode};
use hyper_tls::HttpsConnector;

const UPLOAD_ENDPOINT: &'static str = "https://www.googleapis.com/upload/drive/v3/files";

fn create_multipart_data(metadata: Vec<(&str, &str)>, data_bytes: Vec<u8>) -> Vec<u8> {
    let part1_first = "Content-Type: application/json; charset=UTF-8\r\n\r\n{".as_bytes();

    let json_data = metadata.iter().fold(String::new(), |acc, x| format!("{acc}\"{}\":\"{}\",", x.0, x.1));
    let part1_middle;
    if metadata.len() > 0 {
        part1_middle = &json_data.as_bytes()[..json_data.len() - 1];
    } else {
        part1_middle = "".as_bytes();
    }
    let part1_end = "}\r\n".as_bytes();

    let part1 = [part1_first, part1_middle, part1_end].concat();

    let part2_header = "Content-Type: application/zip\r\n\r\n".as_bytes();
    let part2_data = data_bytes.as_slice();
    //let part2_data = "data".as_bytes();
    let part2_end = "\r\n".as_bytes();

    let part2 = [part2_header, part2_data, part2_end].concat();

    let body_boundary = "--filesync_boundary\r\n".as_bytes();
    let end_boundary = "--filesync_boundary--\r\n".as_bytes();

    let body = [body_boundary, part1.as_slice(), body_boundary, part2.as_slice(), end_boundary].concat();
    body
}

async fn upload_zip(token: &AccessToken, path: &str) {
    let url = format!("{}?uploadType=multipart", UPLOAD_ENDPOINT);

    let mut file = File::open(path).unwrap();
    let mut bytes = vec![];
    let _ = file.read_to_end(&mut bytes);

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let multipart_data = create_multipart_data(vec![("name", "name")], bytes);

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

pub async fn upload_to_drive(token: &AccessToken) {
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

    upload_zip(token, zip_path_str).await;
}
