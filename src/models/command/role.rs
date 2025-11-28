use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{
    prelude::{AsChangeset, Insertable, Queryable},
    sql_types::SmallInt,
};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_role)]
pub struct MasterRole {
    pub id: i16,
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
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

#[derive(Queryable, Serialize, Insertable, Deserialize, Validate, AsChangeset)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[diesel(table_name = crate::schema::tbl_mt_role)]
#[diesel(treat_none_as_null = true)]
pub struct EntryMasterRole {
    #[serde(rename = "name")]
    #[validate(length(
        min = 4,
        max = 100,
        message = "Name must be between 4 and 100 characters"
    ))]
    pub nm: String,
    #[serde(rename = "description")]
    #[validate(length(max = 255, message = "Description must not exceed 255 characters"))]
    pub dscp: Option<String>,
    pub version: i16,
}

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_role_menu)]
pub struct MasterRoleMenu {
    pub id: i64,
    #[serde(rename = "roleId")]
    pub mt_role_id: i16,
    #[serde(rename = "menuId")]
    pub mt_menu_id: i16,
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
