mod filesystem;
mod authenticate;

use filesystem::upload::upload_to_drive;

use authenticate::auth::get_drive_client;
use tokio;
use std::env::args;

fn help() {
    println!(concat!(
        "Usage:\n",
        "[COMMANDS]\n",
        "up\tUpload the local project to google drive\n",
        "down\tDownload the cloud project from google drive\n",
        "[OPTIONS]\n",
        "-a\tUpload/Download all files without confirmation\n"
    ));
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let command = args().nth(1);
    if command.is_none() {
        help();
        return Ok(())
    }

    let (client, token) = get_drive_client().await;

    match command.unwrap().as_str() {
        "up" => {
            upload_to_drive(&token).await
        },
        "down" => {

        },
        _ => {
            println!("Invalid argument given.");
            help();
            return Ok(());
        }
    }

    Ok(())
}
