use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_database)]
pub struct ExternalDatabase {
    pub id: i16,
    #[serde(rename = "code")]
    pub cd: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "databaseTypeId")]
    pub mt_database_type_id: i16,
    pub username: String,
    pub password: String,
    #[serde(rename = "databaseConnection")]
    pub db_connection: String,
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
#[diesel(table_name = crate::schema::tbl_ext_database)]
#[diesel(treat_none_as_null = true)]
pub struct EntryExternalDatabase {
    #[serde(rename = "code")]
    #[validate(length(
        min = 1,
        max = 20,
        message = "Code must be between 1 and 20 characters"
    ))]
    pub cd: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "databaseTypeId")]
    #[validate(range(min = 1, max = 8, message = "Type must be min 1"))]
    pub mt_database_type_id: i16,
    #[validate(length(min = 1, message = "Username must be filled"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password must be filled"))]
    pub password: String,
    #[serde(rename = "databaseConnection")]
    #[validate(length(min = 1, message = "Database connection must be filled"))]
    pub db_connection: String,
    #[serde(rename = "lockFlag")]
    pub is_lock: i16,
    #[serde(default)]
    pub version: i16,
}

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_database_query)]
pub struct ExternalDatabaseQuery {
    pub id: i64,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "externalDatabaseId")]
    pub ext_database_id: i16,
    pub query: String,
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

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_query_manual)]
pub struct QueryManual {
    pub id: i64,
    #[serde(rename = "externalDatabaseId")]
    pub ext_database_id: i16,
    pub query: String,
    pub created_by: String,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<String>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}

#[derive(Queryable, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EntryQueryManual {
    pub query: String,
}
