use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

#[derive(Insertable, Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::tbl_example_template)]
pub struct ExampleTemplate {
    pub id: i64,
    #[serde(rename = "name")]
    pub nm: Option<String>,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "value")]
    pub val: Option<i16>,
    #[serde(rename = "amount")]
    pub amt: Option<BigDecimal>,
    #[serde(rename = "date")]
    pub dt: Option<NaiveDate>,
    pub foreign_id: Option<i64>,
    #[serde(rename = "activeFlag")]
    pub is_active: i16,
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

#[derive(Queryable, Serialize, Insertable, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[diesel(table_name = crate::schema::tbl_example_template)]
pub struct NewExampleTemplate {
    #[serde(rename = "name")]
    pub nm: Option<String>,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    #[serde(rename = "value")]
    pub val: Option<i16>,
    #[serde(rename = "amount")]
    pub amt: Option<BigDecimal>,
    #[serde(rename = "date")]
    pub dt: Option<NaiveDate>,
    pub foreign_id: Option<i64>,
}

#[derive(Deserialize, AsChangeset, Validate)]
#[diesel(table_name = crate::schema::tbl_example_template)]
#[diesel(treat_none_as_null = true)]
pub struct UpdateExampleTemplate {
    #[serde(rename = "name")]
    #[validate(length(
        min = 4,
        max = 100,
        message = "Name must be between 4 and 100 characters"
    ))]
    pub nm: Option<String>,
    #[serde(rename = "description")]
    #[validate(length(max = 255, message = "Description must not exceed 255 characters"))]
    pub dscp: Option<String>,
    #[serde(rename = "value")]
    #[validate(range(min = 1, max = 8, message = "Value must be between 1 and 8"))]
    pub val: Option<i16>,
    #[serde(rename = "amount")]
    pub amt: Option<BigDecimal>,
    #[serde(rename = "date")]
    pub dt: Option<NaiveDate>,
    pub foreign_id: Option<i64>,
    pub version: i16,
}
