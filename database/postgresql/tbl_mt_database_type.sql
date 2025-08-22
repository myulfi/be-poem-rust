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

INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'Postgres', 'org.postgresql.Driver', 'postgresql://%3$s?user=%1$s&password=%2$s', '{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY', 0, 'system', CURRENT_DATE, NULL, NULL, 0)
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'Oracle', 'oracle.jdbc.driver.OracleDriver', 'jdbc:oracle:thin:%1$s/%2$s@%3$s', '{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY', 0, 'system', '2018-04-11 00:00:00.0', NULL, NULL, 0);
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'Postgres', 'org.postgresql.Driver', 'jdbc:postgresql://%3$s?user=%1$s&password=%2$s', '{0} OFFSET {1} ROWS FETCH NEXT {2} ROWS ONLY', 0, 'system', '2018-04-11 00:00:00.0', NULL, NULL, 0);
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (3, 'SQL Server', 'com.microsoft.sqlserver.jdbc.SQLServerDriver', 'jdbc:sqlserver://%3$s;user=%1$s;password=%2$s', NULL, 0, 'myulfi', '2022-01-27 07:33:09.444459', NULL, NULL, 0);
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (4, 'MySQL', 'com.mysql.cj.jdbc.Driver', 'jdbc:mysql://%3$s?user=%1$s&password=%2$s', '{0} LIMIT {2} OFFSET {1}', 0, 'myulfi', '2022-01-27 07:33:09.444459', NULL, NULL, 0);
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (5, 'MariaDB', 'org.mariadb.jdbc.Driver', 'jdbc:mariadb://%3$s?user=%1$s&password=%2$s', NULL, 0, 'myulfi', '2022-01-27 07:33:09.444459', NULL, NULL, 0);
-- INSERT INTO tbl_mt_database_type (id, nm, driver, url, pagination, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (6, 'Cloudera', 'com.cloudera.impala.jdbc4.Driver', 'jdbc:hive2:%3$s/default;user=%1$s;password=%2$s', NULL, 0, 'myulfi', '2022-04-01 09:48:41.612014', NULL, NULL, 0);