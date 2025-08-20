use crate::facades::external::api;
use crate::facades::external::database;
use crate::facades::external::server;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/database.json", get(database::list).post(database::add))
        .at(
            "/:id/database.json",
            get(database::get)
                .patch(database::update)
                .delete(database::delete),
        )
        .at("/:id/database-connect.json", get(database::connect))
        .at(
            "/:id/database-query-object-list.json",
            get(database::query_object_list),
        )
        .at(
            "/:id/database-query-whitelist-list.json",
            get(database::query_whitelist_list),
        )
        .at(
            "/:id/database-query-manual-run.json",
            post(database::query_manual_run),
        )
        .at(
            "/:id/database-query-manual-list.json",
            get(database::query_manual_list),
        )
        .at(
            "/:id/database-query-manual-all-list.json",
            get(database::query_manual_all_list),
        )
        .at(
            "/:id/:includeColumnNameFlag/:numberLinePerAction/database-query-manual-sql-insert.json",
            get(database::query_manual_sql_insert),
        )
        .at(
            "/:id/:name/database-query-exact-object-run.json",
            post(database::query_exact_object_run),
        )
        .at(
            "/:id/:name/database-query-exact-object-list.json",
            get(database::query_exact_object_list),
        )
        .at(
            "/:id/database-query-exact-whitelist-run.json",
            post(database::query_exact_whitelist_run),
        )
        .at(
            "/:id/database-query-exact-whitelist-list.json",
            get(database::query_exact_whitelist_list),
        )
        .at("/server.json", get(server::list).post(server::add))
        .at(
            "/:id/server.json",
            get(server::get)
                .patch(server::update)
                .delete(server::delete),
        )
        .at("/api.json", get(api::list).post(api::add))
        .at(
            "/:id/api.json",
            get(api::get).patch(api::update).delete(api::delete),
        )
}
