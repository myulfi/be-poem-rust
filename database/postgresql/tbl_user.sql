CREATE TABLE tbl_user(
	id BIGINT PRIMARY KEY
	, username VARCHAR (20) NOT NULL
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
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_user VALUES (1764248315616711, 'myulfi', 'Password*123', 'Mul', 'Pak Mul', 'mulfiyuladi@gmail.com', '6287877636847', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 0, CURRENT_DATE, NULL, NULL, 0);