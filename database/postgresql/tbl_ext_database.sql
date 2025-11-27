CREATE TABLE tbl_ext_database(
	id BIGINT PRIMARY KEY
	, cd VARCHAR (20) NOT NULL
	, dscp VARCHAR (800)
	, ext_server_id BIGINT
	, mt_database_type_id SMALLINT NOT NULL
	, ip VARCHAR (15) NOT NULL
	, port SMALLINT DEFAULT(22)
	, username VARCHAR (200) NOT NULL
	, password VARCHAR (200) NOT NULL
	, db_name VARCHAR (50)
	, username VARCHAR (200) NOT NULL
	, db_connection VARCHAR (350)
	, is_use_page SMALLINT DEFAULT(1)
	, is_lock SMALLINT DEFAULT(1)
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_ext_database (id, cd, dscp, ext_server_id, mt_database_type_id, ip, port, username, password, db_name, db_connection, is_use_page, is_lock, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1757330524398919, 'MAIN', 'Main Database', NULL, 1, 'localhost', 5432, 'postgres', 'Password*123', 'main_db', 'localhost:5432/main_db', 1, 1, 0, 1764248315616711, '2024-10-29 00:00:00', 1764248315616711, '2024-11-14 09:18:58.528', 1);