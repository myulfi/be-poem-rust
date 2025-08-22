CREATE TABLE tbl_mt_lang(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (2)
	, nm VARCHAR (20)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_lang (id, cd, nm, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'en', 'English', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang (id, cd, nm, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'id', 'Indonesia', 0, 'system', CURRENT_DATE, NULL, NULL, 0);