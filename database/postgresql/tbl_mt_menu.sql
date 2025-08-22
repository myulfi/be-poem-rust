CREATE TABLE tbl_mt_menu(
	id SMALLINT PRIMARY KEY
	, nm VARCHAR (30) NOT NULL
	, icon VARCHAR (20)
	, seq SMALLINT NOT NULL
	, path VARCHAR (100)
	, mt_menu_parent_id SMALLINT
	, color VARCHAR (6) DEFAULT('E7E7E7')
	, is_new SMALLINT DEFAULT(0)
	, is_blank_target SMALLINT DEFAULT(0)
	, is_del SMALLINT DEFAULT(0)
	, created_by VARCHAR (20)
	, dt_created TIMESTAMP
	, updated_by VARCHAR (20)
	, dt_updated TIMESTAMP
	, version SMALLINT DEFAULT(0)
);

INSERT INTO tbl_mt_menu VALUES (1, 'Home', 'bi-house-door', 1, '/home.html', 0, '00AA00', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (2, 'Master', 'bi-files', 10, NULL, 0, 'D43F3A', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (3, 'Zone', 'bi-pin-map', 1, '/master/zone.html', 2, NULL, 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (4, 'External', 'bi-arrow-90deg-right', 20, NULL, 0, '08B5FB', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (5, 'Server', 'bi-hdd-rack', 1, '/external/server.html', 4, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (6, 'Database', 'bi-database', 2, '/external/database.html', 4, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (7, 'API', 'bi-plugin', 3, '/external/api.html', 4, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (8, 'Command', 'bi-shield-lock', 30, NULL, 0, 'FFC40D', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (9, 'Monitoring', 'bi-laptop', 1, '/command/monitoring.html', 8, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (10, 'Access', 'bi-lock', 2, NULL, 8, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (11, 'Menu', 'bi-diagram-3', 1, '/command/menu.html', 10, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (12, 'Role', 'bi-file-ruled', 2, '/command/role.html', 10, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (13, 'User', 'bi-person', 3, '/command/user.html', 10, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (14, 'Configuration', 'bi-gear', 3, NULL, 8, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (15, 'Properties', 'bi-file-text', 1, '/command/properties.html', 14, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (16, 'Language', 'bi-translate', 2, '/command/language.html', 14, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (17, 'Procedure', 'bi-arrow-right', 3, '/command/procedure.html', 14, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (18, 'Email Scheduler', 'bi-envelope', 4, '/command/email-scheduler.html', 14, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);
INSERT INTO tbl_mt_menu VALUES (19, 'Example', 'bi-puzzle', 99, '/test/example-template.html', 0, 'E7E7E7', 0, 0, 0, 'system', CURRENT_DATE, NULL, NULL, 0);