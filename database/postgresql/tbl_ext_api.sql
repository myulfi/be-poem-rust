CREATE TABLE tbl_ext_api(
	id BIGINT PRIMARY KEY
	, nm VARCHAR (50) NOT NULL
	, dscp VARCHAR (255)
	, authz VARCHAR (500)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);