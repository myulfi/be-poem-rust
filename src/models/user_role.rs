use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use serde::Serialize;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_user_role)]
pub struct UserRole {
    pub id: i64,
    pub user_id: i64,
    #[serde(rename = "roleId")]
    pub mt_role_id: i16,
    #[serde(rename = "deletedFlag")]
    pub is_del: i16,
    pub created_by: i64,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<i64>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}
