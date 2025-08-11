use chrono::NaiveDateTime;
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
    pub path: Option<String>,
    #[serde(rename = "menuParentId")]
    pub mt_menu_parent_id: i16,
    pub color: Option<String>,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuNode {
    pub id: i16,
    pub name: String,
    pub icon: Option<String>,
    pub sequence: i16,
    pub path: Option<String>,
    pub menu_parent_id: i16,
    pub color: Option<String>,
    pub new_flag: i16,
    pub blank_target_flag: i16,
    pub deleted_flag: i16,
    pub created_by: String,
    pub created_date: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_date: Option<NaiveDateTime>,
    pub version: i16,
    pub children: Vec<MenuNode>, // 👈 tambahan children
}
