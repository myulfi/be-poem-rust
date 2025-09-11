use crate::facades::main;
use poem::{Route, get};

pub fn routes() -> Route {
    Route::new().nest("/menu.json", get(main::menu_list))
}
