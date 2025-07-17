use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};

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

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::tbl_example_template)]
pub struct NewExampleTemplate {
    pub nm: Option<String>,
    pub dscp: Option<String>,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::tbl_example_template)]
#[diesel(treat_none_as_null = true)]
pub struct UpdateExampleTemplate {
    #[serde(rename = "name")]
    pub nm: Option<String>,
    #[serde(rename = "description")]
    pub dscp: Option<String>,
    pub version: i16,
}
