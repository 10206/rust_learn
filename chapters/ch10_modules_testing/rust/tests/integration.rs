//! 통합 테스트: `tests/` 폴더의 파일은 각각 **별도 크레이트**로 컴파일되며
//! 라이브러리의 `pub` 항목만 쓸 수 있습니다 (바깥 사용자의 입장).
//! (`pub(crate)` 인 `month_key` 나 비공개 `parse_date` 는 여기서 보이지 않아요.)

use ch10_modules_testing::model::{Category, format_cents};
use ch10_modules_testing::parser::{ParseError, parse_ledger};
use ch10_modules_testing::report::{expense_by_category, monthly_totals};

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

// ─────────────────────────────────────────────
// TODO(9): 직접 통합 테스트를 작성하세요.
//   시나리오: 알 수 없는 카테고리('gift') 한 줄이 들어간 CSV 를 `parse_ledger` 에 넣으면
//            - 성공 목록은 비어 있고
//            - 에러가 정확히 1개이며
//            - `errors[0].to_string()` 이 "line 2: unknown category 'gift'" 이다.
//   입력 예: "date,category,amount_cents,description\n2024-01-03,gift,-100,present\n"
//   💡 힌트: 이 파일 위쪽의 다른 테스트 모양을 따라 하세요. `use` 는 이미 위에 있습니다.
//   ❓ 왜?: `tests/` 의 파일은 라이브러리를 **외부 크레이트처럼** 가져다 씁니다. pub 이 아닌 건 못 써요.
//          공개 API 만으로 시나리오가 시험되는지 확인하는 용도입니다.
// ─────────────────────────────────────────────
#[test]
fn unknown_category_is_reported_with_line_number() {
    todo!("TODO(9): 통합 테스트 직접 작성")
}

#[test]
fn category_is_reexported_at_crate_root() {
    // lib.rs 의 `pub use` 덕분에 짧은 경로로도 접근할 수 있어요.
    assert_eq!(
        ch10_modules_testing::Category::parse("food"),
        Some(Category::Food)
    );
}
