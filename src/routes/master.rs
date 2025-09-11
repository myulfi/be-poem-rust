use crate::facades::master;
use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at("/database-type.json", get(master::database_type))
        .at("/server-type.json", get(master::server_type))
        .at("/external-server.json", get(master::external_server))
        .at("/language.json", get(master::language))
}
