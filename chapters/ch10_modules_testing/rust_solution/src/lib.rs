//! # 가계부 라이브러리 (Ch10 정답)
//!
//! 모듈 구성:
//! - [`model`]  : 도메인 타입 (`Transaction`, `Category`) 과 금액 포맷
//! - [`parser`] : CSV 텍스트 → `Transaction`
//! - [`report`] : 월별 합계, 카테고리별 지출
//!
//! ```
//! use ch10_modules_testing_solution::{parser, report};
//!
//! let (txs, errors) = parser::parse_ledger("2024-01-03,salary,1000,pay\n");
//! assert!(errors.is_empty());
//! assert_eq!(report::monthly_totals(&txs)[&(2024, 1)].income_cents, 1000);
//! ```

pub mod model;
pub mod parser;
pub mod report;

// 자주 쓰는 이름은 크레이트 루트에서 바로 쓸 수 있게 다시 내보냅니다 (re-export).
pub use model::{Category, Transaction};
