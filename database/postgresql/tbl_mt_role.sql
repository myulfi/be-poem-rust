CREATE TABLE tbl_mt_role(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (20) NOT NULL
	, dscp VARCHAR (100)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_role VALUES (0, 'System Admin', NULL, 0, 'system', CURRENT_DATE, NULL, NULL, 0);