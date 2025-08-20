use rand::Rng;
use regex::Regex;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_postgres::Row;
use validator::ValidationErrors;

use crate::utils::common;
use poem::{IntoResponse, Response, Result, error::Error, http::StatusCode, web::Json};
use serde::Serialize;
use tokio_postgres::types::Oid;

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

pub fn error_message(status: StatusCode, msg: &str) -> Error {
    Error::from_response(
        (
            status,
            Json(MessageResponse {
                message: msg.to_string(),
            }),
        )
            .into_response(),
    )
}

// pub fn error_message(status: StatusCode, message: impl Into<String>) -> Response {
//     let msg = message.into();
//     (status, msg).into_response()
// }

pub fn generate_id() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_millis(0))
        .as_millis();

    let random = rand::thread_rng().gen_range(0..=999) as u128;
    let id = now * 1000 + random;
    let max_safe: u128 = i64::MAX as u128;
    let safe_id = if id > max_safe { max_safe } else { id };
    safe_id as i64
}

pub fn validate_id(id: i64) -> Result<()> {
    if id >= 1_000_000_000_000_000 && id <= 9_999_999_999_999_999 {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND.into())
    }
}

pub fn validate_ids(ids: &str) -> Result<()> {
    if ids.len() % 16 == 0 && ids.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND.into())
    }
}

pub fn parse_ids_from_string(input: &str) -> Result<Vec<i64>> {
    let mut ids = Vec::new();

    for chunk in input.as_bytes().chunks(16) {
        let id_str = std::str::from_utf8(chunk)
            .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "Invalid UTF-8 ID"))?;

        let id = id_str
            .parse::<i64>()
            .map_err(|_| common::error_message(StatusCode::BAD_REQUEST, "Invalid ID number"))?;

        ids.push(id);
    }

    Ok(ids)
}

pub fn validation_error_response(e: ValidationErrors) -> poem::Error {
    let mut details = serde_json::Map::new();

    for (field, errors) in e.field_errors().iter() {
        let messages: Vec<String> = errors
            .iter()
            .filter_map(|err| err.message.as_ref())
            .map(|msg| msg.to_string())
            .collect();

        details.insert(field.to_string(), json!(messages));
    }

    let body = json!({
        "error": "Validation failed",
        "details": details
    });

    poem::Error::from_response(
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .content_type("application/json")
            .body(body.to_string()),
    )
}

// pub fn convert_select_to_count(query: &str) -> Option<String> {
//     let query_lower = query.to_lowercase();

//     // Cari posisi "from"
//     let from_pos = query_lower.find("from")?;

//     // Split dari FROM ke akhir
//     let (_, after_select) = query.split_at(from_pos);

//     // Deteksi kalau ada ORDER, LIMIT, OFFSET dan buang
//     let mut cleaned = after_select.trim().to_string();

//     for clause in ["order by", "limit", "offset"] {
//         if let Some(pos) = cleaned.to_lowercase().find(clause) {
//             cleaned = cleaned[..pos].trim().to_string();
//         }
//     }

//     // Gabungkan ulang menjadi SELECT count(*) FROM ...
//     let count_query = format!("SELECT count(*) {}", cleaned);
//     Some(count_query)
// }

pub fn convert_to_count_query(raw_query: &str) -> Option<String> {
    let lower = raw_query.to_lowercase();

    // Cari posisi kata "from" pertama
    let from_pos = lower.find(" from ")?; // spasi penting agar tidak salah match

    // Ambil bagian FROM ke akhir
    let from_clause = &raw_query[from_pos..];

    // Buang ORDER BY, LIMIT, OFFSET jika ada
    let clauses_to_remove = [" order by ", " limit ", " offset ", " fetch "];
    let mut clean_clause = from_clause.to_string();
    for clause in clauses_to_remove {
        if let Some(pos) = clean_clause.to_lowercase().find(clause) {
            clean_clause = clean_clause[..pos].trim_end().to_string();
        }
    }

    // Susun ulang
    let result = format!("SELECT COUNT(*){}", clean_clause);
    Some(result)
}

pub fn rows_to_json(rows: &[Row]) -> Vec<Value> {
    let mut results = Vec::new();

    for row in rows {
        let mut map = Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();

            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    match v {
                        Some(oid) => Value::Number((oid as i64).into()),
                        None => Value::Null,
                    }
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    Value::Number((v as i64).into())
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    Value::Number((v as i64).into())
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    match v {
                        Some(val) => Value::Number(val.into()),
                        None => Value::Null,
                    }
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }
                "numeric" => {
                    let v: Option<rust_decimal::Decimal> = row.get(i);
                    match v {
                        Some(val) => Value::String(val.to_string()),
                        None => Value::Null,
                    }
                }
                "bool" => {
                    let v: bool = row.get(i);
                    Value::Bool(v)
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    match val {
                        Some(s) => Value::String(s),
                        None => Value::Null,
                    }
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    match v {
                        Some(s) => Value::String(s),
                        None => Value::Null,
                    }
                }
            };

            map.insert(name.to_string(), value);
        }

        results.push(Value::Object(map));
    }

    results
}

pub fn rows_to_insert_query_string(
    table_name: &str,
    include_column_name_flag: i16,
    number_line_per_action: i16,
    rows: &[Row],
) -> String {
    let mut result = String::new();
    let mut batch = Vec::new();

    for (row_index, row) in rows.iter().enumerate() {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name().to_string();

            let value = match column.type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    v.to_string()
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    v.to_string()
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map(|val| val.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| format!("'{}'", val.to_string()))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "bool" => {
                    let v: bool = row.get(i);
                    v.to_string()
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.map(|s| format!("'{}'", s))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.map(|s| format!("'{}'", s.replace('\'', "''")))
                        .unwrap_or_else(|| "NULL".to_string())
                }
            };

            columns.push(name);
            values.push(value);
        }

        // Tambahkan values ke dalam batch
        batch.push(values.join(", "));

        let is_last = row_index == rows.len() - 1;
        let batch_size = number_line_per_action.max(1) as usize;

        if batch.len() == batch_size || is_last {
            if include_column_name_flag == 1 {
                let _ = write!(
                    &mut result,
                    "INSERT INTO {} ({}) VALUES ({});\n",
                    table_name,
                    columns.join(", "),
                    batch
                        .iter()
                        .map(|v| format!("({})", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            } else {
                let _ = write!(
                    &mut result,
                    "INSERT INTO {} VALUES ({});\n",
                    table_name,
                    batch
                        .iter()
                        .map(|v| format!("({})", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            batch.clear();
        }
    }

    result
}

pub fn rows_to_update_query_string(
    table_name: &str,
    multiple_line_flag: i16,
    first_amount_conditioned: i16,
    rows: &[Row],
) -> String {
    let mut result = String::new();

    for row in rows {
        let mut set_clauses = Vec::new();
        let mut where_clauses = Vec::new();
        let mut column_names = Vec::new();

        for (i, column) in row.columns().iter().enumerate() {
            column_names.push(column.name().to_string());
        }

        for (i, name) in column_names.iter().enumerate() {
            let value = match row.columns()[i].type_().name() {
                "oid" => {
                    let v: Option<Oid> = row.try_get(i).ok();
                    v.map(|oid| oid.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "int2" => {
                    let v: i16 = row.get(i);
                    v.to_string()
                }
                "int4" => {
                    let v: i32 = row.get(i);
                    v.to_string()
                }
                "int8" => {
                    let v: Option<i64> = row.get(i);
                    v.map(|val| val.to_string())
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "float4" | "float8" => {
                    let v: f64 = row.get(i);
                    if v.is_finite() {
                        v.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                "numeric" => {
                    let v: Option<Decimal> = row.get(i);
                    v.map(|val| format!("'{}'", val.to_string()))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                "bool" => {
                    let v: bool = row.get(i);
                    v.to_string()
                }
                "date" | "timestamp" => {
                    let val: Option<String> = row.try_get(i).ok();
                    val.map(|s| format!("'{}'", s))
                        .unwrap_or_else(|| "NULL".to_string())
                }
                _ => {
                    let v: Option<String> = row.get(i);
                    v.map(|s| format!("'{}'", s.replace('\'', "''")))
                        .unwrap_or_else(|| "NULL".to_string())
                }
            };

            if (i as i16) < first_amount_conditioned {
                where_clauses.push(format!("{} = {}", name, value));
            } else {
                set_clauses.push(format!("{} = {}", name, value));
            }
        }

        let mut update_sql = String::new();

        if multiple_line_flag == 1 {
            let _ = writeln!(&mut update_sql, "UPDATE {}", table_name);
            if !set_clauses.is_empty() {
                let _ = writeln!(
                    &mut update_sql,
                    "SET {}",
                    set_clauses
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if i == 0 {
                                format!("{}", v)
                            } else {
                                format!(", {}", v)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
            if !where_clauses.is_empty() {
                let _ = writeln!(&mut update_sql, "WHERE {};", where_clauses.join(" AND "));
            } else {
                update_sql.push_str(";\n");
            }

            update_sql.push('\n'); // <- Tambahkan newline antar statement
        } else {
            let _ = write!(
                &mut update_sql,
                "UPDATE {} SET {}",
                table_name,
                set_clauses.join(", ")
            );
            if !where_clauses.is_empty() {
                let _ = write!(&mut update_sql, " WHERE {};", where_clauses.join(" AND "));
            }
            update_sql.push('\n');
        }

        result.push_str(&update_sql);
    }

    result
}

pub fn extract_columns_info(rows: &[Row]) -> Vec<Value> {
    let mut columns_info = Vec::new();

    if let Some(row) = rows.get(0) {
        for column in row.columns() {
            let mut map = Map::new();
            map.insert("name".to_string(), Value::String(column.name().to_string()));
            map.insert(
                "type".to_string(),
                Value::String(column.type_().name().to_string()),
            );
            columns_info.push(Value::Object(map));
        }
    }

    columns_info
}

// fn extract_columns_info(rows: &[Row]) -> Option<Vec<Value>> {
//     rows.get(0).map(|row| {
//         row.columns()
//             .iter()
//             .map(|column| {
//                 let mut map = Map::new();
//                 map.insert("name".to_string(), Value::String(column.name().to_string()));
//                 map.insert(
//                     "type".to_string(),
//                     Value::String(column.type_().name().to_string()),
//                 );
//                 Value::Object(map)
//             })
//             .collect()
//     })
// }

pub fn split_manual_query(input: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut current = String::new();

    enum State {
        Normal,
        SingleQuote,
        DoubleQuote,
        LineComment,
        BlockComment,
        BeginEndBlock,
    }

    use State::*;

    let mut state = Normal;
    let mut begin_end_level = 0;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match state {
            Normal => match c {
                '\'' => {
                    state = SingleQuote;
                    current.push(c);
                }
                '"' => {
                    state = DoubleQuote;
                    current.push(c);
                }
                '-' => {
                    if chars.peek() == Some(&'-') {
                        current.push('-');
                        current.push('-');
                        chars.next();
                        state = LineComment;
                    } else {
                        current.push(c);
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'*') {
                        current.push('/');
                        current.push('*');
                        chars.next();
                        state = BlockComment;
                    } else {
                        current.push(c);
                    }
                }
                'B' | 'b' => {
                    let mut peek_str = String::from(c);
                    for _ in 0..4 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if peek_str.to_uppercase() == "BEGIN" {
                        begin_end_level += 1;
                        state = BeginEndBlock;
                    }

                    current.push_str(&peek_str);
                }
                ';' => {
                    if begin_end_level == 0 {
                        let trimmed = current.trim();
                        if !trimmed.is_empty() {
                            results.push(trimmed.to_string());
                        }
                        current.clear();
                    } else {
                        current.push(c);
                    }
                }
                _ => {
                    current.push(c);
                }
            },
            SingleQuote => {
                current.push(c);
                if c == '\'' {
                    state = Normal;
                } else if c == '\\' {
                    if let Some(next_c) = chars.next() {
                        current.push(next_c);
                    }
                }
            }
            DoubleQuote => {
                current.push(c);
                if c == '"' {
                    state = Normal;
                } else if c == '\\' {
                    if let Some(next_c) = chars.next() {
                        current.push(next_c);
                    }
                }
            }
            LineComment => {
                current.push(c);
                if c == '\n' {
                    state = Normal;
                }
            }
            BlockComment => {
                current.push(c);
                if c == '*' {
                    if let Some(&'/') = chars.peek() {
                        chars.next();
                        current.push('/');
                        state = Normal;
                    }
                }
            }
            BeginEndBlock => {
                current.push(c);

                if c == 'B' || c == 'b' {
                    let mut peek_str = String::from(c);
                    for _ in 0..4 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if peek_str.to_uppercase() == "BEGIN" {
                        begin_end_level += 1;
                    }
                    current.push_str(&peek_str[1..]);
                }

                if c == 'E' || c == 'e' {
                    let mut peek_str = String::from(c);
                    for _ in 0..2 {
                        if let Some(&ch) = chars.peek() {
                            peek_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if peek_str.to_uppercase() == "END" && begin_end_level > 0 {
                        begin_end_level -= 1;
                        if begin_end_level == 0 {
                            state = Normal;
                        }
                    }
                    current.push_str(&peek_str[1..]);
                }
            }
        }
    }

    // Tambahkan statement terakhir jika ada
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        results.push(trimmed.to_string());
    }

    results
}

// pub fn is_sql_type(query: &str, keyword: &str) -> bool {
//     let pattern = format!(
//         r"(?si)^\s*((--.*(\r\n|\r|\n))|(\/\*[\s\S]*?\*\/))*\s*{}\b",
//         keyword
//     );
//     Regex::new(&pattern).unwrap().is_match(query)
// }

pub fn is_sql_type(query: &str, keyword: &str) -> bool {
    // Contoh: keyword = "DROP|CREATE|ALTER"
    let pattern = format!(
        r"(?si)^\s*((--[^\n]*\n?)|(/\*[\s\S]*?\*/))*\s*{}(\s|\(|$)",
        keyword
    );
    Regex::new(&pattern).unwrap().is_match(query)
}

pub fn is_only_comment(query: &str) -> bool {
    let re = Regex::new(r"(?s)^\s*((--[^\n]*\n?)|(/\*[\s\S]*?\*/))*\s*$").unwrap();
    re.is_match(query)
}

pub fn extract_query_parts(flat_query: &str) -> Option<(String, String)> {
    // let query_pattern = r"(?is)(SELECT)\s+.*?\s+FROM\s+(\S+)
    //                     |(INSERT)\s+INTO\s+(\S+)
    //                     |(UPDATE)\s+(\S+)\s+SET
    //                     |(DELETE)\s+FROM\s+(\S+)
    //                     |((?:CREATE OR REPLACE|CREATE|REPLACE|ALTER|DROP)\s+(?:FUNCTION|TABLE|VIEW|PROCEDURE))\s+(\S+)";
    let query_pattern = r"(?is)(SELECT)\s+.*?\s+FROM\s+(\S+)|(INSERT)\s+INTO\s+(\S+)|(UPDATE)\s+(\S+)\s+SET|(DELETE)\s+FROM\s+(\S+)|(CREATE|REPLACE|ALTER|DROP)\s+(\S+)";

    let re = Regex::new(query_pattern).unwrap();

    if let Some(caps) = re.captures(flat_query) {
        for i in (2..=10).rev().step_by(2) {
            if let (Some(name_match), Some(action_match)) = (caps.get(i), caps.get(i - 1)) {
                let name = name_match.as_str().to_lowercase();
                let action = action_match.as_str().to_lowercase();
                return Some((name, action));
            }
        }
    }
    None
}
