//! CSV reader matching Python's `csv.DictReader` default dialect:
//! comma delimiter, `"` quotechar, doubled-quote escaping, embedded newlines
//! inside quoted fields, CRLF/LF tolerant. Operates on raw bytes so UTF-8
//! multi-byte sequences pass through intact.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type Row = HashMap<String, String>;

pub struct CsvTable {
    pub headers: Vec<String>,
    pub rows: Vec<Row>,
}

/// Parse CSV text into (headers, rows).
pub fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let bytes = text.as_bytes();
    let mut fields: Vec<Vec<String>> = Vec::new();
    let mut field: Vec<u8> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut i = 0usize;
    let n = bytes.len();
    let mut field_quoted = false;

    fn take_field(field: &mut Vec<u8>) -> String {
        // `text` is valid UTF-8 and only ASCII bytes are consumed as
        // delimiters/quotechars, so every field boundary falls on a char
        // boundary.
        String::from_utf8(std::mem::take(field)).unwrap_or_default()
    }

    while i < n {
        if in_quotes {
            if bytes[i] == b'"' {
                if i + 1 < n && bytes[i + 1] == b'"' {
                    field.push(b'"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
                i += 1;
                continue;
            }
            field.push(bytes[i]);
            i += 1;
            continue;
        }
        match bytes[i] {
            b'"' if field.is_empty() && !field_quoted => {
                in_quotes = true;
                field_quoted = true;
                i += 1;
            }
            b',' => {
                record.push(take_field(&mut field));
                field_quoted = false;
                i += 1;
            }
            b'\r' => {
                if i + 1 < n && bytes[i + 1] == b'\n' {
                    i += 1;
                }
                record.push(take_field(&mut field));
                field_quoted = false;
                fields.push(std::mem::take(&mut record));
                i += 1;
            }
            b'\n' => {
                record.push(take_field(&mut field));
                field_quoted = false;
                fields.push(std::mem::take(&mut record));
                i += 1;
            }
            b => {
                field.push(b);
                i += 1;
            }
        }
    }
    if !field.is_empty() || !record.is_empty() || field_quoted {
        record.push(take_field(&mut field));
        fields.push(record);
    }

    if fields.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let headers = fields[0].clone();
    let data = fields[1..].to_vec();
    (headers, data)
}

/// Read a CSV file into headers + dict rows (like csv.DictReader).
/// Missing trailing fields become "" (DictReader restval=None → we map to "").
pub fn read_csv(path: &Path) -> Result<CsvTable, String> {
    let raw = fs::read(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    // Python opens with utf-8 strict; tolerate BOM.
    let text =
        String::from_utf8(raw).map_err(|e| format!("{}: invalid utf-8: {}", path.display(), e))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_string();
    let (headers, data) = parse_csv(&text);
    let rows = data
        .into_iter()
        .map(|rec| {
            let mut row = Row::new();
            for (i, h) in headers.iter().enumerate() {
                row.insert(h.clone(), rec.get(i).cloned().unwrap_or_default());
            }
            row
        })
        .collect();
    Ok(CsvTable { headers, rows })
}
