//! 통합 테스트: `tests/` 폴더의 파일은 각각 **별도 크레이트**로 컴파일되며
//! 라이브러리의 `pub` 항목만 쓸 수 있습니다 (바깥 사용자의 입장).
//! (`pub(crate)` 인 `month_key` 나 비공개 `parse_date` 는 여기서 보이지 않아요.)

use ch10_modules_testing_solution::model::{Category, format_cents};
use ch10_modules_testing_solution::parser::{ParseError, parse_ledger};
use ch10_modules_testing_solution::report::{expense_by_category, monthly_totals};

const LEDGER: &str = "\
date,category,amount_cents,description
2024-01-03,salary,300000,pay
2024-01-05,rent,-120000,rent
2024-01-07,food,-4550,groceries
2024-02-01,salary,300000,pay
2024-02-03,food,-1275,lunch
";

#[test]
fn full_pipeline() {
    let (txs, errors) = parse_ledger(LEDGER);
    assert!(errors.is_empty());
    assert_eq!(txs.len(), 5);

    let totals = monthly_totals(&txs);
    let jan = totals[&(2024, 1)];
    assert_eq!(format_cents(jan.net()), "1754.50");
    let feb = totals[&(2024, 2)];
    assert_eq!(format_cents(feb.net()), "2987.25");

    let by_cat = expense_by_category(&txs);
    assert_eq!(by_cat["food"], 5825);
    assert_eq!(by_cat["rent"], 120000);
    assert!(!by_cat.contains_key("salary"));
}

#[test]
fn bad_lines_do_not_stop_parsing() {
    let text = "2024-01-03,salary,100,pay\nbroken\n2024-01-04,food,-5,tea\n";
    let (txs, errors) = parse_ledger(text);
    assert_eq!(txs.len(), 2);
    assert_eq!(errors, vec![ParseError::FieldCount { line: 2, found: 1 }]);
}

// TODO(9) 의 정답: 알 수 없는 카테고리가 줄 번호와 함께 보고되는지 검사
#[test]
fn unknown_category_is_reported_with_line_number() {
    let text = "date,category,amount_cents,description\n2024-01-03,gift,-100,present\n";
    let (txs, errors) = parse_ledger(text);
    assert!(txs.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].to_string(), "line 2: unknown category 'gift'");
}

#[test]
fn category_is_reexported_at_crate_root() {
    // lib.rs 의 `pub use` 덕분에 짧은 경로로도 접근할 수 있어요.
    assert_eq!(
        ch10_modules_testing_solution::Category::parse("food"),
        Some(Category::Food)
    );
}
