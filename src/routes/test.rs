use crate::facades::test::example_template;
use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at(
            "/example-template.json",
            get(example_template::list).post(example_template::add),
        )
        .at(
            "/:id/example-template.json",
            get(example_template::get)
                .patch(example_template::update)
                .delete(example_template::delete),
        )
}
