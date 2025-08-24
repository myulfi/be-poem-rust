use chrono::NaiveDateTime;
use diesel::prelude::Queryable;
use serde::Serialize;

#[derive(Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_database_type)]
pub struct DatabaseType {
    pub id: i16,
    #[serde(rename = "name")]
    pub nm: String,
    pub driver: String,
    pub url: String,
    pub pagination: String,
    #[serde(rename = "deletedFlag")]
    pub is_del: i16,
    pub created_by: String,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<String>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}
