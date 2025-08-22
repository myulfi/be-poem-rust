CREATE TABLE tbl_example_template (
	id BIGINT PRIMARY KEY
	, nm VARCHAR (100)
	, dscp VARCHAR (250)
	, val SMALLINT
	, amt NUMERIC (15, 2)
	, dt DATE
	, foreign_id BIGINT
	, is_active SMALLINT
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);