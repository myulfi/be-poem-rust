CREATE TABLE tbl_mt_lang_key(
	id BIGINT PRIMARY KEY
	, screen_cd VARCHAR (30) NOT NULL
	, label_typ VARCHAR (30) NOT NULL
	, key_cd VARCHAR (40) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);