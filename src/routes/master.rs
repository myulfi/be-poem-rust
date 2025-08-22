use crate::facades::master;
use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at("/database-type.json", get(master::database_type))
        .at("/external-server.json", get(master::external_server))
}
