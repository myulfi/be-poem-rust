CREATE TABLE tbl_ext_server_shortcut(
	id BIGINT PRIMARY KEY
	, nm VARCHAR (100) NOT NULL
	, ext_server_id BIGINT NOT NULL
	, dir VARCHAR (1000) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);