use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_api)]
pub struct ExternalApi {
    pub id: i16,
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
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
#[diesel(table_name = crate::schema::tbl_ext_api)]
#[diesel(treat_none_as_null = true)]
pub struct EntryExternalApi {
    #[serde(rename = "name")]
    #[validate(length(
        min = 1,
        max = 20,
        message = "Code must be between 1 and 20 characters"
    ))]
    pub nm: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(default)]
    pub version: i16,
}
