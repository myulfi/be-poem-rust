use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserAuthResponse,
}

#[derive(Serialize)]
pub struct UserAuthResponse {
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "roleList")]
    pub role: Vec<u8>,
}
