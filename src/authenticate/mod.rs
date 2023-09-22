pub mod auth;
pub mod token_storing;
pub mod auth_code;

pub struct APICredentials(pub google_drive::Client, pub google_drive::AccessToken);
