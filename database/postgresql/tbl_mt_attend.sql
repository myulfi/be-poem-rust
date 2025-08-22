CREATE TABLE tbl_mt_attend(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (30) NOT NULL
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_attend VALUES (1, 'Location & Face Recognition', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_attend VALUES (2, 'Location & QR Code', 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_attend VALUES (3, 'Location', 0, 'system', CURRENT_DATE, NULL, NULL, 0);