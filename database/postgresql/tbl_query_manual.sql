CREATE TABLE tbl_query_manual(
	id BIGINT PRIMARY KEY
	, ext_database_id SMALLINT NOT NULL
	, query TEXT NOT NULL
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);