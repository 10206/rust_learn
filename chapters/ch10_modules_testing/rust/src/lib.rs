//! # 가계부 라이브러리 (Ch10 실습)
//!
//! ## 목표
//! - 파일 하나가 모듈 하나: `lib.rs` 의 `mod` 선언 ↔ `model.rs` / `parser.rs` / `report.rs`
//! - `pub` / `pub(crate)` / 비공개의 차이, `use` 경로, `pub use` 재내보내기
//! - 단위 테스트(모듈 안) · 통합 테스트(`tests/`) · 문서 테스트(`///` 안의 코드)
//!
//! ## 대응 파일
//! - Python: `../python/ledger/` 패키지 + `../python/main.py`
//! - C++   : `../cpp/*.hpp`, `../cpp/*.cpp`
//!
//! ## 진행 방법
//! `src/model.rs` → `src/parser.rs` → `src/report.rs` 순서로 `TODO(1)` ~ `TODO(8)` 을 구현하고,
//! `tests/integration.rs` 의 `TODO(9)` 와 `src/report.rs` 의 `TODO(10)` 은 **직접 테스트를 작성**합니다.
//!
//! ## 완료 기준
//! `cargo test -p ch10_modules_testing` 전부 통과 AND `../../scripts/compare.sh ch10` 의 세 언어 출력이 동일.
//! (막히면 `../rust_solution/` 을 열어 보세요.)
//!
//! ## 모듈 지도
//!
//! - [`model`]  : 도메인 타입 (`Transaction`, `Category`) 과 금액 포맷
//! - [`parser`] : CSV 텍스트 → `Transaction`
//! - [`report`] : 월별 합계, 카테고리별 지출
//!
//! ```
//! use ch10_modules_testing::{parser, report};
//!
//! let (txs, errors) = parser::parse_ledger("2024-01-03,salary,1000,pay\n");
//! assert!(errors.is_empty());
//! assert_eq!(report::monthly_totals(&txs)[&(2024, 1)].income_cents, 1000);
//! ```

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 컴파일러가 안 쓴 코드를 알려 줍니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

// ─────────────────────────────────────────────
// 모듈 선언: `mod model;` 한 줄이 "같은 폴더의 model.rs 를 이 자리에 붙여라"는 뜻입니다.
//   🐍 Python: 폴더 + __init__.py 가 있으면 자동으로 패키지 (python/ledger/__init__.py)
//   ⚙️ C++  : #include "model.hpp" + 링크할 .cpp 목록을 CMake 에 적기 (cpp/*.hpp, cpp/*.cpp)
//   🦀 Rust : `mod` 로 선언하지 않은 파일은 컴파일 대상이 아니에요! 파일만 만들고 mod 를 안 적으면 무시됩니다.
//   `pub mod` = 바깥(다른 크레이트, tests/, main.rs)에도 공개.  그냥 `mod` = 이 크레이트 안에서만.
// ─────────────────────────────────────────────
pub mod model;
pub mod parser;
pub mod report;

// 자주 쓰는 이름은 크레이트 루트에서 바로 쓸 수 있게 다시 내보냅니다 (re-export).
pub use model::{Category, Transaction};
