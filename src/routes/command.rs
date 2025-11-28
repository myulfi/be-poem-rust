use crate::facades::command::language;
use crate::facades::command::role;
use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/role.json", get(role::list).post(role::add))
        .at(
            "/:id/role.json",
            get(role::get).put(role::update).delete(role::delete),
        )
        .at(
            "/:id/role-menu.json",
            get(role::menu_list).post(role::menu_update),
        )
        .at("/language.json", get(language::list).post(language::add))
        .at(
            "/:id/language.json",
            get(language::get)
                .put(language::update)
                .delete(language::delete),
        )
        .at("/language-implement.json", post(language::implement))
}
