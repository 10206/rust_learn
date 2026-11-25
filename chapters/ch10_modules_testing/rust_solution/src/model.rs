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
    /// 소문자 이름을 카테고리로 바꿉니다. 모르는 이름이면 `None`.
    pub fn parse(s: &str) -> Option<Category> {
        match s {
            "food" => Some(Category::Food),
            "transport" => Some(Category::Transport),
            "salary" => Some(Category::Salary),
            "rent" => Some(Category::Rent),
            "other" => Some(Category::Other),
            _ => None,
        }
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
    pub fn is_income(&self) -> bool {
        self.amount_cents > 0
    }

    /// (연, 월) 키. `pub(crate)` = 이 크레이트 안(report 모듈)에서만 보입니다.
    pub(crate) fn month_key(&self) -> (u16, u8) {
        (self.year, self.month)
    }
}

/// 센트 정수를 `1234.56` 형태 문자열로 바꿉니다. 부동소수점을 쓰지 않습니다.
///
/// ```
/// use ch10_modules_testing_solution::model::format_cents;
/// assert_eq!(format_cents(123456), "1234.56");
/// assert_eq!(format_cents(-5), "-0.05");
/// assert_eq!(format_cents(0), "0.00");
/// ```
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
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
