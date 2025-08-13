use crate::facades::external::api;
use crate::facades::external::database;
use crate::facades::external::server;

use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at("/database.json", get(database::list).post(database::add))
        .at(
            "/:id/database.json",
            get(database::get)
                .patch(database::update)
                .delete(database::delete),
        )
        .at("/database-test-connection.json", get(database::manual_list))
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
