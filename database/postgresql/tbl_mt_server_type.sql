CREATE TABLE tbl_mt_server_type(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (15)
	, icon VARCHAR (20)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_server_type (id, nm, icon, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'Linux Kernel', 'fa-solid fa-ubuntu', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_server_type (id, nm, icon, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'Windows', 'fa-solid fa-windows', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_server_type (id, nm, icon, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (3, 'macOS', 'fa-solid fa-apple', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);