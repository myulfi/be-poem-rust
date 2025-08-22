CREATE TABLE tbl_mt_lang_value(
	id BIGINT PRIMARY KEY
	, mt_lang_id SMALLINT NOT NULL
	, mt_lang_key_id BIGINT NOT NULL
	, value VARCHAR (150) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);


INSERT INTO tbl_mt_lang_key VALUES (func_generate_id(), 'common', 'text', 'farmer', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_value VALUES
(func_generate_id(), 1, (SELECT id FROM tbl_mt_lang_key WHERE screen_cd||'.'||label_typ||'.'||key_cd = 'common.text.farmer' ), 'Farmer', 0, 'system', CURRENT_DATE, NULL, NULL, 0),
(func_generate_id(), 2, (SELECT id FROM tbl_mt_lang_key WHERE screen_cd||'.'||label_typ||'.'||key_cd = 'common.text.farmer' ), 'Petani', 0, 'system', CURRENT_DATE, NULL, NULL, 0);