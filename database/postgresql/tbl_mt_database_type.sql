CREATE TABLE tbl_mt_database_type(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (50)
	, driver VARCHAR (50)
	, url VARCHAR (350)
	, pagination VARCHAR(50)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'Postgres', 'org.postgresql.Driver', 'postgres://{0}:{1}@{2}', '{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'MySQL', 'com.mysql.cj.jdbc.Driver', 'mysql://{0}:{1}@{2}', '{0} LIMIT {2} OFFSET {1}', 0, 'myulfi', CURRENT_DATE, NULL, NULL, 0);