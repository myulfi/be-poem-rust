CREATE TABLE tbl_ext_api_var(
	id BIGINT PRIMARY KEY
	, seq SMALLINT
	, ext_api_id BIGINT NOT NULL
	, key VARCHAR (50) NOT NULL
	, val VARCHAR (1000)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);