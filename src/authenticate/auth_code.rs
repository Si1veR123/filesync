use tokio::sync::mpsc::{self, Sender};

use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::server::conn::AddrStream;
use hyper::{Body, Response, Request, Server};
use hyper::service::{make_service_fn, service_fn};

use super::auth::{CLIENT_ID, PORT};

const AUTH_ROOT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";

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

pub async fn get_auth_code() -> GoogleResponse {
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

    let url = format!("{}{}{}{}{}{}{}{}{}{}",
        AUTH_ROOT, "?",
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