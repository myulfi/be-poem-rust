CREATE TABLE tbl_mt_json(
	id BIGINT PRIMARY KEY
	, nm VARCHAR (50) NOT NULL
	, mt_http_method_id SMALLINT
	, notation VARCHAR (200)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);