//! 리포트: 월별 합계와 카테고리별 지출.

use std::collections::BTreeMap;

use crate::model::Transaction;

/// 한 달의 수입/지출 합계 (둘 다 양수로 저장).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MonthTotal {
    pub income_cents: i64,
    pub expense_cents: i64,
}

impl MonthTotal {
    pub fn net(&self) -> i64 {
        self.income_cents - self.expense_cents
    }
}

/// (연, 월) → 합계. `BTreeMap` 이라 월 순서대로 정렬됩니다.
pub fn monthly_totals(txs: &[Transaction]) -> BTreeMap<(u16, u8), MonthTotal> {
    // ─────────────────────────────────────────────
    // TODO(7): (연, 월) 별로 수입 합계와 지출 합계(양수)를 모으세요.
    //   `t.is_income()` 이면 income 에 더하고, 아니면 expense 에 `-amount` 를 더합니다.
    //   🐍 Python: totals.setdefault(key, MonthTotal())                     (python/ledger/report.py)
    //   ⚙️ C++  : totals[{year, month}]                                      (cpp/report.cpp)
    //   🦀 Rust : `totals.entry(t.month_key()).or_default()` — MonthTotal 이 Default 를 derive 했기 때문에 가능
    //   💡 힌트: `month_key()` 는 model.rs 에서 `pub(crate)` 입니다 → 같은 크레이트인 여기서는 쓸 수 있어요.
    //           main.rs 나 tests/ 에서는 못 씁니다 (직접 시도해 보세요: 🔬 실험 1).
    // ─────────────────────────────────────────────
    todo!("TODO(7): 월별 합계")
}

/// 카테고리 이름 → 지출 합계(양수). 지출(음수 금액)만 셉니다.
pub fn expense_by_category(txs: &[Transaction]) -> BTreeMap<&'static str, i64> {
    // ─────────────────────────────────────────────
    // TODO(8): 지출(금액 < 0)만 골라 카테고리 이름별 합계(양수)를 구하세요.
    //   🐍 Python: out[name] = out.get(name, 0) - t.amount_cents             (python/ledger/report.py)
    //   ⚙️ C++  : out[category_name(t.category)] += -t.amount_cents           (cpp/report.cpp)
    //   🦀 Rust : `*map.entry(key).or_insert(0) += ...` (Ch07 entry API 복습)
    //   💡 힌트: 키 타입이 `&'static str` 이라 문자열 복사가 없습니다 (`Category::as_str`).
    // ─────────────────────────────────────────────
    todo!("TODO(8): 카테고리별 지출")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    fn tx(month: u8, category: Category, amount_cents: i64) -> Transaction {
        Transaction {
            year: 2024,
            month,
            day: 1,
            category,
            amount_cents,
            description: String::new(),
        }
    }

    #[test]
    fn monthly_totals_split_income_and_expense() {
        let txs = vec![
            tx(1, Category::Salary, 1000),
            tx(1, Category::Food, -300),
            tx(1, Category::Rent, -200),
            tx(2, Category::Food, -50),
        ];
        let totals = monthly_totals(&txs);
        assert_eq!(totals.len(), 2);
        assert_eq!(
            totals[&(2024, 1)],
            MonthTotal {
                income_cents: 1000,
                expense_cents: 500
            }
        );
        assert_eq!(totals[&(2024, 1)].net(), 500);
        assert_eq!(totals[&(2024, 2)].net(), -50);
    }

    #[test]
    fn expense_by_category_sums_only_expenses() {
        let txs = vec![
            tx(1, Category::Salary, 1000),
            tx(1, Category::Food, -300),
            tx(2, Category::Food, -50),
            tx(2, Category::Rent, -200),
        ];
        let by_cat = expense_by_category(&txs);
        let v: Vec<_> = by_cat.into_iter().collect();
        assert_eq!(v, vec![("food", 350), ("rent", 200)]);
    }

    // ─────────────────────────────────────────────
    // TODO(10): 직접 테스트를 작성하세요! `MonthTotal::net()` 이 (수입 - 지출) 인지 확인합니다.
    //   예) income 10, expense 25 → net 은 -15.  `assert_eq!(t.net(), -15);`
    //   🐍 Python: assert                                                    ⚙️ C++: assert / gtest 의 EXPECT_EQ
    //   🦀 Rust : `#[test]` 가 붙은 함수 = 테스트. 이 파일 안 `mod tests` 는 `cargo test` 때만 컴파일됩니다 (`#[cfg(test)]`).
    //   ❓ 왜?: 단위 테스트는 같은 파일 안에 두는 것이 Rust 의 관례예요. 비공개 함수도 `use super::*` 로 시험할 수 있습니다.
    //   ✍️ 아래 `todo!` 를 지우고 테스트 본문을 채우세요.
    // ─────────────────────────────────────────────
    #[test]
    fn net_is_income_minus_expense() {
        todo!("TODO(10): MonthTotal::net 테스트 작성")
    }
}
