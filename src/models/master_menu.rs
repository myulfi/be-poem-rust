use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{Insertable, Queryable};
use serde::Serialize;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_menu)]
pub struct MasterMenu {
    pub id: i16,
    #[serde(rename = "name")]
    pub nm: String,
    pub icon: Option<String>,
    #[serde(rename = "sequence")]
    pub seq: i16,
    pub path: String,
    #[serde(rename = "menuParentId")]
    pub mt_menu_parent_id: i16,
    pub color: String,
    #[serde(rename = "newFlag")]
    pub is_new: i16,
    #[serde(rename = "blankTargetFlag")]
    pub is_blank_target: i16,
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
