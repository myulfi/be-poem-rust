use crate::db::DbPool;
use crate::models::common::DataResponse;
use crate::models::master_menu::{MasterMenu, MenuNode};
use crate::schema::tbl_mt_menu::dsl::*;
use crate::utils::common::{self};
use diesel::prelude::*;
use poem::IntoResponse;
use poem::{handler, http::StatusCode, web::Json};

fn build_menu_tree(data: Vec<MasterMenu>) -> Vec<MenuNode> {
    use std::collections::HashMap;

    let mut map: HashMap<i16, Vec<MasterMenu>> = HashMap::new();

    // Grouping berdasarkan parent
    for item in data.into_iter() {
        map.entry(item.mt_menu_parent_id).or_default().push(item);
    }

    // Fungsi recursive untuk membangun tree
    fn build_nodes(parent_id: i16, map: &mut HashMap<i16, Vec<MasterMenu>>) -> Vec<MenuNode> {
        if let Some(children) = map.remove(&parent_id) {
            children
                .into_iter()
                .map(|item| MenuNode {
                    id: item.id,
                    nm: item.nm,
                    icon: item.icon,
                    seq: item.seq,
                    path: item.path,
                    mt_menu_parent_id: item.mt_menu_parent_id,
                    color: item.color,
                    is_new: item.is_new,
                    is_blank_target: item.is_blank_target,
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

    build_nodes(0, &mut map) // Root: mt_menu_parent_id == 0
}

#[handler]
pub fn menu_list(
    pool: poem::web::Data<&DbPool>,
    _: crate::auth::middleware::JwtAuth,
) -> poem::Result<impl IntoResponse> {
    let conn = &mut pool.get().map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Connection failed")
    })?;

    let data = tbl_mt_menu
        .filter(is_del.eq(0))
        .order(seq.asc())
        .load::<MasterMenu>(conn)
        .map_err(|_| {
            common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load data")
        })?;

    let tree = build_menu_tree(data);
    Ok(Json(DataResponse { data: tree }))
}
