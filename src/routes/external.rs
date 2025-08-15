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
