use tokio::sync::mpsc::{self, Sender};

use std::convert::Infallible;
use std::net::SocketAddr;

use std::fs::{File, OpenOptions};
use std::io::{Read, self, Write};
use std::path::PathBuf;

use home::home_dir;

use hyper::server::conn::AddrStream;
use hyper::{Body, Response, Request, Server};
use hyper::service::{make_service_fn, service_fn};

use google_drive::Client;

const PORT: u16 = 3087;
const CLIENT_ID: &'static str = "957076101620-pe34fuljjgf45on19fhsq8d9upepjj3m.apps.googleusercontent.com";
const CLIENT_SECRET: &'static str = "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";
// currently not used, may be in the future
const _CODE_VERIFIER: &'static str = "mbD-tYXw0716E1Of8Bx0b2Z6a253D8yINEKTZTDvntkSDleLMgWcIrU4krGvHnme4jdrVz8NMPkUwj.5X0FY_9T_FfdZjhSYYi3AOcLPZnLfxykqa-OyiDOt-AWtGmT4";
const SAVED_DATA_FILENAME: &'static str = "AppData/filesync";

#[derive(Debug)]
pub struct GoogleResponse {
    pub code: String,
    pub scope: String
}

fn parse_uri(uri: &str) -> Option<GoogleResponse> {
    // parse the 'GoogleResponse' from the google redirected uri
    let second_part = uri.split("code=").nth(1)?;
    let mut end = second_part.split("&scope=");
    let code = end.next()?.to_string();
    let scope = end.next()?.to_string();
    Some(GoogleResponse { code, scope })
}

async fn handle(tx: Sender<GoogleResponse>, request: Request<Body>) -> Result<Response<Body>, Infallible> {
    // the response from the redirected uri

    // send the GoogleResponse over this channel, ending the server
    tx.send(parse_uri(&request.uri().to_string()).expect("Error in response from google...")).await.unwrap();
    Ok(Response::new(Body::from("You can close this window.")))
}

async fn get_auth_code() -> GoogleResponse {
    // set up a web server to receive the redirected uri from googles auth
    // use mpsc channels to send the auth data, and notify the server to end

    let (tx, mut rx) = mpsc::channel(1);

    let make_service = make_service_fn(move |_conn: &AddrStream| {
        let tx1 = tx.clone();

        let service = service_fn(move |req| {
            handle(tx1.clone(), req)
        });

        async move { Ok::<_, Infallible>(service) }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let server = Server::bind(&addr)
        .serve(make_service);

    let mut login = None;

    let graceful = server.with_graceful_shutdown(
        async {
            // end when receiving data over the channel
            let login_channel = rx.recv().await.unwrap();
            login = Some(login_channel);
        }
    );

    let url = format!("{}{}{}{}{}{}{}{}{}",
        "https://accounts.google.com/o/oauth2/v2/auth?",
        "redirect_uri=http://127.0.0.1:", PORT, "^&",
        "response_type=code^&",
        "scope=https://www.googleapis.com/auth/drive.file^&",
        "client_id=", CLIENT_ID, "^&"
    );

    println!("Opening google auth page...");
    std::process::Command::new("cmd.exe").arg("/C").arg("start").arg(&url).spawn().unwrap();
    println!("Waiting for login...");

    graceful.await.unwrap();

    login.unwrap()
}

fn file_path() -> PathBuf {
    home_dir().unwrap().join(SAVED_DATA_FILENAME)
}

fn read_refresh_token() -> Option<String> {
    // read first line
    let mut file = File::open(file_path()).ok()?;
    let mut string_buf = String::new();
    file.read_to_string(&mut string_buf).ok()?;
    let mut lines = string_buf.split("\r\n");
    Some(lines.nth(0)?.to_string())
}

fn write_refresh_token(refresh_token: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path())?;
    
    let mut string_buf = String::new();
    let _ = file.read_to_string(&mut string_buf)?;
    let lines = string_buf.split("\r\n");
    let lines_left = lines.skip(1);

    let remaining_lines = lines_left.fold(String::new(), |acc, x| format!("{acc}{x}\r\n"));

    let new_data = format!("{}\r\n{}", refresh_token, remaining_lines);
    let _ = file.set_len(0);
    let _ = file.write(new_data.as_bytes())?;
    Ok(())
}

async fn new_user_client() -> Client {
    // new user
    // get auth code from browser, and use this to get access token and refresh token
    let mut client = Client::new(
        CLIENT_ID,
        CLIENT_SECRET,
        format!("http://127.0.0.1:{}", PORT),
        "",
        ""
    );

    let auth_code = get_auth_code().await;
    let token = client.get_access_token(&auth_code.code, "").await.unwrap();

    if token.access_token == "" {
        panic!("Error getting access token");
    };

    write_refresh_token(&token.refresh_token).expect("Couldn't write to file");

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
                format!("http://127.0.0.1:{}", PORT),
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
