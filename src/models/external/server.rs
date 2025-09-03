use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_server)]
pub struct ExternalServer {
    pub id: i16,
    #[serde(rename = "code")]
    pub cd: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "serverTypeId")]
    pub mt_server_type_id: i16,
    pub ip: String,
    pub port: i16,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    #[serde(rename = "lockFlag")]
    pub is_lock: i16,
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

#[derive(Queryable, Serialize, Insertable, Deserialize, Validate, AsChangeset)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[diesel(table_name = crate::schema::tbl_ext_server)]
#[diesel(treat_none_as_null = true)]
pub struct EntryExternalServer {
    #[serde(rename = "code")]
    #[validate(length(
        min = 1,
        max = 20,
        message = "Code must be between 1 and 20 characters"
    ))]
    pub cd: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "serverTypeId")]
    #[validate(range(min = 1, message = "Type must be filled"))]
    pub mt_server_type_id: i16,
    pub ip: String,
    pub port: i16,
    #[validate(length(min = 1, message = "Username must be filled"))]
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    #[serde(default)]
    pub version: i16,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EntryExternalServerDirectory {
    #[serde(rename = "name")]
    #[validate(length(
        min = 1,
        max = 20,
        message = "Name must be between 1 and 20 characters"
    ))]
    pub nm: String,
    #[serde(rename = "directory")]
    #[validate(length(min = 1, message = "Directory must have at least one item"))]
    pub dir: Vec<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct EntryExternalServerFile {
    #[serde(rename = "name")]
    #[validate(length(min = 1, message = "Name must be fill"))]
    pub nm: String,
    pub content: String,
    #[serde(rename = "directory")]
    #[validate(length(min = 1, message = "Directory must have at least one item"))]
    pub dir: Vec<String>,
}
