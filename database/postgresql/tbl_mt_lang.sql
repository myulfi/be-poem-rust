CREATE TABLE tbl_mt_lang(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (2) NOT NULL
	, nm VARCHAR (20) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_lang (id, cd, nm, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'en', 'English', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_lang (id, cd, nm, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'id', 'Indonesia', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);