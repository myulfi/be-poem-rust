use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::models::external::api::{ExternalApiRequest, ExternalApiRequestNode};
use crate::schema::tbl_ext_api_req;
use crate::utils::common::{self, validate_id};
use diesel::prelude::*;
use poem::IntoResponse;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Path},
};

fn build_request_tree(data: Vec<ExternalApiRequest>) -> Vec<ExternalApiRequestNode> {
    use std::collections::HashMap;

    let mut map: HashMap<i64, Vec<ExternalApiRequest>> = HashMap::new();

    // Grouping berdasarkan parent
    for item in data.into_iter() {
        map.entry(item.parent_id).or_default().push(item);
    }

    // Fungsi recursive untuk membangun tree
    fn build_nodes(
        parent_id: i64,
        map: &mut HashMap<i64, Vec<ExternalApiRequest>>,
    ) -> Vec<ExternalApiRequestNode> {
        if let Some(children) = map.remove(&parent_id) {
            children
                .into_iter()
                .map(|item| ExternalApiRequestNode {
                    id: item.id,
                    seq: item.seq,
                    nm: item.nm,
                    ext_api_id: item.ext_api_id,
                    parent_id: item.parent_id,
                    mt_http_method_id: item.mt_http_method_id,
                    path: item.path,
                    is_have_authz: item.is_have_authz,
                    body: item.body,
                    is_del: item.is_del,
                    created_by: item.created_by,
                    dt_created: item.dt_created,
                    updated_by: item.updated_by,
                    dt_updated: item.dt_updated,
                    version: item.version,
                    children: build_nodes(item.id, map),
                })
                .collect()
        } else {
            vec![]
        }
    }

    build_nodes(0 as i64, &mut map)
}

#[handler]
pub fn get(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
    Path(ext_api_id): Path<i64>,
) -> poem::Result<impl IntoResponse> {
    validate_id(ext_api_id)?;

    let conn = &mut pool.get().map_err(|_| {
        common::error_message(
            StatusCode::INTERNAL_SERVER_ERROR,
            "information.connectionFailed",
        )
    })?;

    let ext_api = tbl_ext_api_req::table
        .filter(tbl_ext_api_req::ext_api_id.eq(ext_api_id))
        .order(tbl_ext_api_req::seq.asc())
        .load::<ExternalApiRequest>(conn)
        .map_err(|_| common::error_message(StatusCode::NOT_FOUND, "Failed to load data"))?;

    let tree = build_request_tree(ext_api);
    Ok(Json(DataResponse { data: tree }))
}
