CREATE TABLE tbl_mt_lang_type(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (15) NOT NULL
	, nm VARCHAR (30) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_lang_type VALUES (1, 'text', 'Text', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_type VALUES (2, 'information', 'Information', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_type VALUES (3, 'confirmation', 'Confirmation', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_type VALUES (4, 'table', 'Table', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_type VALUES (5, 'validate', 'Validate', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang_type VALUES (6, 'menu', 'Menu', 0, 'system', CURRENT_DATE, NULL, NULL, 0);