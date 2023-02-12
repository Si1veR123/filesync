mod file_diffs;
mod authenticate;

use authenticate::browser::get_drive_client;
use tokio;

#[tokio::main]
async fn main() {
    let client = get_drive_client().await;
}
