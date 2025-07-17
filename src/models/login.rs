use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]

pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize, // waktu kedaluwarsa (wajib untuk JWT)
}

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_user)]
pub struct User {
    pub username: String,
    #[serde(rename = "password")]
    pub pass: Option<String>,
    #[serde(rename = "nickName")]
    pub nick_nm: Option<String>,
    #[serde(rename = "fullName")]
    pub full_nm: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub supervisor: Option<String>,
    #[serde(rename = "activeDate")]
    pub dt_active: Option<NaiveDateTime>,
    #[serde(rename = "loginDate")]
    pub dt_login: Option<NaiveDateTime>,
    #[serde(rename = "logoutDate")]
    pub dt_logout: Option<NaiveDateTime>,
    pub ip: Option<String>,
    pub last_access: Option<String>,
    pub agent: Option<String>,
    #[serde(rename = "resignDate")]
    pub dt_resign: Option<NaiveDate>,
    pub created_by: String,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<String>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}

#[derive(Serialize)]
pub struct UserAuthResponse {
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "roleList")]
    pub role: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserAuthResponse,
}
