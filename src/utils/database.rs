use poem::http::StatusCode;
use regex::Regex;
use serde_json::{Map, Number, Value};
use sqlx::{Column, Row, TypeInfo, mysql::MySqlRow, postgres::PgRow};
use std::fmt::Write;
use umya_spreadsheet::Style;
use umya_spreadsheet::structs::Fill;
use umya_spreadsheet::structs::Font;
use umya_spreadsheet::structs::PatternFill;
use umya_spreadsheet::{Color, PatternValues};
use umya_spreadsheet::{new_file, writer};

use crate::utils::common;

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

pub fn rows_to_json_postgres(rows: &[PgRow]) -> Vec<Value> {
    let mut results = Vec::new();

    for row in rows {
        let mut map = Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();
            let type_name = column.type_info().name().to_uppercase();

            let value = match type_name.as_str() {
                "INT2" => row
                    .try_get::<i16, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "INT4" => row
                    .try_get::<i32, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "INT8" => row
                    .try_get::<i64, _>(i)
                    .map(|v| Value::Number(Number::from(v))),
                "FLOAT4" | "FLOAT8" => row.try_get::<f64, _>(i).map(|v| {
                    Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }),
                "NUMERIC" | "DECIMAL" => row.try_get::<String, _>(i).map(Value::String),
                "BOOL" => row.try_get::<bool, _>(i).map(Value::Bool),
                "DATE" | "TIMESTAMP" => row.try_get::<String, _>(i).map(Value::String),
                "OID" => row
                    .try_get::<i64, _>(i)
                    .map(|v| Value::Number(Number::from(v)))
                    .or_else(|_| {
                        row.try_get::<i32, _>(i)
                            .map(|v| Value::Number((v as i64).into()))
                    }),
                _ => {
                    // Coba ambil sebagai String dulu
                    row.try_get::<String, _>(i)
                        .map(Value::String)
                        .or_else(|_| row.try_get::<bool, _>(i).map(Value::Bool))
                        .or_else(|_| {
                            row.try_get::<f64, _>(i).map(|v| {
                                Number::from_f64(v)
                                    .map(Value::Number)
                                    .unwrap_or(Value::Null)
                            })
                        })
                }
            }
            .unwrap_or(Value::Null);

            map.insert(name.to_string(), value);
        }

        results.push(Value::Object(map));
    }

    results
}

pub fn rows_to_json_mysql(rows: &[MySqlRow]) -> Vec<Value> {
    let mut results = Vec::new();

    for row in rows {
        let mut map = Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();
            let type_name = column.type_info().name().to_uppercase();

            let value = match type_name.as_str() {
                "SMALLINT" => row
                    .try_get::<i16, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "INT" | "INTEGER" => row
                    .try_get::<i32, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "BIGINT" => row
                    .try_get::<i64, _>(i)
                    .map(|v| Value::Number(Number::from(v))),
                "FLOAT" | "DOUBLE" => row.try_get::<f64, _>(i).map(|v| {
                    Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }),
                "DECIMAL" => row.try_get::<String, _>(i).map(Value::String),
                "BOOLEAN" | "TINYINT(1)" => row.try_get::<bool, _>(i).map(Value::Bool),
                "DATE" | "DATETIME" | "TIMESTAMP" => row.try_get::<String, _>(i).map(Value::String),
                _ => row.try_get::<String, _>(i).map(Value::String),
            }
            .unwrap_or(Value::Null);

            map.insert(name.to_string(), value);
        }

        results.push(Value::Object(map));
    }

    results
}

pub fn extract_columns_info_postgres(rows: &[PgRow]) -> Vec<Value> {
    let mut columns_info = Vec::new();

    if let Some(row) = rows.get(0) {
        for column in row.columns() {
            let mut map = Map::new();
            map.insert("name".to_string(), Value::String(column.name().to_string()));
            map.insert(
                "type".to_string(),
                Value::String(column.type_info().name().to_string()),
            );
            columns_info.push(Value::Object(map));
        }
    }

    columns_info
}

pub fn extract_columns_info_mysql(rows: &[MySqlRow]) -> Vec<Value> {
    let mut columns_info = Vec::new();

    if let Some(row) = rows.get(0) {
        for column in row.columns() {
            let mut map = Map::new();
            map.insert("name".to_string(), Value::String(column.name().to_string()));
            map.insert(
                "type".to_string(),
                Value::String(column.type_info().name().to_string()),
            );
            columns_info.push(Value::Object(map));
        }
    }

    columns_info
}

pub fn rows_to_insert_query_string(
    table_name: &str,
    include_column_name_flag: i16,
    number_line_per_action: i16,
    rows: Vec<Value>,
) -> String {
    let mut result = String::new();
    let mut batch = Vec::new();

    for (row_index, row_value) in rows.iter().enumerate() {
        let obj = match row_value.as_object() {
            Some(obj) => obj,
            None => continue, // skip non-object rows
        };

        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (name, value) in obj {
            columns.push(name.to_string());

            let value_str = match value {
                Value::Null => "NULL".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                _ => "NULL".to_string(), // fallback untuk array, object, dsb
            };

            values.push(value_str);
        }

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
    rows: Vec<Value>,
) -> String {
    let mut result = String::new();

    for row_value in rows {
        let obj = match row_value.as_object() {
            Some(obj) => obj,
            None => continue, // skip jika bukan object
        };

        let mut set_clauses = Vec::new();
        let mut where_clauses = Vec::new();
        let mut column_names: Vec<&String> = obj.keys().collect();
        column_names.sort(); // optional: sort biar konsisten

        for (i, name) in column_names.iter().enumerate() {
            let value = match obj.get(*name).unwrap_or(&Value::Null) {
                Value::Null => "NULL".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                _ => "NULL".to_string(), // default fallback
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

            update_sql.push('\n');
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

pub fn rows_to_xlsx_bytes(first_amount_combined: i16, rows: Vec<Value>) -> poem::Result<Vec<u8>> {
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let mut book = new_file();
    let sheet_name = "Sheet1";
    let _ = book.new_sheet(sheet_name);

    // Ambil kolom dari keys object pertama
    let first_obj = rows[0].as_object().ok_or_else(|| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "xlsx.invalidDataFormat")
    })?;

    let mut column_names: Vec<String> = first_obj.keys().cloned().collect();
    column_names.sort(); // urutkan agar kolom konsisten

    let mut row_idx = 1;

    // Style header
    let mut header_style = Style::default();
    let mut font = Font::default();
    font.set_bold(true);
    font.set_color({
        let mut color = Color::default();
        color.set_argb("FFFFFF");
        color
    });
    font.set_size(14.0);
    header_style.set_font(font);

    let mut fill = Fill::default();
    let mut pattern = PatternFill::default();
    pattern.set_pattern_type(PatternValues::Solid);
    pattern.set_foreground_color({
        let mut color = Color::default();
        color.set_argb("000000");
        color
    });
    pattern.set_background_color({
        let mut color = Color::default();
        color.set_argb("000000");
        color
    });
    fill.set_pattern_fill(pattern);
    header_style.set_fill(fill);

    // Tulis header
    for (col_idx, col_name) in column_names.iter().enumerate() {
        let cell = book
            .get_sheet_by_name_mut(sheet_name)
            .unwrap()
            .get_cell_mut((col_idx as u32 + 1, row_idx));
        cell.set_value(col_name);
        cell.set_style(header_style.clone());
    }

    row_idx += 1;

    // Data matrix
    let mut data_matrix: Vec<Vec<String>> = vec![];

    for value in rows {
        let obj = match value.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let mut row_data = vec![];
        for col_name in &column_names {
            let cell_value = match obj.get(col_name).unwrap_or(&Value::Null) {
                Value::Null => "".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.to_string(),
                _ => "".to_string(),
            };
            row_data.push(cell_value);
        }
        data_matrix.push(row_data);
    }

    // Tulis data ke sheet
    for (r, row_data) in data_matrix.iter().enumerate() {
        for (c, value) in row_data.iter().enumerate() {
            book.get_sheet_by_name_mut(sheet_name)
                .unwrap()
                .get_cell_mut((c as u32 + 1, row_idx + r as u32))
                .set_value(value);
        }
    }

    // Gabung sel jika nilai sama (first_amount_combined)
    let mut last_seen = vec![None; first_amount_combined as usize];
    for (r, row_data) in data_matrix.iter().enumerate() {
        for col_idx in 0..(first_amount_combined as usize).min(column_names.len()) {
            if Some(&row_data[col_idx]) == last_seen[col_idx].as_ref() {
                book.get_sheet_by_name_mut(sheet_name)
                    .unwrap()
                    .get_cell_mut((col_idx as u32 + 1, row_idx + r as u32))
                    .set_value("");
            } else {
                last_seen[col_idx] = Some(row_data[col_idx].clone());
            }
        }
    }

    // Tulis ke buffer
    let mut buffer = Vec::new();
    writer::xlsx::write_writer(&book, &mut buffer).map_err(|_| {
        common::error_message(StatusCode::INTERNAL_SERVER_ERROR, "xlsx.writeFailed")
    })?;

    Ok(buffer)
}

pub fn rows_to_csv_string(header_flag: i16, delimiter: &str, rows: Vec<Value>) -> String {
    let mut result = String::new();

    if rows.is_empty() {
        return result;
    }

    // Ambil header dari key pada object pertama
    let first_obj = match rows[0].as_object() {
        Some(obj) => obj,
        None => return result,
    };

    let mut column_names: Vec<String> = first_obj.keys().cloned().collect();
    column_names.sort(); // urutkan agar kolom konsisten

    // Header
    if header_flag == 1 {
        result.push_str(&column_names.join(delimiter));
        result.push('\n');
    }

    for row in rows {
        let obj = match row.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let mut values = vec![];

        for col in &column_names {
            let raw_val = obj.get(col).unwrap_or(&Value::Null);
            let value = match raw_val {
                Value::Null => "".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.to_string(),
                _ => "".to_string(),
            };

            // Bungkus string jika mengandung delimiter, quote, atau newline
            let formatted_value =
                if value.contains(delimiter) || value.contains('"') || value.contains('\n') {
                    format!("\"{}\"", value.replace('"', "\"\""))
                } else {
                    value
                };

            values.push(formatted_value);
        }

        result.push_str(&values.join(delimiter));
        result.push('\n');
    }

    result
}

pub fn rows_to_json_string(rows: Vec<Value>) -> String {
    serde_json::to_string_pretty(&Value::Array(rows)).unwrap_or_else(|_| "[]".to_string())
}

pub fn rows_to_xml_string(table_name: &str, rows: Vec<Value>) -> String {
    let mut result = String::new();
    result.push_str("<List>\n");

    for row in rows {
        result.push_str(&format!("  <{}>\n", table_name));

        if let Value::Object(map) = row {
            for (col_name, value) in map {
                // Konversi nilai ke string dan escape karakter XML
                let value_str = match value {
                    Value::Null => "".to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s,
                    _ => value.to_string(), // fallback untuk array/object
                };

                let escaped_value = value_str
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&apos;");

                result.push_str(&format!(
                    "    <{}>{}</{}>\n",
                    col_name, escaped_value, col_name
                ));
            }
        }

        result.push_str(&format!("  </{}>\n", table_name));
    }

    result.push_str("</List>\n");
    result
}
