use crate::facades::command::language;
use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at("/language.json", get(language::list).post(language::add))
        .at(
            "/:id/language.json",
            get(language::get)
                .patch(language::update)
                .delete(language::delete),
        )
}
