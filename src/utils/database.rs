use serde_json::{Map, Value};
use sqlx::{Column, Row, TypeInfo, any::AnyRow};

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

pub fn rows_to_json(rows: &[AnyRow]) -> Vec<Value> {
    let mut results = Vec::new();

    for row in rows {
        let mut map = Map::new();

        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();
            let type_name = column.type_info().name();

            let value = match type_name {
                "INT2" => row
                    .try_get::<i16, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "INT4" => row
                    .try_get::<i32, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                "INT8" => row
                    .try_get::<i64, _>(i)
                    .map(|v| Value::Number(serde_json::Number::from(v))),
                "FLOAT4" | "FLOAT8" => row.try_get::<f64, _>(i).map(|v| {
                    serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                }),
                "NUMERIC" | "DECIMAL" => row.try_get::<String, _>(i).map(Value::String),
                "BOOL" => row.try_get::<bool, _>(i).map(Value::Bool),
                "DATE" | "TIMESTAMP" | "DATETIME" => row.try_get::<String, _>(i).map(Value::String),
                "OID" => row
                    .try_get::<i32, _>(i)
                    .map(|v| Value::Number((v as i64).into())),
                _ => row.try_get::<String, _>(i).map(Value::String),
            }
            .unwrap_or(Value::Null);

            map.insert(name.to_string(), value);
        }

        results.push(Value::Object(map));
    }

    results
}
