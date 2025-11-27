CREATE TABLE tbl_ext_database_query(
	id BIGINT PRIMARY KEY
	, dscp VARCHAR (800) NOT NULL
	, ext_database_id BIGINT NOT NULL
	, query TEXT NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);