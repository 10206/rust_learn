//! CSV 텍스트를 `Transaction` 으로 바꿉니다.

use std::fmt;

use crate::model::{Category, Transaction};

/// 파싱 실패 이유. 모든 변형이 줄 번호(1부터)를 가집니다.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    FieldCount { line: usize, found: usize },
    BadDate { line: usize, text: String },
    UnknownCategory { line: usize, text: String },
    BadAmount { line: usize, text: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::FieldCount { line, found } => {
                write!(f, "line {line}: expected 4 fields, found {found}")
            }
            ParseError::BadDate { line, text } => write!(f, "line {line}: invalid date '{text}'"),
            ParseError::UnknownCategory { line, text } => {
                write!(f, "line {line}: unknown category '{text}'")
            }
            ParseError::BadAmount { line, text } => {
                write!(f, "line {line}: invalid amount '{text}'")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// "2024-01-05" → (2024, 1, 5). 비공개(private) 도우미 함수.
fn parse_date(s: &str) -> Option<(u16, u8, u8)> {
    let mut it = s.split('-');
    let (y, m, d) = (it.next()?, it.next()?, it.next()?);
    if it.next().is_some() || y.len() != 4 {
        return None;
    }
    let year: u16 = y.parse().ok()?;
    let month: u8 = m.parse().ok()?;
    let day: u8 = d.parse().ok()?;
    if (1..=12).contains(&month) && (1..=31).contains(&day) {
        Some((year, month, day))
    } else {
        None
    }
}

/// 한 줄 `date,category,amount_cents,description` 을 파싱합니다.
pub fn parse_line(line_no: usize, line: &str) -> Result<Transaction, ParseError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 4 {
        return Err(ParseError::FieldCount {
            line: line_no,
            found: fields.len(),
        });
    }
    let (year, month, day) = parse_date(fields[0]).ok_or_else(|| ParseError::BadDate {
        line: line_no,
        text: fields[0].to_string(),
    })?;
    let category = Category::parse(fields[1]).ok_or_else(|| ParseError::UnknownCategory {
        line: line_no,
        text: fields[1].to_string(),
    })?;
    let amount_cents: i64 = fields[2].parse().map_err(|_| ParseError::BadAmount {
        line: line_no,
        text: fields[2].to_string(),
    })?;
    Ok(Transaction {
        year,
        month,
        day,
        category,
        amount_cents,
        description: fields[3].to_string(),
    })
}

/// 전체 텍스트를 파싱합니다. 헤더(`date,`로 시작)와 빈 줄은 건너뛰고,
/// 잘못된 줄은 에러 목록에 모아 둡니다 (하나 틀렸다고 전체를 포기하지 않아요).
pub fn parse_ledger(text: &str) -> (Vec<Transaction>, Vec<ParseError>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if line.trim().is_empty() || line.starts_with("date,") {
            continue;
        }
        match parse_line(line_no, line) {
            Ok(t) => ok.push(t),
            Err(e) => errors.push(e),
        }
    }
    (ok, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_is_private_but_testable() {
        assert_eq!(parse_date("2024-01-05"), Some((2024, 1, 5)));
        assert_eq!(parse_date("2024-13-01"), None);
        assert_eq!(parse_date("2024-01"), None);
        assert_eq!(parse_date("24-01-05"), None);
    }

    #[test]
    fn parse_line_ok() {
        let t = parse_line(2, "2024-01-07,food,-4550,groceries").unwrap();
        assert_eq!((t.year, t.month, t.day), (2024, 1, 7));
        assert_eq!(t.category, Category::Food);
        assert_eq!(t.amount_cents, -4550);
        assert_eq!(t.description, "groceries");
    }

    #[test]
    fn parse_line_errors() {
        assert_eq!(
            parse_line(3, "2024-01-07,food"),
            Err(ParseError::FieldCount { line: 3, found: 2 })
        );
        assert_eq!(
            parse_line(4, "2024-1x-07,food,1,a"),
            Err(ParseError::BadDate {
                line: 4,
                text: "2024-1x-07".to_string()
            })
        );
        assert_eq!(
            parse_line(5, "2024-01-07,gift,1,a").unwrap_err().to_string(),
            "line 5: unknown category 'gift'"
        );
        assert_eq!(
            parse_line(6, "2024-01-07,food,abc,a").unwrap_err().to_string(),
            "line 6: invalid amount 'abc'"
        );
    }

    #[test]
    fn parse_ledger_skips_header_and_blank_lines() {
        let text = "date,category,amount_cents,description\n\n2024-01-03,salary,100,pay\nbad\n";
        let (ok, errors) = parse_ledger(text);
        assert_eq!(ok.len(), 1);
        assert_eq!(errors, vec![ParseError::FieldCount { line: 4, found: 1 }]);
    }
}
