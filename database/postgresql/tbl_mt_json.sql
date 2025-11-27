CREATE TABLE tbl_mt_json(
	id BIGINT PRIMARY KEY
	, nm VARCHAR (50) NOT NULL
	, mt_http_method_id SMALLINT
	, notation VARCHAR (200)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);