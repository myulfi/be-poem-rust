CREATE TABLE tbl_mt_role_menu(
	id BIGINT PRIMARY KEY
	, mt_role_id SMALLINT NOT NULL
	, mt_menu_id SMALLINT NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);