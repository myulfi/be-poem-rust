CREATE TABLE tbl_mt_lang_key(
	id BIGINT PRIMARY KEY
	, mt_lang_type_id SMALLINT NOT NULL
	, key_cd VARCHAR (40) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);