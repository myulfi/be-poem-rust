use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_api)]
pub struct ExternalApi {
    pub id: i64,
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "authorization")]
    pub authz: Option<String>,
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

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_api_var)]
pub struct ExternalApiVariable {
    pub id: i64,
    #[serde(rename = "sequence")]
    pub seq: i16,
    #[serde(rename = "externalApiId")]
    pub ext_api_id: i64,
    pub key: String,
    #[serde(rename = "value")]
    pub val: Option<String>,
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
#[diesel(table_name = crate::schema::tbl_ext_api_var)]
#[diesel(treat_none_as_null = true)]
pub struct EntryExternalApiVariable {
    #[validate(length(min = 1, message = "Key must be filled"))]
    pub key: String,
    #[serde(rename = "value")]
    pub val: Option<String>,
    #[serde(default)]
    pub version: i16,
}

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_ext_api_req)]
pub struct ExternalApiRequest {
    pub id: i64,
    #[serde(rename = "sequence")]
    pub seq: i16,
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "externalApiId")]
    pub ext_api_id: i64,
    pub parent_id: i64,
    #[serde(rename = "httpMethodId")]
    pub mt_http_method_id: i16,
    pub path: Option<String>,
    #[serde(rename = "haveAuthorizationFlag")]
    pub is_have_authz: i16,
    pub body: Option<String>,
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
pub struct ExternalApiRequestNode {
    pub id: i64,
    #[serde(rename = "sequence")]
    pub seq: i16,
    #[serde(rename = "name")]
    pub nm: String,
    #[serde(rename = "externalApiId")]
    pub ext_api_id: i64,
    pub parent_id: i64,
    #[serde(rename = "httpMethodId")]
    pub mt_http_method_id: i16,
    pub path: Option<String>,
    #[serde(rename = "haveAuthorizationFlag")]
    pub is_have_authz: i16,
    pub body: Option<String>,
    #[serde(rename = "deletedFlag")]
    pub is_del: i16,
    pub created_by: String,
    #[serde(rename = "dateCreated")]
    pub dt_created: NaiveDateTime,
    pub updated_by: Option<String>,
    #[serde(rename = "dateUpdated")]
    pub dt_updated: Option<NaiveDateTime>,
    pub version: i16,
    pub children: Vec<ExternalApiRequestNode>,
}

#[derive(Queryable, Serialize, Insertable, Deserialize, Validate, AsChangeset)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[diesel(table_name = crate::schema::tbl_ext_api_req)]
#[diesel(treat_none_as_null = true)]
pub struct EntryExternalApiRequest {
    #[serde(rename = "name")]
    #[validate(length(min = 1, message = "Key must be filled"))]
    pub nm: String,
    #[serde(rename = "httpMethodId")]
    pub mt_http_method_id: i16,
    pub path: Option<String>,
    #[serde(rename = "haveAuthorizationFlag")]
    is_have_authz: i16,
    body: String,
    #[serde(default)]
    pub version: i16,
}
