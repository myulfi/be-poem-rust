CREATE TABLE tbl_user_branch(
	id BIGINT PRIMARY KEY
	, username VARCHAR (20)
	, mt_branch_id SMALLINT
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_user_branch VALUES (func_generate_id(), 'myulfi', 1, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_user_branch VALUES (func_generate_id(), 'myulfi', 2, 0, 'system', CURRENT_DATE, NULL, NULL, 0);