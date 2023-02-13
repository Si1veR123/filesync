use google_drive::Client;

use super::auth_code::get_auth_code;
use super::token_storing::{read_refresh_token, write_refresh_token};

pub const CLIENT_ID: &'static str = "957076101620-pe34fuljjgf45on19fhsq8d9upepjj3m.apps.googleusercontent.com";
const CLIENT_SECRET: &'static str = "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";
// currently not used, may be in the future
const _CODE_VERIFIER: &'static str = "mbD-tYXw0716E1Of8Bx0b2Z6a253D8yINEKTZTDvntkSDleLMgWcIrU4krGvHnme4jdrVz8NMPkUwj.5X0FY_9T_FfdZjhSYYi3AOcLPZnLfxykqa-OyiDOt-AWtGmT4";
pub const PORT: u16 = 3087;


async fn new_user_client() -> Client {
    // new user
    // get auth code from browser, and use this to get access token and refresh token

    let auth_code = get_auth_code().await;

    let mut client = Client::new(
        CLIENT_ID,
        CLIENT_SECRET,
        format!("http://127.0.0.1:{}", PORT),
        "",
        ""
    );

    let token = client.get_access_token(&auth_code.code, "").await.unwrap();

    if token.access_token == "" {
        panic!("Error getting access token");
    };

    write_refresh_token(&token.refresh_token).expect("Couldn't write to file");
    println!("Logged in");
    client
}

pub async fn get_drive_client() -> Client {
    // if refresh token exists, use this to get a access token
    // else, prompt user login

    let refresh_token = read_refresh_token();

    match refresh_token {
        Some(refresh_token) => {
            let mut client = Client::new(
                CLIENT_ID,
                CLIENT_SECRET,
                "http://127.0.0.1:0",
                "",
                refresh_token
            );
            let access_token = client.refresh_access_token().await;

            if access_token.is_err() | (access_token.unwrap().access_token == "") {
                // error with refresh token, login again
                client = new_user_client().await;
            }

            client
        },
        None => {
            new_user_client().await
        }
    }
}
