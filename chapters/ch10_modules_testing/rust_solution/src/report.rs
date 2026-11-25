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
    let mut totals: BTreeMap<(u16, u8), MonthTotal> = BTreeMap::new();
    for t in txs {
        let total = totals.entry(t.month_key()).or_default();
        if t.is_income() {
            total.income_cents += t.amount_cents;
        } else {
            total.expense_cents += -t.amount_cents;
        }
    }
    totals
}

/// 카테고리 이름 → 지출 합계(양수). 지출(음수 금액)만 셉니다.
pub fn expense_by_category(txs: &[Transaction]) -> BTreeMap<&'static str, i64> {
    let mut map = BTreeMap::new();
    for t in txs.iter().filter(|t| t.amount_cents < 0) {
        *map.entry(t.category.as_str()).or_insert(0) += -t.amount_cents;
    }
    map
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

    #[test]
    fn net_is_income_minus_expense() {
        let t = MonthTotal {
            income_cents: 10,
            expense_cents: 25,
        };
        assert_eq!(t.net(), -15);
    }
}
