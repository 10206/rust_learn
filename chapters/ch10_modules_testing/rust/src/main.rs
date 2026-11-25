//! 가계부 CLI. 라이브러리(lib.rs)를 가져다 쓰는 얇은 껍데기입니다. (이 파일은 완성되어 있어요)
//! 실행: cargo run -q -p ch10_modules_testing   (챕터 폴더에서)
//!
//! main.rs 는 lib.rs 와 **별개의 크레이트**라서 `ch10_modules_testing::...` 로 라이브러리를 `use` 합니다.
//! 그래서 `pub` 이 아닌 항목은 여기서 보이지 않아요.

use std::fs;

use ch10_modules_testing::model::format_cents;
use ch10_modules_testing::{parser, report};

fn main() {
    let text = fs::read_to_string("data/ledger.csv").expect("data/ledger.csv 를 읽을 수 없어요");
    let (txs, errors) = parser::parse_ledger(&text);
    println!("loaded {} transactions, {} errors", txs.len(), errors.len());
    for e in &errors {
        println!("{e}");
    }

    println!("== monthly totals ==");
    for ((year, month), total) in report::monthly_totals(&txs) {
        println!(
            "{year}-{month:02}  income {:>9}  expense {:>9}  net {:>9}",
            format_cents(total.income_cents),
            format_cents(total.expense_cents),
            format_cents(total.net())
        );
    }

    println!("== expenses by category ==");
    for (category, cents) in report::expense_by_category(&txs) {
        println!("{category:<10} {:>9}", format_cents(cents));
    }
}
