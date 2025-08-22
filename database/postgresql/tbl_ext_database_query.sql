CREATE TABLE tbl_ext_database_query(
	id BIGINT PRIMARY KEY
	, dscp VARCHAR (800)
	, ext_database_id SMALLINT NOT NULL
	, query TEXT NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (50)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (50)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);