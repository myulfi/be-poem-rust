CREATE TABLE tbl_ext_server(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (20) NOT NULL
	, dscp VARCHAR (500)
	, ip VARCHAR (15) NOT NULL
	, port SMALLINT DEFAULT(22)
	, username VARCHAR (200) NOT NULL
	, password VARCHAR (200)
	, private_key TEXT
	, is_lock SMALLINT DEFAULT(1)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);