CREATE TABLE tbl_user(
	username VARCHAR (20) NOT NULL PRIMARY KEY
	, pass VARCHAR (32)
	, nick_nm VARCHAR (20)
	, full_nm VARCHAR (50)
	, email VARCHAR (30)
	, phone VARCHAR (20)
	, supervisor VARCHAR (20)
	, dt_active TIMESTAMP
	, dt_login TIMESTAMP
	, dt_logout TIMESTAMP
	, ip VARCHAR (20)
	, last_access VARCHAR (2000)
	, agent VARCHAR (300)
	, dt_resign DATE
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_user  VALUES ('myulfi', 'Password*123', 'Mul', 'Pak Mul', 'mulfiyuladi@gmail.com', '6287877636847', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'system', CURRENT_DATE, NULL, NULL, 0);