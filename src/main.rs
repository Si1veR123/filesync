mod filesystem;
mod authenticate;

use filesystem::upload::upload_to_drive;
use filesystem::download::download_from_drive;
use filesystem::{delete_from_cloud, get_directory_name, get_all_files, current_dir};

use authenticate::auth::get_drive_client;
use authenticate::token_storing::delete_refresh_token;
use authenticate::APICredentials;

use tokio;

use std::env::args;

fn help() {
    println!(concat!(
        "About:\n",
        "Filesync uses the current directory's name to download/upload files to google drive\n",
        "From any other device on the same google account, run the download command in a same\n",
        "named directory to download the files\n",
        "Usage:\n",
        "[COMMANDS]\n",
        "up\tUpload the local project to google drive\n",
        "down\tDownload the cloud project from google drive\n\t-o [Overwrite without confirmation]\n",
        "logout\tRemove google account from this device (will not remove this app from your google account)\n",
        "delete\tDelete the project on google drive\n",
        "list\tList projects on google drive\n"
    ));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = args().nth(1);
    if command.is_none() {
        help();
        return Ok(())
    }

    println!("Filesync (directory: {})", get_directory_name(&current_dir())?);

    match command.unwrap().as_str() {
        "up" => {
            let APICredentials(client, token) = get_drive_client().await?;
            upload_to_drive(&client, &token).await?
        },
        "down" => {
            let overwrite = match args().nth(2) {
                None => false,
                Some(arg) => arg.to_lowercase() == "-o"
            };

            let APICredentials(client, _token) = get_drive_client().await?;
            download_from_drive(&client, overwrite).await?
        },
        "logout" => {
            if delete_refresh_token().is_ok() {
                println!("Logged out");
            }
        },
        "delete" => {
            let APICredentials(client, _token) = get_drive_client().await?;
            delete_from_cloud(&client).await?;
        },
        "list" => {
            let APICredentials(client, _token) = get_drive_client().await?;
            let files = get_all_files(&client).await;
            match files {
                Ok(files) => {
                    if files.len() == 0 {
                        println!("No projects found");
                    }

                    for file in &files {
                        println!("{}", file.name);
                    }
                }
                Err(e) => {
                    return Err(e.context("Getting files from drive."));
                }
            }
        },
        _ => {
            println!("Invalid argument given.");
            help();
        }
    }

    Ok(())
}
