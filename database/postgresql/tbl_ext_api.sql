CREATE TABLE tbl_ext_api(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (50) NOT NULL
	, dscp VARCHAR (255)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (50)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (50)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);