CREATE TABLE tbl_ext_database(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (20) NOT NULL
	, dscp VARCHAR (800)
	, ext_server_id SMALLINT DEFAULT(0)
	, mt_database_type_id SMALLINT NOT NULL
	, username VARCHAR (200) NOT NULL
	, password VARCHAR (200) NOT NULL
	, db_connection VARCHAR (350)
	, is_use_page SMALLINT DEFAULT(1)
	, is_lock SMALLINT DEFAULT(1)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (50)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (50)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_ext_database (id, cd, dscp, ext_server_id, mt_database_type_id, username, password, db_connection, is_use_page, is_lock, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES ((1, 'MAIN', 'Main Database', 0, 1, 'postgres', 'Password*123', 'localhost:5432/main_db', 1, 1, 0, 'myulfi', '2024-10-29 00:00:00', 'myulfi', '2024-11-14 09:18:58.528', 1));