use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_lang_key)]
pub struct MasterLanguageKey {
    pub id: i64,
    #[serde(rename = "languageTypeId")]
    pub mt_lang_type_id: i16,
    #[serde(rename = "keyCode")]
    pub key_cd: String,
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

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_lang_value)]
pub struct MasterLanguageValue {
    pub id: i64,
    #[serde(rename = "languageId")]
    pub mt_lang_id: i16,
    #[serde(rename = "languageKeyId")]
    pub mt_lang_key_id: i64,
    pub value: String,
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

#[derive(Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryMasterLanguageKey {
    #[serde(rename = "languageTypeId")]
    pub mt_lang_type_id: i16,
    #[serde(rename = "keyCode")]
    #[validate(length(min = 1, message = "Key Code must be filled"))]
    pub key_cd: String,
    pub value: Vec<MasterLanguageValueResponse>,
    #[serde(default)]
    pub version: i16,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MasterLanguageValueResponse {
    #[serde(rename = "languageId")]
    pub mt_lang_id: i16,
    pub value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MasterLanguageKeyResponse {
    pub id: i64,
    #[serde(rename = "languageTypeId")]
    pub mt_lang_type_id: i16,
    #[serde(rename = "keyCode")]
    pub key_cd: String,
    pub value: Vec<MasterLanguageValueResponse>,
}

#[derive(Serialize, Queryable)]
pub struct MasterLanguageKeySummary {
    pub id: i64,
    pub mt_lang_type_id: i16,
    pub key_cd: String,
    pub version: i16,
}
