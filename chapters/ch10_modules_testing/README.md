# Ch10. 모듈·크레이트·테스트

> 파일을 어떻게 나누고, 무엇을 공개하고, 어떻게 검증하는가 — "프로젝트 구조"를 읽는 눈  ·  예상 소요: 이론 60분 / 실습 80분  ·  선행: Ch05, Ch06, Ch08

이 챕터는 코드 한 장이 아니라 **여러 파일로 이루어진 라이브러리**를 다룹니다. 그래서 정답도 `rust/examples/solution.rs` 가 아니라
형제 크레이트 **`rust_solution/`** (패키지 이름 `ch10_modules_testing_solution`) 입니다.

## 🎯 학습 목표

- **패키지(package) / 크레이트(crate) / 모듈(module)** 의 차이를 설명하고, 파일 ↔ 모듈 대응 규칙을 알 수 있습니다.
- `mod`, `pub`, `pub(crate)`, `use`, `pub use`, `crate::`/`super::`/`self::` 경로를 읽고 쓸 수 있습니다.
- `lib.rs` + `main.rs` 분리(라이브러리 + 얇은 실행 파일) 구조와 `Cargo.toml`(의존성·features·workspace)을 이해합니다.
- **단위 테스트**(`#[cfg(test)]`), **통합 테스트**(`tests/`), **문서 테스트**(`///` 안의 코드)를 각각 작성·실행할 수 있습니다.
- `cargo test`, `cargo doc`, `cargo clippy`, `cargo fmt` 를 일상 워크플로로 사용합니다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 빌드/패키지 단위 | CMake target (라이브러리/실행파일) | 배포 패키지 (wheel) | **패키지** = `Cargo.toml` 하나 |
| 컴파일 단위 | 번역 단위(.cpp 한 개) | 모듈(.py 한 개) | **크레이트** (패키지 안에 lib 1개 + bin 여러 개) |
| 코드 조직 | `namespace` + 헤더/소스 분리 | 모듈 / 패키지(폴더 + `__init__.py`) | **모듈 트리** (`mod`), 파일/폴더에 대응 |
| 선언 공유 | `#include "x.hpp"` | `import x` | `mod x;` (선언) + `use x::Y;` (이름 가져오기) |
| 공개 범위 | `public`/`private`(클래스), 헤더에 선언해야 외부 공개 | `_name` 관례뿐 | 기본 **private**, `pub` / `pub(crate)` / `pub(super)` |
| 외부 라이브러리 | vcpkg/Conan + CMake `find_package` | `pip install` + requirements | `Cargo.toml` 의 `[dependencies]` |
| 여러 프로젝트 묶기 | CMake `add_subdirectory` | monorepo / poetry workspace | **workspace** (루트 `Cargo.toml`) |
| 단위 테스트 | GoogleTest / Catch2 (별도 설치) | `unittest` / `pytest` | 내장 `#[test]` + `cargo test` |
| 통합 테스트 | 별도 실행 파일 | `tests/` + pytest | `tests/` 폴더 (자동으로 외부 사용자 입장) |
| 문서 + 예제 검증 | Doxygen | docstring + doctest | `///` + **doc test** + `cargo doc` |

## 📘 이론

> 이 챕터의 이론 코드 중 단일 파일 예제는 <https://play.rust-lang.org> 에서 실행할 수 있습니다 (`mod` 를 인라인으로 쓰는 경우).
> 여러 파일 예제는 이 저장소의 `rust_solution/` 을 열어 따라가 보세요.

### 1. 패키지, 크레이트, 모듈

```text
패키지 (Cargo.toml 이 있는 폴더 = cargo 가 다루는 단위)
 └─ 크레이트 (rustc 가 한 번에 컴파일하는 단위)
      ├─ 라이브러리 크레이트: src/lib.rs      (패키지당 최대 1개)
      └─ 바이너리 크레이트  : src/main.rs, src/bin/*.rs   (여러 개 가능)
           └─ 모듈 (mod): 크레이트 안의 이름 공간 트리
```

> 💡 **쉬운 설명**: 패키지는 "상품 상자", 크레이트는 "그 안에 든 완성품 하나(라이브러리 또는 실행 파일)",
> 모듈은 완성품 안의 "서랍 칸막이"입니다. C++ 에서 CMake 의 target 이 크레이트와 가깝고, `namespace` 가 모듈과 가깝습니다.

이 챕터의 `rust_solution/` 패키지는 **크레이트 2개**를 가집니다.

| 크레이트 | 파일 | 역할 |
|---|---|---|
| 라이브러리 `ch10_modules_testing_solution` | `src/lib.rs` (+ `model.rs`, `parser.rs`, `report.rs`) | 로직 전부. 테스트하기 쉬움 |
| 실행 파일 `ch10_modules_testing_solution` | `src/main.rs` | 파일 읽고, 라이브러리를 호출해 출력만 하는 **얇은 껍데기** |

main.rs 는 라이브러리를 **외부 크레이트처럼** `use ch10_modules_testing_solution::...` 로 가져다 씁니다.
이 분리 덕분에 (1) 로직을 단위/통합 테스트로 검증하기 쉽고, (2) 나중에 다른 프로그램에서 라이브러리만 재사용할 수 있어요.
AI 가 만든 프로젝트 대부분이 이 구조입니다.

### 2. 파일 ↔ 모듈 규칙

```text
src/
 ├─ lib.rs         ← 크레이트 루트.  `pub mod model;` `pub mod parser;` `pub mod report;`
 ├─ model.rs       ← 모듈 crate::model
 ├─ parser.rs      ← 모듈 crate::parser
 └─ report.rs      ← 모듈 crate::report
```

```rust
// lib.rs
pub mod model;      // "같은 폴더의 model.rs (또는 model/mod.rs) 를 여기 모듈로 붙여라"
pub mod parser;
pub mod report;
```

- **`mod model;` 로 선언해야** 파일이 컴파일에 포함됩니다. 파일만 만들고 선언을 안 하면 **조용히 무시**돼요 (⚠️ C++/Python 과 가장 다른 점).
- 하위 모듈이 더 필요하면: `parser.rs` 안에 `mod lexer;` → `src/parser/lexer.rs` (폴더 이름 = 부모 모듈 이름).
  예전 방식인 `src/parser/mod.rs` 도 같은 뜻으로 인정됩니다. AI 코드에서 두 스타일을 모두 볼 수 있어요.
- 인라인 모듈도 가능합니다: `mod tests { ... }` (바로 이 형태로 테스트 모듈을 씁니다).

> 💡 파일이 없는데 `mod missing;` 이라고 쓰면: `error[E0583]: file not found for module 'missing'` +
> `help: to create the module 'missing', create file "missing.rs" or "missing/mod.rs"`

⚠️ **C++ 개발자가 헷갈리는 점**

- Rust 에는 **헤더 파일이 없습니다.** 선언과 정의를 따로 적지 않고, 컴파일러가 크레이트 전체를 한 번에 보고 순서에 상관없이 해석해요 (전방 선언 불필요).
- `#include` 는 텍스트 복붙이지만 `mod`/`use` 는 **이름 공간 연결**입니다. 순환 include 문제 같은 것이 없습니다.
- 빌드 파일에 소스 목록을 적을 필요가 없습니다 (CMake 의 `add_executable(... a.cpp b.cpp)` 불필요). `mod` 선언이 곧 소스 목록이에요.

### 3. 가시성: `pub`, `pub(crate)`, 비공개

**기본은 비공개(private)** 입니다. (C++ `struct` 의 기본이 public 인 것과 반대!)

| 표기 | 보이는 범위 | 이 챕터 예 |
|---|---|---|
| (없음) | 선언된 모듈과 그 **자식 모듈** | `parser::parse_date` (같은 파일의 `mod tests` 는 자식이라 접근 가능) |
| `pub(super)` | 부모 모듈까지 | |
| `pub(crate)` | **같은 크레이트 어디서나**, 바깥 크레이트는 ✗ | `Transaction::month_key` (report.rs 는 쓰고, main.rs·`tests/` 는 못 씀) |
| `pub` | 어디서나 (바깥 크레이트 포함) | `parse_ledger`, `format_cents` |

- `pub struct` 라도 **필드는 따로 `pub` 을 붙여야** 바깥에서 접근됩니다. (`pub enum` 은 변형이 모두 공개.)
- `pub(crate)` 는 "라이브러리 내부 협력용, 외부 API 로는 약속하지 않음"이라는 뜻이라서 API 설계에서 아주 유용합니다.

```rust
mod shapes {
    pub struct Circle { radius: f64 }   // 필드는 비공개
    fn secret() {}                      // 비공개 함수
}
// shapes::secret();  → error[E0603]: function `secret` is private
```

### 4. `use` 와 경로

```rust
use std::collections::BTreeMap;            // 외부(표준) 크레이트에서
use crate::model::{Category, Transaction}; // 이 크레이트 루트부터의 절대 경로
use super::*;                              // 부모 모듈의 모든 것 (테스트 모듈에서 흔함)
use self::helper::run;                     // 현재 모듈의 자식
use ch10_modules_testing_solution::report; // main.rs / tests/ 에서: 라이브러리 크레이트 이름으로 시작
```

- `crate::` = 현재 크레이트 루트, `super::` = 부모 모듈, `self::` = 현재 모듈.
- `use a::b as c;` 로 이름을 바꾸고, `use a::{b, c::d};` 로 묶습니다.
- **`pub use`** = 재내보내기(re-export): `lib.rs` 의 `pub use model::{Category, Transaction};` 덕분에
  사용자는 `ch10_...::model::Category` 대신 `ch10_...::Category` 로 짧게 쓸 수 있어요. 내부 파일 구조를 숨기고 **API 모양을 따로 설계**하는 도구입니다.
- 경로를 못 찾으면 `E0432`(use 의 경로 오류) 또는 `E0433`(모듈/크레이트 없음)가 납니다.

### 5. Cargo.toml: 의존성, features, workspace

패키지 매니페스트(`rust_solution/Cargo.toml`)는 이렇게 생겼습니다.

```toml
[package]
name = "ch10_modules_testing_solution"
version = "0.1.0"
edition.workspace = true      # 루트 Cargo.toml 의 값을 상속
publish = false               # crates.io 에 올리지 않음
```

일반적인 Cargo.toml 에는 다음도 들어갑니다 (이 챕터에서는 쓰지 않지만 AI 프로젝트에서 항상 보게 돼요).

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }   # 외부 크레이트 + 기능 선택
tokio = { workspace = true }                         # 루트에서 버전을 한 번만 정하고 가져다 쓰기

[dev-dependencies]            # 테스트/예제에서만 쓰는 의존성 (배포물에는 안 들어감)
tempfile = "3"

[features]                    # 선택 기능 (C++ 의 #ifdef / 빌드 옵션과 비슷)
default = []
verbose = []                  # cargo build --features verbose  →  코드에서는 #[cfg(feature = "verbose")]
```

- **의존성 버전** `"1"` 은 "1.x.y 중 호환되는 최신"(SemVer). `Cargo.lock` 이 실제 고른 버전을 고정합니다 (실행 파일 프로젝트는 커밋하세요).
- **features** 는 크레이트가 제공하는 선택 기능 스위치입니다. `reqwest = { features = ["json"] }` 처럼 필요한 기능만 켜서 컴파일 시간을 줄여요.
- 의존성 추가는 `cargo add serde` 로도 됩니다.

#### 워크스페이스: 이 저장소가 바로 그 예

이 저장소 루트의 `Cargo.toml` 은 **패키지가 아니라 workspace** 입니다.

```toml
[workspace]
resolver = "3"
members = ["chapters/*/rust*"]       # chapters/ch01.../rust 와 rust_solution 들을 한꺼번에 멤버로

[workspace.package]
edition = "2024"                     # 각 멤버가 edition.workspace = true 로 가져감

[workspace.dependencies]             # 버전은 여기서 한 번만
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
# ...
```

- 멤버 패키지들이 **하나의 `Cargo.lock` 과 하나의 `target/` 을 공유**합니다 (같은 의존성을 한 번만 컴파일).
- 그래서 어느 챕터 폴더에서든 `cargo run -p ch10_modules_testing_solution` 처럼 `-p 패키지이름` 으로 지정해 실행할 수 있어요.
- `cargo test --workspace`, `cargo clippy --workspace` 는 모든 멤버에 적용됩니다.
- ⚙️ C++ 의 CMake `add_subdirectory` 로 묶은 최상위 프로젝트, 🐍 Python 의 monorepo 와 비슷합니다.
- `members` 에 `rust*` 와일드카드를 쓴 덕분에 `rust/` 와 `rust_solution/` 이 모두 잡힙니다.

### 6. 테스트 3종 세트

#### (1) 단위 테스트 — 코드 옆에, 같은 파일 안에

```rust
pub fn format_cents(cents: i64) -> String { /* ... */ }

#[cfg(test)]                 // cargo test 일 때만 컴파일됨 (릴리스 바이너리에 안 들어감)
mod tests {
    use super::*;            // 부모 모듈(위 코드)을 가져옴 — 비공개 함수도 OK

    #[test]
    fn format_cents_cases() {
        assert_eq!(format_cents(150050), "1500.50");
        assert_eq!(format_cents(-5), "-0.05");
    }

    #[test]
    #[should_panic(expected = "overflow")]   // 패닉이 나야 통과
    fn panics_example() { /* ... */ }
}
```

- 주요 매크로: `assert!`, `assert_eq!`, `assert_ne!` (실패 시 좌/우 값을 출력). 메시지 추가: `assert!(x > 0, "x was {x}")`.
- 테스트가 `Result<(), E>` 를 반환하게 하고 `?` 를 쓸 수도 있습니다.
- 실행 필터: `cargo test parse` (이름에 parse 포함), `cargo test -- --nocapture` (println 출력 보기), `cargo test -- --test-threads=1`.
- ⚙️ C++ 에서는 GoogleTest 를 설치·링크해야 하지만 Rust 는 **언어 + cargo 에 내장**이라 설정이 0 입니다.

#### (2) 통합 테스트 — `tests/` 폴더, 바깥 사용자의 눈으로

```text
rust_solution/
 ├─ src/...
 └─ tests/
     └─ integration.rs      ← 이 파일 하나가 "별도 크레이트"로 컴파일됨
```

- `tests/*.rs` 파일은 각각 **라이브러리를 외부 의존성처럼** `use ch10_..::...` 해서 씁니다. → **`pub` 항목만 보입니다.**
- 즉 "공개 API 만으로 실제 시나리오가 동작하는가"를 검증합니다. (비공개 함수는 단위 테스트가 담당)
- 라이브러리 크레이트가 없고 `main.rs` 만 있는 패키지는 `tests/` 에서 내부 함수를 가져올 수 없습니다 — 그래서 **lib + main 분리**가 중요해요.

#### (3) 문서 테스트 — 설명서 속 예제가 진짜로 돈다

```rust
/// 센트 정수를 `1234.56` 형태 문자열로 바꿉니다.
///
/// ```
/// use ch10_modules_testing_solution::model::format_cents;
/// assert_eq!(format_cents(123456), "1234.56");
/// ```
pub fn format_cents(cents: i64) -> String { /* ... */ }
```

- `///` 는 **문서 주석**(마크다운). 그 안의 ```` ``` ```` 코드 블록은 `cargo test` 때 **컴파일 + 실행**됩니다 → 문서가 낡아서 거짓말하는 일이 줄어요.
- `//!` 는 **감싸는 항목**(크레이트/모듈)의 문서입니다 (lib.rs 맨 위).
- `cargo doc --open -p ch10_modules_testing_solution --no-deps` 로 HTML 문서를 만들어 브라우저로 봅니다 (docs.rs 와 같은 모양).
- ⚙️ Doxygen 은 예제 코드가 컴파일되는지 검사하지 않고, 🐍 Python doctest 와 가장 비슷합니다.

```text
$ cargo test -p ch10_modules_testing_solution
running 10 tests ... ok            ← 단위 테스트 (src/ 안의 #[cfg(test)])
running 4 tests  ... ok            ← 통합 테스트 (tests/integration.rs)
Doc-tests ch10_modules_testing_solution
running 2 tests  ... ok            ← 문서 테스트
```

### 7. 일상 워크플로 명령어

| 명령 | 하는 일 |
|---|---|
| `cargo check` | 컴파일만 검사 (빠름) |
| `cargo test` / `cargo test -p 이름` | 단위 + 통합 + 문서 테스트 |
| `cargo clippy --all-targets -- -D warnings` | 린트 (테스트 코드까지, 경고 = 에러) |
| `cargo fmt` | 코드 자동 정렬 (`cargo fmt --check` 는 CI 용) |
| `cargo doc --open` | 문서 생성 |
| `cargo tree` | 의존성 트리 |

## 🧯 자주 만나는 컴파일 에러

(아래는 이 환경의 `rustc`/`cargo` 로 직접 얻은 실제 출력입니다.)

### 1) E0603 — 비공개 항목에 접근했어요

```text
error[E0603]: function `parse_date` is private
 --> tests/integration.rs:62:49
   |
62 | fn _t(){ ch10_modules_testing_solution::parser::parse_date("x"); }
   |                                                 ^^^^^^^^^^ private function
note: the function `parse_date` is defined here
 --> src/parser.rs:36:1
```

- **의미**: `pub` 이 없는 함수는 정의된 모듈(과 자식)에서만 보입니다. 통합 테스트는 "바깥"이라 못 써요.
- **고치기**: 정말 공개 API 면 `pub` 을 붙이고, 아니면 **단위 테스트(같은 파일 `mod tests`)** 에서 시험하세요.

### 2) E0624 — `pub(crate)` 메서드를 다른 크레이트에서 불렀어요

```text
error[E0624]: method `month_key` is private
  --> src/main.rs:12:22
   |
12 |     let key = txs[0].month_key();
   |                      ^^^^^^^^^ private method
   |
  ::: src/model.rs:54:5
54 |     pub(crate) fn month_key(&self) -> (u16, u8) {
   |     ------------------------------------------- private method defined here
```

- **의미**: `main.rs` 는 라이브러리와 **다른 크레이트**입니다. `pub(crate)` 는 라이브러리 안에서만 열려 있어요.

### 3) E0432 — `mod` 선언을 빼먹어서 경로를 못 찾아요

```text
error[E0432]: unresolved import `ch10_modules_testing_solution::report`
 --> src/main.rs:7:45
```

- **의미**: `report.rs` 파일은 있지만 lib.rs 에 `pub mod report;` 가 없어서 모듈이 존재하지 않습니다.
- **고치기**: lib.rs 에 `pub mod report;` 추가. (AI 가 새 파일을 만들고 `mod` 선언을 빠뜨리는 일이 종종 있어요.)

### 4) E0433 — 모듈/크레이트 이름을 찾을 수 없어요

```text
error[E0433]: cannot find module or crate `report` in this scope
 --> o.rs:1:21
  |
1 | fn main() { let m = report::monthly_totals(); }
  |                     ^^^^^^ use of unresolved module or unlinked crate `report`
  = help: you might be missing a crate named `report`
```

- **고치기**: `use` 로 가져오거나(`use ch10_..::report;`), 전체 경로(`crate::report::...`)를 쓰세요. 외부 크레이트면 `Cargo.toml` 의 `[dependencies]` 에 추가.

### 5) E0583 — `mod` 가 가리키는 파일이 없어요

```text
error[E0583]: file not found for module `missing`
 --> main.rs:1:1
  |
1 | mod missing;
  | ^^^^^^^^^^^^
  = help: to create the module `missing`, create file "missing.rs" or "missing/mod.rs"
```

## 🛠️ 실습: 가계부 라이브러리

### 무엇을 만드나요

`data/ledger.csv` (헤더 + 거래 줄, 일부러 잘못된 줄 4개 포함)를 읽어 거래를 파싱하고, 월별 수입/지출/순이익과 카테고리별 지출을 출력합니다.
금액은 **센트 단위 정수**로 다루고 출력할 때만 `1234.56` 꼴로 바꿉니다 (돈에 `f64` 를 쓰지 않는 이유 — Ch12 에서도 다시 나와요).

예상 출력:

```text
loaded 11 transactions, 4 errors
line 6: unknown category 'gift'
line 11: invalid amount 'abc'
line 13: invalid date '2024-13-01'
line 16: expected 4 fields, found 2
== monthly totals ==
2024-01  income   3000.00  expense   1270.50  net   1729.50
2024-02  income   3010.00  expense   1212.75  net   1797.25
2024-03  income   3000.00  expense     60.30  net   2939.70
== expenses by category ==
food          118.55
rent         2400.00
transport      25.00
```

### 파일 구성

```text
ch10_modules_testing/
 ├─ data/ledger.csv
 ├─ python/                       🐍 패키지 구조
 │   ├─ main.py                      (진입점)
 │   └─ ledger/ __init__.py model.py parser.py report.py
 ├─ cpp/                          ⚙️ 헤더 + 소스 (scripts/compare.sh 가 cpp/*.cpp 를 함께 컴파일)
 │   ├─ main.cpp  model.{hpp,cpp}  parser.{hpp,cpp}  report.{hpp,cpp}
 ├─ rust/                         🦀 실습 (여기를 채웁니다)  패키지 ch10_modules_testing
 │   ├─ Cargo.toml
 │   ├─ src/ lib.rs  main.rs  model.rs  parser.rs  report.rs
 │   └─ tests/integration.rs
 └─ rust_solution/                🦀 정답  패키지 ch10_modules_testing_solution (같은 구조)
```

### 실행 방법

```bash
cd chapters/ch10_modules_testing
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/*.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch10_modules_testing                   # 내 풀이 실행
cargo test -p ch10_modules_testing                     # 내 풀이 테스트 (단위 + 통합 + 문서)
cargo run -q -p ch10_modules_testing_solution          # 정답 실행
cargo test -p ch10_modules_testing_solution            # 정답 테스트
cargo doc --no-deps -p ch10_modules_testing_solution --open   # 문서 보기
../../scripts/compare.sh ch10                          # 세 언어 비교 (내 풀이)
../../scripts/compare.sh ch10 --solution               # 정답 비교
```

### TODO 목록

| 번호 | 파일 | 내용 | 관련 개념 |
|---|---|---|---|
| 1 | `rust/src/model.rs` | `Category::parse` | enum + `match` 문자열, `Option` |
| 2 | `rust/src/model.rs` | `Transaction::is_income` | `pub` 메서드 |
| 3 | `rust/src/model.rs` | `format_cents` | 정수로 돈 다루기, **문서 테스트** |
| 4 | `rust/src/parser.rs` | `parse_date` (비공개 fn) | 비공개 함수, 단위 테스트에서 접근 |
| 5 | `rust/src/parser.rs` | `parse_line` | `crate::` 경로, `ok_or_else`, `?`, 에러 enum |
| 6 | `rust/src/parser.rs` | `parse_ledger` | 부분 성공 설계, `enumerate` |
| 7 | `rust/src/report.rs` | `monthly_totals` | `pub(crate)` 사용, `entry().or_default()` |
| 8 | `rust/src/report.rs` | `expense_by_category` | `BTreeMap`, `&'static str` 키 |
| 9 | `rust/tests/integration.rs` | **통합 테스트 직접 작성** | `tests/` 와 공개 API |
| 10 | `rust/src/report.rs` | **단위 테스트 직접 작성** | `#[cfg(test)]`, `assert_eq!` |

> 권장 순서: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10. 한 TODO 를 풀 때마다 `cargo test -p ch10_modules_testing model` 처럼 **이름 필터**를 써 보세요.
> (`lib.rs` 맨 위 `//!` 설명과 각 파일 위쪽 가이드도 같이 읽으세요.)

### 완료 기준

- `cargo test -p ch10_modules_testing` 전부 통과 (단위 10 + 통합 4 + 문서 2)
- `../../scripts/compare.sh ch10` 의 세 언어 출력 동일
- (보너스) `cargo clippy -p ch10_modules_testing --all-targets -- -D warnings` 통과

### 🔬 실험 목록

1. **`pub(crate)` 의 벽**: `main.rs` 에서 `txs[0].month_key()` 호출 → `E0624`. 같은 크레이트(`report.rs`)에서는 되는 이유를 설명해 보세요.
2. **`mod` 선언 빼기**: `lib.rs` 의 `pub mod report;` 삭제 → `E0432`. "파일은 있는데 왜?"
3. **`tests/` 에서 비공개 함수 호출**: `parser::parse_date` 호출 → `E0603`. 단위 테스트에서는 왜 되는지?
4. **`pub use` 연습**: `lib.rs` 의 `pub use model::{Category, Transaction};` 줄을 지워 보고 `tests/integration.rs` 의 `category_is_reexported_at_crate_root` 테스트가 어떻게 되는지 관찰하세요.

## 🤖 바이브 코딩 포인트

- **새 파일을 만들었다고 해서 끝이 아닙니다.** AI 가 `foo.rs` 를 추가하고 `mod foo;` 를 빠뜨리면 컴파일 에러 또는 "안 쓰이는 코드"가 됩니다. 항상 `lib.rs`/`main.rs` 의 `mod` 목록을 확인하세요.
- **`pub` 을 남발하는지** 보세요. 처음엔 모두 비공개로 두고 필요한 만큼만 `pub`/`pub(crate)` 로 여는 것이 좋습니다. "에러 나서 일단 pub" 은 나쁜 신호.
- **`main.rs` 에 로직이 수백 줄**이면 "lib.rs 로 로직을 옮기고 main.rs 는 인자 파싱과 출력만 하게 나눠 줘, 각 모듈에 단위 테스트도 넣어 줘"라고 요청하세요.
- **테스트 요청 프롬프트**: "공개 함수마다 정상/경계/에러 케이스 단위 테스트를 `#[cfg(test)] mod tests` 에, 전체 흐름은 `tests/` 통합 테스트에, 핵심 함수는 `///` doc test 예제로 넣어 줘."
- **`cargo test` 가 통과했다고 안심하지 마세요.** AI 가 테스트를 약하게 쓰거나 `#[ignore]`, `assert!(true)` 로 우회했는지 확인하세요. 에러 케이스와 경계값 테스트가 있는지 보세요.
- **Cargo.toml 변경 점검**: AI 가 추가한 의존성이 정말 필요한지, `features` 가 과도하지 않은지, 워크스페이스라면 `{ workspace = true }` 를 쓰는지 확인합니다 (버전이 여기저기 흩어지면 충돌의 원인).

## ✅ 셀프 체크

**Q1.** 패키지, 크레이트, 모듈의 차이는?

<details><summary>답</summary>

패키지는 `Cargo.toml` 이 있는 빌드 단위(lib 크레이트 최대 1개 + bin 크레이트 여러 개를 가질 수 있음), 크레이트는 rustc 가 한 번에 컴파일하는 단위(라이브러리 또는 실행 파일), 모듈은 크레이트 안의 이름 공간 트리입니다.
</details>

**Q2.** `src/utils.rs` 를 만들었는데 `utils::helper()` 가 "unresolved" 라고 합니다. 무엇을 빠뜨렸나요?

<details><summary>답</summary>

`lib.rs`(또는 `main.rs`) 에 `mod utils;` (또는 `pub mod utils;`) 선언을 빠뜨렸습니다. 파일만 있다고 모듈이 되지 않아요. 그리고 함수에 `pub` 이 필요한지도 확인하세요.
</details>

**Q3.** `pub(crate)` 항목은 `tests/integration.rs` 에서 쓸 수 있나요? 이유는?

<details><summary>답</summary>

쓸 수 없습니다. `tests/` 의 각 파일은 라이브러리와 별개의 크레이트로 컴파일되어 라이브러리를 외부 사용자처럼 쓰기 때문에 `pub` 만 보입니다.
</details>

**Q4.** 비공개 함수를 테스트하고 싶습니다. 어디에 테스트를 두어야 하나요?

<details><summary>답</summary>

같은 파일 안의 `#[cfg(test)] mod tests { use super::*; ... }` 입니다. 자식 모듈은 부모의 비공개 항목에 접근할 수 있어요. (다만 "비공개 함수를 직접 테스트해야만 하나?"를 먼저 생각해 보세요. 공개 API 로 충분히 검증되는 경우가 많습니다.)
</details>

**Q5.** doc test 는 일반 단위 테스트와 어떤 점이 다른가요?

<details><summary>답</summary>

`///` 문서 주석의 코드 블록이 별도 프로그램으로 컴파일·실행되어, 공개 API 만 사용합니다(외부 사용자 입장). 설명서의 예제가 항상 올바르게 동작하는지 보장하는 용도입니다.
</details>

**Q6.** 이 저장소에서 `cargo run -p ch10_modules_testing_solution` 을 어느 폴더에서나 실행할 수 있는 이유는?

<details><summary>답</summary>

루트 `Cargo.toml` 이 workspace 이고 `members = ["chapters/*/rust*"]` 로 모든 패키지를 등록했기 때문입니다. cargo 는 현재 위치에서 위로 올라가며 workspace 루트를 찾고, `-p` 로 멤버를 고를 수 있어요.
</details>

## 📚 더 알아보기

- The Rust Book Ch7 (패키지·크레이트·모듈): <https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html>
- The Rust Book Ch11 (테스트 작성): <https://doc.rust-lang.org/book/ch11-00-testing.html>
- The Rust Book Ch14 (Cargo 와 workspace): <https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html>
- The Cargo Book — Workspaces: <https://doc.rust-lang.org/cargo/reference/workspaces.html>, Features: <https://doc.rust-lang.org/cargo/reference/features.html>
- Rustdoc Book (문서 테스트): <https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html>
- Rust by Example — Modules: <https://doc.rust-lang.org/rust-by-example/mod.html>
