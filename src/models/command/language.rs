use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_mt_lang_key)]
pub struct MasterLanguageKey {
    pub id: i64,
    #[serde(rename = "labelType")]
    pub label_typ: String,
    #[serde(rename = "keyCode")]
    pub key_cd: String,
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
    pub created_by: String,
    #[serde(rename = "createdDate")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<String>,
    #[serde(rename = "updatedDate")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
}

#[derive(Serialize)]
pub struct MasterLanguageValueResponse {
    pub mt_lang_id: i16,
    pub value: String,
}

#[derive(Serialize)]
pub struct MasterLanguageKeyResponse {
    pub id: i64,
    pub label_typ: String,
    pub key_cd: String,
    pub value: Vec<MasterLanguageValueResponse>,
}
