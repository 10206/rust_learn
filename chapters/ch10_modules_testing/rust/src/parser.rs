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

// ─────────────────────────────────────────────
// TODO(4): "2024-01-05" → Some((2024, 1, 5)). 형식이 틀리면 None.
//   규칙: '-' 로 정확히 3조각, 연도는 정확히 4글자, 월 1..=12, 일 1..=31, 숫자로 파싱되어야 함.
//   🐍 Python: _parse_date                                           (python/ledger/parser.py)
//   ⚙️ C++  : 익명 namespace 안의 parse_date                          (cpp/parser.cpp)
//   🦀 Rust : `pub` 이 없는 **비공개 함수**입니다. 이 파일 안(그리고 아래 `mod tests`)에서만 보여요.
//   💡 힌트: `s.split('-')` 이터레이터에서 `it.next()?` 를 세 번, 그 뒤에 `it.next().is_some()` 이면 조각이 너무 많은 것.
//           `y.parse::<u16>().ok()?`,  `(1..=12).contains(&month)`
//   ❓ 왜?: 비공개 함수도 같은 파일의 `#[cfg(test)] mod tests` 에서는 `use super::*` 로 테스트할 수 있어요.
//          (통합 테스트에서는 못 봅니다 — pub 만 보이는 "바깥 사용자" 시점이라서)
// ─────────────────────────────────────────────
fn parse_date(s: &str) -> Option<(u16, u8, u8)> {
    todo!("TODO(4): 날짜 파싱")
}

/// 한 줄 `date,category,amount_cents,description` 을 파싱합니다.
pub fn parse_line(line_no: usize, line: &str) -> Result<Transaction, ParseError> {
    // ─────────────────────────────────────────────
    // TODO(5): 쉼표로 나눠 Transaction 을 만드세요. 검사 순서는 필드 개수(4개) → 날짜 → 카테고리 → 금액.
    //   실패하면 알맞은 ParseError 변형(줄 번호 `line_no` 포함)을 Err 로 돌려줍니다.
    //   🐍 Python: raise ParseError(...)                                 (python/ledger/parser.py: parse_line)
    //   ⚙️ C++  : ParseResult{ok, err}                                    (cpp/parser.cpp: parse_line)
    //   🦀 Rust : Option → Result 변환은 `.ok_or_else(|| ParseError::...)?`, parse 에러는 `.map_err(|_| ...)?`
    //   💡 힌트: `use crate::model::{Category, Transaction};` 가 파일 위에 이미 있어요 — `crate::` 는 크레이트 루트부터의 절대 경로.
    //           `ParseError::FieldCount { line: line_no, found: .. }`, 에러 안에는 틀린 원문 `text: String` 을 넣습니다.
    //   ❓ 왜?: 에러가 줄 번호와 원문을 들고 있으면 사용자에게 바로 "몇 번째 줄의 무엇이 틀렸는지" 알려 줄 수 있습니다 (ch06 복습).
    // ─────────────────────────────────────────────
    todo!("TODO(5): 한 줄 파싱")
}

/// 전체 텍스트를 파싱합니다. 헤더(`date,`로 시작)와 빈 줄은 건너뛰고,
/// 잘못된 줄은 에러 목록에 모아 둡니다 (하나 틀렸다고 전체를 포기하지 않아요).
pub fn parse_ledger(text: &str) -> (Vec<Transaction>, Vec<ParseError>) {
    // ─────────────────────────────────────────────
    // TODO(6): 전체 텍스트를 줄 단위로 파싱해 (성공 목록, 에러 목록) 을 돌려주세요.
    //   - 줄 번호는 **1부터** (`enumerate()` 는 0부터라서 +1)
    //   - 빈 줄(공백뿐 포함)과 헤더 줄(`date,` 로 시작)은 건너뜁니다 (에러도 아님).
    //   - 한 줄이 틀려도 멈추지 말고 에러만 모읍니다.
    //   🐍 Python: for i, line in enumerate(text.splitlines()) + try/except    (python/ledger/parser.py: parse_ledger)
    //   ⚙️ C++  : '\n' 로 잘라 가며 parse_line                                 (cpp/parser.cpp: parse_ledger)
    //   🦀 Rust : `match parse_line(..) { Ok(t) => ..., Err(e) => ... }`
    //   ❓ 왜?: "부분 성공"을 허용하는 설계입니다. 하나 틀렸다고 전체를 `Err` 로 포기하면 사용자가 가장 불편해요.
    // ─────────────────────────────────────────────
    todo!("TODO(6): 전체 파싱")
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
