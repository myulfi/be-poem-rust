CREATE TABLE tbl_mt_http_method(
	id SMALLINT PRIMARY KEY
	, cd VARCHAR (7) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_http_method (id, cd, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (1, 'GET', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_http_method (id, cd, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (2, 'POST', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_http_method (id, cd, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (3, 'PUT', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_http_method (id, cd, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (4, 'PATCH', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_http_method (id, cd, is_del, created_by, dt_created, updated_by, dt_updated, version) VALUES (5, 'DELETE', 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);