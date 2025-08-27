use crate::facades::external::api;
use crate::facades::external::database;
use crate::facades::external::database_query;
use crate::facades::external::server;
use crate::facades::external::server_command;

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
        .at("/:id/database-connect.json", get(database_query::connect))
        .at(
            "/:id/database-query-object-list.json",
            get(database_query::query_object_list),
        )
        .at(
            "/:id/database-query-whitelist-list.json",
            get(database_query::query_whitelist_list),
        )
        .at(
            "/:id/database-query-manual-run.json",
            post(database_query::query_manual_run),
        )
        .at(
            "/:id/database-query-manual-list.json",
            get(database_query::query_manual_list),
        )
        .at(
            "/:id/database-query-manual-all-list.json",
            get(database_query::query_manual_all_list),
        )
        .at(
            "/:id/:include_column_name_flag/:number_line_per_action/database-query-manual-sql-insert.json",
            get(database_query::query_manual_sql_insert),
        )
        .at(
            "/:id/:multiple_line_flag/:first_amount_conditioned/database-query-manual-sql-update.json",
            get(database_query::query_manual_sql_update),
        )
        .at(
            "/:id/:first_amount_combined/database-query-manual.xlsx",
            get(database_query::query_manual_xlsx),
        )
        .at(
            "/:id/:header_flag/:delimiter/database-query-manual-csv.json",
            get(database_query::query_manual_csv),
        )
        .at(
            "/:id/database-query-manual.json",
            get(database_query::query_manual_json),
        )
        .at(
            "/:id/database-query-manual-xml.json",
            get(database_query::query_manual_xml),
        )
        .at(
            "/:id/:name/database-query-exact-object-run.json",
            post(database_query::query_exact_object_run),
        )
        .at(
            "/:id/:name/database-query-exact-object-list.json",
            get(database_query::query_exact_object_list),
        )
        .at(
            "/:id/database-query-exact-whitelist-run.json",
            post(database_query::query_exact_whitelist_run),
        )
        .at(
            "/:id/database-query-exact-whitelist-list.json",
            get(database_query::query_exact_whitelist_list),
        )
        .at("/server.json", get(server::list).post(server::add))
        .at(
            "/:id/server.json",
            get(server::get)
                .patch(server::update)
                .delete(server::delete),
        )
        .at("/:id/server-connect.json", get(server_command::connect))
        .at("/api.json", get(api::list).post(api::add))
        .at(
            "/:id/api.json",
            get(api::get).patch(api::update).delete(api::delete),
        )
}
