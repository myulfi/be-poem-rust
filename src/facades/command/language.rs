use std::collections::HashMap;
use std::fs;

use crate::models::command::language::{
    EntryMasterLanguageKey, MasterLanguageKeyResponse, MasterLanguageKeySummary,
    MasterLanguageValueResponse,
};
use crate::models::common::{DataResponse, PaginatedResponse};
use crate::schema::{tbl_mt_lang, tbl_mt_lang_key, tbl_mt_lang_type, tbl_mt_lang_value};
use crate::utils::common::{
    self, parse_ids_from_string, validate_id, validate_ids, validation_error_response,
};
use crate::{
    db::DbPool,
    models::command::language::{MasterLanguageKey, MasterLanguageValue},
    models::common::Pagination,
};
// use bigdecimal::BigDecimal;
// use bigdecimal::num_bigint::BigInt;
use chrono::Utc;
use diesel::prelude::*;
use poem::IntoResponse;
use poem::web::Query;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Path},
};
use serde_json::json;
use validator::Validate;

#[handler]
pub fn list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Query(pagination): Query<Pagination>,
) -> poem::Result<impl IntoResponse> {
    let start = pagination.start.unwrap_or(0);
    let length = pagination.length.unwrap_or(10).min(100);

    let mut query = tbl_mt_lang_key::table.into_boxed();
    if let Some(ref term) = pagination.search {
        query = query.filter(tbl_mt_lang_key::key_cd.ilike(format!("%{}%", term)));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let total: i64 = match query.count().get_result(conn) {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Counting error: {}", e);
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            ));
        }
    };

    if total > 0 {
        let mut query = tbl_mt_lang_key::table.into_boxed();
        if let Some(ref term) = pagination.search {
            query = query.filter(tbl_mt_lang_key::key_cd.ilike(format!("%{}%", term)));
        }

        match (pagination.sort.as_deref(), pagination.dir.as_deref()) {
            (Some("keyCode"), Some("desc")) => query = query.order(tbl_mt_lang_key::key_cd.desc()),
            (Some("keyCode"), _) => query = query.order(tbl_mt_lang_key::key_cd.asc()),
            (Some("createdDate"), Some("desc")) => {
                query = query.order(tbl_mt_lang_key::dt_created.desc())
            }
            (Some("createdDate"), _) => query = query.order(tbl_mt_lang_key::dt_created.asc()),
            _ => {}
        }

        let data = query
            .offset(start)
            .limit(length)
            .load::<MasterLanguageKey>(conn)
            .map_err(|e| {
                eprintln!("Loading error: {}", e);
                common::error_message(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "information.internalServerError",
                )
            })?;

        let data_ids: Vec<i64> = data.iter().map(|k| k.id).collect();

        let values: Vec<MasterLanguageValue> = tbl_mt_lang_value::table
            .filter(tbl_mt_lang_value::is_del.eq(0))
            .filter(tbl_mt_lang_value::mt_lang_key_id.eq_any(&data_ids))
            .load(conn)
            .map_err(|e| {
                eprintln!("Load values error: {}", e);
                common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "load_values_error")
            })?;

        let mut map: HashMap<i64, Vec<MasterLanguageValueResponse>> = HashMap::new();
        for val in values {
            map.entry(val.mt_lang_key_id)
                .or_default()
                .push(MasterLanguageValueResponse {
                    mt_lang_id: val.mt_lang_id,
                    value: val.value,
                });
        }

        let result: Vec<MasterLanguageKeyResponse> = data
            .into_iter()
            .map(|k| MasterLanguageKeyResponse {
                id: k.id,
                mt_lang_type_id: k.mt_lang_type_id,
                key_cd: k.key_cd,
                value: map.remove(&k.id).unwrap_or_else(Vec::new),
            })
            .collect();

        Ok(Json(PaginatedResponse {
            total,
            data: result,
        }))
    } else {
        Ok(Json(PaginatedResponse {
            total: 0,
            data: vec![],
        }))
        // Err(common::error_message(StatusCode::NOT_FOUND, "Not Found"))
    }
}

#[handler]
pub fn get(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(mt_lang_key_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(mt_lang_key_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let mt_lang_key = tbl_mt_lang_key::table
        .select((
            tbl_mt_lang_key::id,
            tbl_mt_lang_key::mt_lang_type_id,
            tbl_mt_lang_key::key_cd,
            tbl_mt_lang_key::version,
        ))
        .filter(tbl_mt_lang_key::id.eq(mt_lang_key_id))
        .first::<MasterLanguageKeySummary>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    let mt_lang_value_vec = tbl_mt_lang_value::table
        .select((tbl_mt_lang_value::mt_lang_id, tbl_mt_lang_value::value))
        .filter(tbl_mt_lang_value::mt_lang_key_id.eq(mt_lang_key_id))
        .load::<(i16, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    Ok(Json(DataResponse {
        data: json!({
            "id": mt_lang_key.id,
            "languageTypeId": mt_lang_key.mt_lang_type_id,
            "keyCode": mt_lang_key.key_cd,
            "value": mt_lang_value_vec
                .into_iter()
                .map(|val| MasterLanguageValueResponse {
                    mt_lang_id: val.0,
                    value: val.1,
                })
                .collect::<Vec<_>>(),
            "version": mt_lang_key.version,
        }),
    }))
}

#[handler]
pub fn add(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Json(entry_mt_lang_key): Json<EntryMasterLanguageKey>,
) -> poem::Result<impl IntoResponse> {
    if let Err(e) = entry_mt_lang_key.validate() {
        return Err(validation_error_response(e));
    }

    let user_id = jwt_auth.claims.user_id.clone();
    let mt_lang_key = MasterLanguageKey {
        id: common::generate_id(),
        mt_lang_type_id: entry_mt_lang_key.mt_lang_type_id,
        key_cd: entry_mt_lang_key.key_cd,
        is_del: 0,
        created_by: user_id,
        dt_created: Utc::now().naive_utc(),
        updated_by: None,
        dt_updated: None,
        version: 0,
    };

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let inserted = diesel::insert_into(tbl_mt_lang_key::table)
        .values(&mt_lang_key)
        .get_result::<MasterLanguageKey>(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    diesel::insert_into(tbl_mt_lang_value::table)
        .values(
            entry_mt_lang_key
                .value
                .into_iter()
                .map(|val| MasterLanguageValue {
                    id: common::generate_id(),
                    mt_lang_id: val.mt_lang_id,
                    mt_lang_key_id: inserted.id,
                    value: val.value,
                    is_del: 0,
                    created_by: user_id,
                    dt_created: Utc::now().naive_utc(),
                    updated_by: None,
                    dt_updated: None,
                    version: 0,
                })
                .collect::<Vec<_>>(),
        )
        .execute(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    Ok((StatusCode::CREATED, Json(DataResponse { data: inserted })))
}

#[handler]
pub fn update(
    pool: poem::web::Data<&DbPool>,
    jwt_auth: crate::auth::middleware::JwtAuth,
    Path(mt_lang_key_id): Path<i64>,
    Json(mut entry_mt_lang_key): Json<EntryMasterLanguageKey>,
) -> poem::Result<impl IntoResponse> {
    validate_id(mt_lang_key_id)?;

    if let Err(e) = entry_mt_lang_key.validate() {
        return Err(validation_error_response(e));
    }

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let user_id = jwt_auth.claims.user_id;
    let updated = diesel::update(
        tbl_mt_lang_key::table
            .filter(tbl_mt_lang_key::id.eq(mt_lang_key_id))
            .filter(tbl_mt_lang_key::version.eq(entry_mt_lang_key.version)),
    )
    .set((
        tbl_mt_lang_key::mt_lang_type_id.eq(entry_mt_lang_key.mt_lang_type_id),
        tbl_mt_lang_key::key_cd.eq(entry_mt_lang_key.key_cd),
        tbl_mt_lang_key::version.eq(entry_mt_lang_key.version + 1),
        tbl_mt_lang_key::updated_by.eq(user_id),
        tbl_mt_lang_key::dt_updated.eq(Some(Utc::now().naive_utc())),
    ))
    .get_result::<MasterLanguageKey>(conn)
    .map_err(|e| {
        eprintln!("Updating error: {}", e);
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.internalServerError",
        )
    })?;

    diesel::update(
        tbl_mt_lang_value::table.filter(tbl_mt_lang_value::mt_lang_key_id.eq(mt_lang_key_id)),
    )
    .set(tbl_mt_lang_value::is_del.eq(1))
    .execute(conn)
    .map_err(|e| {
        eprintln!("Backupking error: {}", e);
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.internalServerError",
        )
    })?;

    diesel::insert_into(tbl_mt_lang_value::table)
        .values(
            entry_mt_lang_key
                .value
                .into_iter()
                .map(|val| MasterLanguageValue {
                    id: common::generate_id(),
                    mt_lang_id: val.mt_lang_id,
                    mt_lang_key_id: updated.id,
                    value: val.value,
                    is_del: 0,
                    created_by: user_id,
                    dt_created: Utc::now().naive_utc(),
                    updated_by: None,
                    dt_updated: None,
                    version: 0,
                })
                .collect::<Vec<MasterLanguageValue>>(),
        )
        .execute(conn)
        .map_err(|e| {
            eprintln!("Inserting error: {}", e);
            common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            )
        })?;

    let _ = diesel::delete(
        tbl_mt_lang_value::table
            .filter(tbl_mt_lang_value::mt_lang_key_id.eq(mt_lang_key_id))
            .filter(tbl_mt_lang_value::is_del.eq(1)),
    )
    .execute(conn);

    Ok(Json(DataResponse { data: updated }))
}

#[handler]
pub fn delete(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(example_template_ids): Path<String>,
) -> poem::Result<impl IntoResponse> {
    validate_ids(&example_template_ids)?;
    let ids = parse_ids_from_string(&example_template_ids)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    match diesel::delete(tbl_mt_lang_key::table.filter(tbl_mt_lang_key::id.eq_any(&ids)))
        .execute(conn)
    {
        Ok(affected_rows) => {
            if affected_rows > 0 {
                let _ = diesel::delete(
                    tbl_mt_lang_value::table.filter(tbl_mt_lang_value::mt_lang_key_id.eq_any(&ids)),
                )
                .execute(conn);
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(common::error_message(
                    StatusCode::NOT_FOUND,
                    "information.notFound",
                ))
            }
        }
        Err(e) => {
            eprintln!("Deleting error: {}", e);
            return Err(common::error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "information.internalServerError",
            ));
        }
    }
}

#[handler]
pub fn implement(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let mt_lang_key = tbl_mt_lang::table
        .select((tbl_mt_lang::id, tbl_mt_lang::cd))
        .filter(tbl_mt_lang::is_del.eq(0))
        .load::<(i16, String)>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

    for (mt_lang_id, mt_lang_cd) in mt_lang_key.iter() {
        let mt_lang_value_vec = tbl_mt_lang_value::table
            .inner_join(
                tbl_mt_lang_key::table.on(tbl_mt_lang_key::id
                    .eq(tbl_mt_lang_value::mt_lang_key_id)
                    .and(tbl_mt_lang_key::is_del.eq(0))),
            )
            .inner_join(
                tbl_mt_lang_type::table.on(tbl_mt_lang_type::id
                    .eq(tbl_mt_lang_key::mt_lang_type_id)
                    .and(tbl_mt_lang_type::is_del.eq(0))),
            )
            .filter(tbl_mt_lang_value::is_del.eq(0))
            .filter(tbl_mt_lang_value::mt_lang_id.eq(mt_lang_id))
            .select((
                tbl_mt_lang_type::cd,
                tbl_mt_lang_key::key_cd,
                tbl_mt_lang_value::value,
            ))
            .load::<(String, String, String)>(conn)
            .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "information.notFound"))?;

        let mut map = HashMap::new();
        for (mt_lang_type_cd, key_cd, value) in mt_lang_value_vec {
            map.insert(format!("{}.{}", mt_lang_type_cd, key_cd), value);
        }

        let json_str = serde_json::to_string_pretty(&map).map_err(|e| {
            eprintln!("JSON serialization error: {}", e);
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "json.serializationError")
        })?;

        let path = format!("ext/language/{}.json", mt_lang_cd);

        fs::write(&path, json_str).map_err(|e| {
            eprintln!("Failed to write file {}: {}", path, e);
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "file.writeError")
        })?;
    }

    Ok(StatusCode::NO_CONTENT)
}
