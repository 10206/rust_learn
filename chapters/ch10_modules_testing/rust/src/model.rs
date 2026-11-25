//! 도메인 모델: 거래(`Transaction`)와 카테고리(`Category`).

/// 지출/수입 분류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Food,
    Transport,
    Salary,
    Rent,
    Other,
}

impl Category {
    // ─────────────────────────────────────────────
    // TODO(1): 소문자 이름("food", "transport", "salary", "rent", "other")을 Category 로 바꾸세요. 모르는 이름이면 None.
    //   🐍 Python: Category.parse (for c in Category: if c.value == s)       (python/ledger/model.py)
    //   ⚙️ C++  : parse_category -> std::optional<Category>                  (cpp/model.cpp)
    //   🦀 Rust : `match s { "food" => Some(Category::Food), ... , _ => None }`
    //   💡 힌트: 대문자("Food")는 인식하지 않습니다 (테스트가 확인해요).
    //   ❓ 왜?: 문자열 → enum 변환을 한 곳에 모아 두면 새 카테고리를 추가할 때 컴파일러가 match 누락을 알려 줘요.
    // ─────────────────────────────────────────────
    /// 소문자 이름을 카테고리로 바꿉니다. 모르는 이름이면 `None`.
    pub fn parse(s: &str) -> Option<Category> {
        todo!("TODO(1): 문자열 → Category")
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Food => "food",
            Category::Transport => "transport",
            Category::Salary => "salary",
            Category::Rent => "rent",
            Category::Other => "other",
        }
    }
}

/// 가계부 한 줄. 금액은 센트 단위 정수 (수입 +, 지출 -).
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub category: Category,
    pub amount_cents: i64,
    pub description: String,
}

impl Transaction {
    // ─────────────────────────────────────────────
    // TODO(2): 금액이 0 보다 크면 수입(true).
    //   💡 한 줄이에요. 이 메서드는 `pub` 이라서 report.rs, tests/, main.rs 어디서든 쓸 수 있습니다.
    // ─────────────────────────────────────────────
    pub fn is_income(&self) -> bool {
        todo!("TODO(2): 수입 여부")
    }

    /// (연, 월) 키. `pub(crate)` = 이 크레이트 안(report 모듈)에서만 보입니다.
    pub(crate) fn month_key(&self) -> (u16, u8) {
        (self.year, self.month)
    }
}

/// 센트 정수를 `1234.56` 형태 문자열로 바꿉니다. 부동소수점을 쓰지 않습니다.
///
/// ```
/// use ch10_modules_testing::model::format_cents;
/// assert_eq!(format_cents(123456), "1234.56");
/// assert_eq!(format_cents(-5), "-0.05");
/// assert_eq!(format_cents(0), "0.00");
/// ```
pub fn format_cents(cents: i64) -> String {
    // ─────────────────────────────────────────────
    // TODO(3): 센트 정수를 "1234.56" 꼴로 바꾸세요. 음수는 "-45.50", 5센트는 "0.05", -5 는 "-0.05".
    //   🐍 Python: f"{sign}{a // 100}.{a % 100:02d}"                   (python/ledger/model.py: format_cents)
    //   ⚙️ C++  : snprintf("%s%lld.%02lld")                            (cpp/model.cpp: format_cents)
    //   🦀 Rust : `format!("{sign}{}.{:02}", ..)`  — `{:02}` 는 "두 자리, 0으로 채움"
    //   💡 힌트: 부호는 따로 빼고, 절댓값은 `cents.unsigned_abs()` (i64::MIN 도 안전하게 u64 로 바꿔 줘요).
    //   ❓ 왜?: 돈에 f64 를 쓰면 0.1+0.2 같은 오차가 생겨요. 정수 센트로 들고 있다가 출력할 때만 점을 찍습니다.
    //   📝 이 함수 위의 `///` 안 코드는 **문서 테스트**입니다. `cargo test` 가 실제로 실행해 줘요!
    // ─────────────────────────────────────────────
    todo!("TODO(3): 센트 → 문자열")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_round_trip() {
        for c in [
            Category::Food,
            Category::Transport,
            Category::Salary,
            Category::Rent,
            Category::Other,
        ] {
            assert_eq!(Category::parse(c.as_str()), Some(c));
        }
        assert_eq!(Category::parse("gift"), None);
        assert_eq!(Category::parse("Food"), None);
    }

    #[test]
    fn format_cents_cases() {
        assert_eq!(format_cents(150050), "1500.50");
        assert_eq!(format_cents(-4550), "-45.50");
        assert_eq!(format_cents(7), "0.07");
        assert_eq!(format_cents(-5), "-0.05");
    }

    #[test]
    fn income_and_month_key() {
        let t = Transaction {
            year: 2024,
            month: 3,
            day: 9,
            category: Category::Salary,
            amount_cents: 100,
            description: "pay".to_string(),
        };
        assert!(t.is_income());
        assert_eq!(t.month_key(), (2024, 3));
    }
}

// ─────────────────────────────────────────────
// 🔬 실험 1: `pub(crate)` 의 벽  (E0624 — private method)
//   `src/main.rs` 의 `main` 안에 아래 한 줄을 넣어 보세요 (txs 는 이미 있는 변수).
//       let key = txs[0].month_key();
//   ❓ 같은 크레이트 안(report.rs)에서는 되는데 main.rs 에서는 왜 안 될까요? (lib 와 bin 은 별개 크레이트!)
//
// 🔬 실험 2: `mod` 선언을 빼 보기
//   `src/lib.rs` 의 `pub mod report;` 를 지우고 `cargo build` 를 해 보세요. report.rs 파일이 있는데도 왜 "unresolved import" 일까요?
// ─────────────────────────────────────────────
