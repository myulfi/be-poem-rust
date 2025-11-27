CREATE TABLE tbl_ext_api_req(
	id BIGINT PRIMARY KEY
	, seq SMALLINT
	, nm VARCHAR (50) NOT NULL
	, ext_api_id BIGINT NOT NULL
	, parent_id BIGINT
	, mt_http_method_id SMALLINT DEFAULT(0)
	, path VARCHAR (1000)
	, is_have_authz SMALLINT DEFAULT(0)
	, body VARCHAR (1000)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);