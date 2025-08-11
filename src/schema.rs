use diesel::prelude::*;

table! {
    tbl_example_template (id) {
        id -> BigInt,
        nm -> Nullable<Varchar>,
        dscp -> Nullable<Varchar>,
        val -> Nullable<SmallInt>,
        amt -> Nullable<Numeric>,
        dt -> Nullable<Date>,
        foreign_id -> Nullable<BigInt>,
        is_active -> SmallInt,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}

table! {
    tbl_user (username) {
        username -> Varchar,
        pass -> Nullable<Varchar>,
        nick_nm -> Nullable<Varchar>,
        full_nm -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        supervisor -> Nullable<Varchar>,
        dt_active -> Nullable<Timestamp>,
        dt_login -> Nullable<Timestamp>,
        dt_logout -> Nullable<Timestamp>,
        ip -> Nullable<Varchar>,
        last_access -> Nullable<Varchar>,
        agent -> Nullable<Varchar>,
        dt_resign -> Nullable<Date>,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}

table! {
    tbl_user_role (id) {
        id -> BigInt,
        username -> Varchar,
        mt_role_id -> SmallInt,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}

table! {
    tbl_mt_menu (id) {
        id -> SmallInt,
        nm -> Varchar,
        icon -> Nullable<Varchar>,
        seq -> SmallInt,
        path -> Nullable<Varchar>,
        mt_menu_parent_id -> SmallInt,
        color -> Nullable<Varchar>,
        is_new -> SmallInt,
        is_blank_target -> SmallInt,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt
    }
}

table! {
    tbl_ext_database (id) {
        id -> SmallInt,
        cd -> Varchar,
        dscp -> Nullable<Varchar>,
        mt_database_type_id -> SmallInt,
        username -> Varchar,
        password -> Varchar,
        db_connection -> Varchar,
        is_lock -> SmallInt,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}

table! {
    tbl_ext_server (id) {
        id -> SmallInt,
        cd -> Varchar,
        dscp -> Nullable<Varchar>,
        ip -> Varchar,
        port -> SmallInt,
        username -> Varchar,
        password -> Nullable<Varchar>,
        private_key -> Nullable<Varchar>,
        is_lock -> SmallInt,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}

table! {
    tbl_ext_api (id) {
        id -> SmallInt,
        nm -> Varchar,
        dscp -> Nullable<Varchar>,
        is_del -> SmallInt,
        created_by -> Varchar,
        dt_created -> Timestamp,
        updated_by -> Nullable<Varchar>,
        dt_updated -> Nullable<Timestamp>,
        version -> SmallInt,
    }
}
