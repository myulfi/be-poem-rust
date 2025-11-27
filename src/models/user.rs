use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{Insertable, Queryable};
use serde::Serialize;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_user)]
pub struct User {
    pub id: i64,
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
    pub created_by: i64,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<i64>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}
