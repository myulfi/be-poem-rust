CREATE TABLE tbl_user_branch(
	id BIGINT PRIMARY KEY
	, user_id BIGINT NOT NULL
	, mt_branch_id SMALLINT
	, is_del SMALLINT DEFAULT(0)
	, created_by BIGINT NOT NULL
	, dt_created TIMESTAMP NOT NULL
	, updated_by BIGINT
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_user_branch VALUES (func_generate_id(), 1764248315616711, 1, 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_user_branch VALUES (func_generate_id(), 1764248315616711, 2, 0, 1764248315616711, CURRENT_DATE, NULL, NULL, 0);