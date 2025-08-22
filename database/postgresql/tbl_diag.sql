CREATE TABLE tbl_diag_ad_rev(
	ad INT
	, rev INT
);
INSERT INTO tbl_diag_ad_rev VALUES(5000000,20000000);
INSERT INTO tbl_diag_ad_rev VALUES(6000000,22000000);
INSERT INTO tbl_diag_ad_rev VALUES(7000000,25000000);
INSERT INTO tbl_diag_ad_rev VALUES(8000000,30000000);
INSERT INTO tbl_diag_ad_rev VALUES(9000000,32000000);
INSERT INTO tbl_diag_ad_rev VALUES(10000000,35000000);
INSERT INTO tbl_diag_ad_rev VALUES(11000000,38000000);
INSERT INTO tbl_diag_ad_rev VALUES(12000000,40000000);
INSERT INTO tbl_diag_ad_rev VALUES(13000000,45000000);
INSERT INTO tbl_diag_ad_rev VALUES(15000000,50000000);

CREATE TABLE tbl_diag_productivity(
	productivity INT
	, quality NUMERIC(3,2)
	, member SMALLINT
);
INSERT INTO tbl_diag_productivity VALUES(120,8.5,5);
INSERT INTO tbl_diag_productivity VALUES(150,7.8,7);
INSERT INTO tbl_diag_productivity VALUES(100,9,4);
INSERT INTO tbl_diag_productivity VALUES(180,7.2,8);
INSERT INTO tbl_diag_productivity VALUES(140,8.2,6);
INSERT INTO tbl_diag_productivity VALUES(200,6.5,10);

CREATE TABLE tbl_diag_budget(
	depart VARCHAR(15)
	, budget INT
);
INSERT INTO tbl_diag_budget VALUES('Marketing',50);
INSERT INTO tbl_diag_budget VALUES('Operations',40);
INSERT INTO tbl_diag_budget VALUES('RnD',30);
INSERT INTO tbl_diag_budget VALUES('HR',20);
INSERT INTO tbl_diag_budget VALUES('IT',30);
INSERT INTO tbl_diag_budget VALUES('Miscellaneous',30);

CREATE TABLE tbl_diag_performance(
	depart VARCHAR(15)
	, efficiency SMALLINT
	, cost SMALLINT
	, innovation SMALLINT
	, satisfaction SMALLINT
	, cs SMALLINT
);
INSERT INTO tbl_diag_performance VALUES('Marketing',8,6,7,9,8);
INSERT INTO tbl_diag_performance VALUES('Operations',9,8,6,7,7);
INSERT INTO tbl_diag_performance VALUES('RnD',7,5,9,8,6);
INSERT INTO tbl_diag_performance VALUES('HR',6,7,5,9,8);
INSERT INTO tbl_diag_performance VALUES('IT',8,7,8,7,9);

CREATE TABLE tbl_diag_task(
	dt VARCHAR(15)
	, nm VARCHAR(20)
);
INSERT INTO tbl_diag_task VALUES('2024-05-01','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-01','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-01','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-02','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-02','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-02','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-03','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-03','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-03','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-03','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-04','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-05','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-05','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-05','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-05','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-06','Bambang');
INSERT INTO tbl_diag_task VALUES('2024-05-06','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-06','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-06','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-07','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-08','Budi');
INSERT INTO tbl_diag_task VALUES('2024-05-08','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-08','Mulyadi');
INSERT INTO tbl_diag_task VALUES('2024-05-08','Asep');
INSERT INTO tbl_diag_task VALUES('2024-05-09','Bambang');