CREATE TABLE tbl_query_manual(
	id BIGINT PRIMARY KEY
	, ext_database_id SMALLINT NOT NULL
	, query TEXT NOT NULL
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);