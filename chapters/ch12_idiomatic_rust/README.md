# Ch12. Rust다운 코드 (관용구, Idioms)

> "컴파일은 되는데 Rust 같지 않은 코드"를 알아보고 고치는 장 — From/TryFrom, Display, Default, 빌더, 뉴타입, 이터레이터, Option 콤비네이터, clippy  ·  예상 소요: 이론 45분 / 실습 60~90분  ·  선행: Ch05~Ch08

## 🎯 학습 목표

- C++/Python 습관으로 쓴 Rust 코드를 **관용적인(idiomatic) Rust** 로 바꿀 수 있다.
- 파라미터 타입(`&[T]`, `&str`, `impl Into<String>`, `impl AsRef<..>`)을 상황에 맞게 고른다.
- `From` / `TryFrom` / `Display` / `Default` 를 구현하고, 빌더 + 검증 패턴으로 "항상 유효한 객체"를 만든다.
- `rustfmt` 와 `clippy` 를 개발 루프에 넣고, 자주 나오는 clippy 경고의 의미를 안다.
- AI 가 생성한 Rust 코드를 보고 "컴파일은 되지만 Rust 답지 않은 부분"을 짚어서 고치도록 요청할 수 있다.

## 🔁 C++ / Python 과 한눈에 비교

| 하고 싶은 일 | C++20 | Python | Rust (관용구) |
|---|---|---|---|
| 읽기 전용 배열 인자 | `std::span<const T>` / `const vector<T>&` | `Sequence[T]` | `&[T]` |
| 읽기 전용 문자열 인자 | `std::string_view` | `str` | `&str` |
| 문자열을 "저장"할 인자 | `std::string` (값, move) | `str` | `impl Into<String>` |
| 변환 | 변환 생성자 / `explicit operator` | `int(x)`, `str(x)` | `From` / `Into`, 실패 가능하면 `TryFrom` |
| 출력 형식 | `operator<<` / `std::formatter` | `__str__` | `impl fmt::Display` |
| 기본값 | 멤버 초기화 / 기본 생성자 | dataclass 기본값 | `impl Default` / `#[derive(Default)]` |
| 많은 인자의 생성 | 지정 초기화자(C++20) / 빌더 | 키워드 인자 | 빌더 / `..Default::default()` |
| "검증된 값" | 클래스 불변식 + 생성자에서 throw | `__post_init__` | 뉴타입 + `TryFrom` |
| 루프 변환 | `std::ranges` 파이프 | 컴프리헨션 | `iter().filter().map().collect()` |
| 정적 분석 | clang-tidy | ruff / pylint | `cargo clippy` |
| 포맷터 | clang-format | black | `cargo fmt` (rustfmt) |

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있습니다.

## 📘 이론

### 0. 이 장의 사용법: "고치기" 학습

이번 실습은 새 프로그램을 만드는 게 아니라 **이미 동작하는 프로그램을 고치는** 방식입니다. 테스트는 처음부터 초록불이고, `cargo clippy -- -D warnings` 는 빨간불이에요. 여러분이 할 일은 **테스트를 계속 초록으로 유지하면서 clippy 를 초록으로 만드는 것**. 실무에서 AI 코드를 리뷰하고 개선하는 일과 똑같습니다.

> 💡 **쉬운 설명 — 관용구(idiom)란?** 문법상 맞는 방법이 여러 개일 때, 그 언어 사용자들이 "이게 자연스럽다"고 합의한 표현입니다. 한국어로 치면 "밥을 먹다" vs "밥을 섭취하다". 둘 다 맞지만 하나가 더 자연스럽죠. 컴파일러는 둘 다 통과시키지만 동료(와 clippy)는 구분합니다.

### 1. 파라미터는 "가장 넓게 받는" 타입으로

```rust
// ❌ C++ 의 const vector<string>& 를 그대로 옮긴 모양
fn join_tools(tools: &Vec<String>) -> String { /* ... */ }
fn find_tool(tools: &Vec<String>, name: &String) -> Option<usize> { /* ... */ }

// ✅ 슬라이스와 &str
fn join_tools(tools: &[String]) -> String { tools.join(", ") }
fn find_tool(tools: &[String], name: &str) -> Option<usize> { tools.iter().position(|t| t == name) }
```

- `&[T]` 는 `Vec`, 배열, `&v[1..3]` 모두 받습니다. `&str` 은 `&String`, 리터럴 `"abc"`, 슬라이스 모두 받습니다.
- 호출하는 쪽은 `join_tools(&my_vec)` 그대로 두면 됩니다. (`&Vec<T>` → `&[T]` 는 Deref 강제 변환)
- 규칙: **읽기만 하면 `&[T]` / `&str`, 소유권이 필요하면 `Vec<T>` / `String` / `impl Into<String>`**.
- clippy 린트 이름: `ptr_arg`.

⚠️ **C++ 개발자가 헷갈리는 점**: C++ 에서 `const std::vector<T>&` 는 매우 흔한 관용구라서 `&Vec<T>` 를 쓰고 싶어집니다. Rust 에서는 `std::span` / `std::string_view` 에 해당하는 `&[T]` / `&str` 이 **언어의 기본값**입니다.

#### `impl Into<String>` 과 `AsRef`

```rust
struct User { name: String }
impl User {
    fn new(name: impl Into<String>) -> Self { Self { name: name.into() } }
}
User::new("kim");                 // &str  -> String 으로 변환되어 저장
User::new(String::from("lee"));   // String -> 그대로 move (복사 없음)

fn print_len(path: impl AsRef<str>) { println!("{}", path.as_ref().len()); }  // 빌리기만 할 때
```
- 저장할 거면 `impl Into<String>` (호출 쪽에서 `.to_string()` 을 안 붙여도 됨), 잠깐 읽기만 하면 `&str` 또는 `impl AsRef<str>`.
- `impl Trait` 인자는 Ch08 의 제네릭 문법의 줄임말입니다. 컴파일 타임에 타입별로 코드가 만들어집니다(단형화).

### 2. `From` / `Into` / `TryFrom` — 변환의 표준 이름

```rust
struct Meters(f64);
impl From<f64> for Meters { fn from(v: f64) -> Self { Meters(v) } }

let m: Meters = 3.5.into();        // From 을 구현하면 Into 는 공짜
let m2 = Meters::from(3.5);

// 실패할 수 있는 변환은 TryFrom
struct Temperature(f64);
impl TryFrom<f64> for Temperature {
    type Error = String;
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        if (0.0..=2.0).contains(&v) { Ok(Temperature(v)) } else { Err(format!("bad temperature {v}")) }
    }
}
let t: Result<Temperature, _> = 3.5.try_into();   // Err
```

- `From<A> for B` 를 구현하면 `A.into()` 가 `B` 로 공짜로 됩니다. (반대로 `Into` 를 직접 구현하지는 마세요.)
- `?` 연산자는 에러에 `From` 변환을 자동 적용합니다 (Ch06). 그래서 `impl From<io::Error> for MyError` 한 줄이 에러 전파를 편하게 만들죠.
- 비교: C++ 의 `explicit` 변환 생성자 ≒ `From`, 하지만 Rust 에는 **암묵적 변환이 없고** 항상 `.into()` / `::from()` 으로 명시합니다.

### 3. `Display` — `to_string()` 을 직접 만들지 마세요

```rust
use std::fmt;
struct Point { x: i32, y: i32 }
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "({}, {})", self.x, self.y) }
}
let p = Point { x: 1, y: 2 };
println!("{p}");                 // (1, 2)
let s: String = p.to_string();   // ToString 은 Display 로부터 자동 구현
```
- `Debug`(`{:?}`, 개발자용, `#[derive(Debug)]`) 와 `Display`(`{}`, 사용자용, 직접 구현)를 구분합니다.
- clippy 린트: `inherent_to_string` (직접 만든 `to_string` 메서드를 발견하면 경고).

### 4. `Default` 와 구조체 갱신 문법

```rust
#[derive(Debug, Default)]                    // 모든 필드가 Default 면 derive 가능
struct Opts { verbose: bool, retries: u8, name: String }

let o = Opts { retries: 3, ..Default::default() };   // 나머지는 기본값
```
기본값이 0/빈 값이 아니면 `impl Default` 를 직접 씁니다. 빌더는 보통 `Default` 위에 쌓아요.

### 5. 빌더 패턴 + 검증

필드가 많고 일부만 지정하고 싶을 때. Rust 에는 키워드 인자가 없어서 아주 흔한 패턴입니다.

```rust
let cfg = AgentConfig::builder("helper")
    .model("claude-sonnet")
    .max_tokens(1024)
    .tool("search")
    .build()?;                  // 검증은 build() 에서 한 번만, 실패하면 Err
```

설계 포인트:
- 설정 메서드는 **`mut self` 를 받고 `Self` 를 돌려줍니다.** 소유권이 체인을 따라 흘러가므로 복사가 없어요. (C++ 은 `Builder&` 를 돌려주는 방식이 흔함)
- `build()` 가 `Result<T, ConfigError>` 를 돌려주면, **`build()` 를 통과한 객체는 항상 유효**합니다. 이후 코드는 재검증이 필요 없어요.
- 이 실습처럼 `impl TryFrom<&str> for AgentConfig` 로 문자열 파싱까지 제공하면 `AgentConfig::try_from(spec)?` 처럼 쓸 수 있습니다.

### 6. 뉴타입 (newtype) — "검증된 값"과 "단위"를 타입으로

```rust
struct UserId(u64);
struct OrderId(u64);
fn cancel(order: OrderId) { /* ... */ }
// cancel(UserId(7));   // ❌ 컴파일 에러: 둘 다 u64 지만 서로 다른 타입
```
- 비용 0 (런타임에 `u64` 와 똑같음). C++ 의 `enum class` / strong typedef 와 비슷한 목적.
- `Temperature(f64)` 처럼 **생성 시 검증**하고 필드를 비공개로 두면 "항상 0.0~2.0" 이 타입으로 보장됩니다.

### 7. 루프 대신 이터레이터 (Ch07 복습)

| 습관(C++/Python) | 관용구 |
|---|---|
| `for i in 0..v.len() { v[i] }` | `for x in &v { ... }` / `for (i, x) in v.iter().enumerate()` |
| 카운터 변수 + `if` 로 세기 | `v.iter().filter(\|x\| ..).count()` |
| 찾으면 `return Some(i)` 루프 | `v.iter().position(\|x\| ..)` / `.find(..)` |
| 합계 누적 변수 | `.iter().map(..).sum::<u32>()` |
| 최대값 추적 변수 + `Option` | `.max_by_key(\|x\| ..)` (→ `Option`) |
| 문자열 `push_str` 반복 + 구분자 처리 | `items.join(", ")` |
| 버블 정렬 직접 구현 | `v.sort()` / `v.sort_by_key(..)` / `v.sort_by(\|a, b\| ..)` |

이터레이터는 경계 검사가 없고(인덱스 실수가 불가능), 컴파일러가 루프로 풀어 최적화해서 보통 직접 쓴 루프만큼 빠릅니다.

### 8. `Option` / `Result` 콤비네이터 (Ch06 복습)

```rust
// ❌ unwrap 로 뒤집기
if c.system_prompt.is_some() { let p = c.system_prompt.as_ref().unwrap(); p.len() } else { 0 }
// ✅
c.system_prompt.as_deref().map_or(0, str::len)

// ❌ match 로 Some/None 을 그대로 옮겨 담기
match c.tools.first() { Some(t) => Some(t.to_uppercase()), None => None }
// ✅
c.tools.first().map(|t| t.to_uppercase())
```
자주 쓰는 것: `map`, `and_then`, `ok_or` / `ok_or_else`, `unwrap_or` / `unwrap_or_else` / `unwrap_or_default`, `map_or`, `as_deref`, `filter`, `?`.

### 9. 반환 타입과 `impl Iterator`

```rust
// 컬렉션을 만들어 돌려주는 대신, 이터레이터 자체를 돌려주면 호출자가 필요한 만큼만 소비합니다.
fn long_names<'a>(names: &'a [String], min: usize) -> impl Iterator<Item = &'a String> + 'a {
    names.iter().filter(move |n| n.len() >= min)
}
```
- 문자열은 **받을 때 `&str`, 새로 만들어 줄 때 `String`**.
- 반환 위치의 `impl Iterator` 는 "구체 타입은 숨기고 이터레이터라는 사실만 약속" (C++ 의 `auto` 반환 + concept 과 비슷).

### 10. 이름 짓기와 모양 (rustfmt 가 정해 줍니다)

| 대상 | 규칙 | 예 |
|---|---|---|
| 함수/변수/모듈/파일 | `snake_case` | `join_tools`, `max_tokens` |
| 타입/트레이트/enum 변종 | `UpperCamelCase` | `AgentConfig`, `MissingField` |
| 상수/static | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES` |
| 변환 메서드 | `as_` (싸고 빌림) / `to_` (비쌈, 새로 만듦) / `into_` (소유권 소비) | `as_str`, `to_string`, `into_bytes` |
| getter | `get_` 접두사 **쓰지 않음** | `name()` (O), `get_name()` (X) |
| bool 반환 | `is_` / `has_` | `is_empty` |

`cargo fmt` 하나로 들여쓰기·줄바꿈 논쟁이 끝납니다. 설정이 거의 없고(`rustfmt.toml` 은 옵션), 팀 전체가 같은 모양을 쓰게 돼요. CI 에서는 `cargo fmt --check`.

### 11. 관용구 카탈로그 (before → after 모음)

| # | Before (C++/Python 스타일) | After (Rust 스타일) | clippy 린트 |
|---|---|---|---|
| 1 | `fn f(v: &Vec<T>)` / `fn f(s: &String)` | `fn f(v: &[T])` / `fn f(s: &str)` | `ptr_arg` |
| 2 | `return x;` (함수 맨 끝) | `x` (세미콜론 없는 마지막 표현식) | `needless_return` |
| 3 | `if v.len() == 0` | `if v.is_empty()` | `len_zero` |
| 4 | `if flag == true` | `if flag` | `bool_comparison` |
| 5 | `Foo { name: name, age: age }` | `Foo { name, age }` | `redundant_field_names` |
| 6 | `x = x + 1;` | `x += 1;` | `assign_op_pattern` |
| 7 | `if a < 1 \|\| a > 10` | `if !(1..=10).contains(&a)` | `manual_range_contains` |
| 8 | `for i in 0..v.len() { use v[i] }` | `for x in &v` / `.iter().enumerate()` | `needless_range_loop` |
| 9 | `match o { Some(x) => Some(f(x)), None => None }` | `o.map(f)` | `manual_map` |
| 10 | `if o.is_some() { o.unwrap() ... }` | `if let Some(x) = o { ... }` / 콤비네이터 | `unnecessary_unwrap` |
| 11 | `fn to_string(&self) -> String` | `impl fmt::Display` | `inherent_to_string` |
| 12 | 인자 8개짜리 `new(..)` | 빌더 / 구조체 인자 | `too_many_arguments` |
| 13 | `let c = x.clone();` (x 가 `Copy`) | `let c = x;` | `clone_on_copy` |
| 14 | `let v = vec![1, 2, 3]; for x in v.iter()` | `for x in [1, 2, 3]` (배열) | `useless_vec` |
| 15 | `Result<T, String>` | `Result<T, MyError>` (enum + `Display` + `Error`) | (스타일, 린트 아님) |
| 16 | `.unwrap()` 남발 | `?` 로 전파, `expect("이유")`, 콤비네이터 | `unwrap_used` (기본 꺼짐) |
| 17 | `let mut v = Vec::new(); for ... { v.push(f(x)) }` | `let v: Vec<_> = it.map(f).collect();` | `manual_map`/`same_item_push` 등 |
| 18 | 값이 바뀌지 않는데 `let mut` | `let` | `unused_mut` (rustc) |
| 19 | 큰 `if/else if` 문자열 비교 사슬 | `match key { "a" => .., "b" => .., _ => .. }` | `comparison_chain` 계열 |
| 20 | 조기 종료 없이 깊게 중첩된 `if` | `let ... else { return ... };`, `?`, `if let ... else` | `collapsible_if` |

### 12. clippy 사용법과 알아 둘 린트

```bash
cargo clippy                          # 경고 보기
cargo clippy -- -D warnings           # 경고를 에러로 (CI 용)
cargo clippy --fix --allow-dirty      # 자동 수정 가능한 것만 (꼭 diff 로 확인!)
cargo clippy --all-targets -- -D warnings   # 테스트/예제까지
```

- 린트는 **그룹**으로 나뉩니다: `correctness`(거의 버그, 기본 deny), `suspicious`, `style`, `complexity`, `perf` (기본 warn), `pedantic` / `nursery` / `restriction` (기본 꺼짐).
- 억제할 때는 **이유를 남기는** 가장 좁은 범위로: `#[allow(clippy::too_many_arguments)] // 외부 API 시그니처라 어쩔 수 없음`.
- `#![allow(...)]` 를 파일 맨 위에 길게 넣는 습관은 AI 가 "경고 없애기" 목적으로 자주 만드는 냄새입니다. 리뷰 때 눈여겨보세요.

| 린트 | 의미 | 왜 중요한가 |
|---|---|---|
| `clippy::needless_borrow` | 이미 참조인데 `&` 를 또 붙임 | 가독성 |
| `clippy::redundant_clone` | 쓰고 버릴 값을 `clone()` | 불필요한 복사 (성능) |
| `clippy::clone_on_copy` | `Copy` 타입에 `clone()` | 의도가 흐려짐 |
| `clippy::needless_range_loop` | 인덱스 루프 | 이터레이터로 바꾸라는 신호 |
| `clippy::manual_map` / `manual_unwrap_or` | 콤비네이터가 있는데 `match` 로 직접 | 간결성 |
| `clippy::unwrap_used` / `expect_used` (restriction) | 라이브러리 코드의 panic 가능성 | 서버/에이전트 코드 신뢰성 |
| `clippy::large_enum_variant` | enum 변종 크기 편차가 큼 | `Box` 로 감싸기 (Ch11) |
| `clippy::await_holding_lock` | 락을 쥔 채 `.await` | 데드락 (Ch13/14) |
| `clippy::let_underscore_future` | 만들고 await 안 한 future | 실행되지 않는 비동기 코드 (Ch14) |
| `clippy::mutex_atomic` | `Mutex<bool/usize>` | `AtomicBool` 등이 더 적합 (Ch13) |
| `clippy::missing_errors_doc` (pedantic) | `Result` 반환 공개 함수에 에러 설명 없음 | 라이브러리 문서화 |

## 🧯 자주 만나는 컴파일 에러

이 장은 "컴파일은 되는" 코드가 주제라서 주로 **clippy 메시지**를 읽는 연습을 합니다. 형태는 같아요.

**1) clippy `ptr_arg`**
```text
error: writing `&Vec` instead of `&[_]` involves a new object where a slice will do
  --> src/main.rs:..
   |
   | pub fn join_tools(tools: &Vec<String>) -> String {
   |                          ^^^^^^^^^^^^ help: change this to: `&[String]`
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg
```
의미: 슬라이스로 받으면 더 많은 호출을 받을 수 있다는 뜻. 고치기: 타입만 바꾸면 보통 호출부는 그대로 컴파일됩니다.

**2) clippy `inherent_to_string`**
```text
error: implementation of inherent method `to_string(&self) -> String` for type `AgentConfig`
   = help: implement trait `Display` for type `AgentConfig` instead
```
고치기: `impl fmt::Display for AgentConfig` 로 옮기면 `to_string()` 은 자동으로 생깁니다.

**3) clippy `too_many_arguments`**
```text
error: this function has too many arguments (8/7)
```
고치기: 빌더 또는 옵션 구조체. (`#[allow]` 로 끄는 것은 마지막 수단)

**4) E0277 — `Display` 를 구현하지 않고 `{}` 로 출력**
```text
error[E0277]: `AgentConfig` doesn't implement `std::fmt::Display`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```
고치기: `impl fmt::Display` 를 만들거나, 개발용이면 `{:?}` + `#[derive(Debug)]`.

**5) E0277 — `?` 가 에러 타입을 변환하지 못함**
```text
error[E0277]: `?` couldn't convert the error to `ConfigError`
  = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```
고치기: `impl From<ParseIntError> for ConfigError`, 또는 `.map_err(|_| ConfigError::InvalidValue { .. })?`.

**6) E0277 — 에러 타입이 `std::error::Error` 가 아니라서 `Box<dyn Error>` 로 못 바꿈**
```text
error[E0277]: `?` couldn't convert the error: `ConfigError: std::error::Error` is not satisfied
  = note: required for `Box<dyn std::error::Error>` to implement `From<ConfigError>`
```
고치기: `impl std::error::Error for ConfigError {}` (Display 와 Debug 가 먼저 구현되어 있어야 해요).

## 🛠️ 실습: AgentConfig 리팩터링

### 무엇을 만드나요

LLM 에이전트 설정(`AgentConfig`)을 만들고 검증하고 파싱하고 리포트로 출력하는 프로그램입니다. `rust/src/main.rs` 는 **일부러 C++/Python 스타일**로 짜여 있어요 (8개 인자 생성자, `&Vec<String>`, 인덱스 루프, 직접 만든 `to_string`, 문자열 에러, `unwrap`…). 처음부터 동작하고 테스트도 통과합니다. 여러분은 TODO 의 순서대로 **동작을 유지한 채** 코드를 Rust 답게 고칩니다.

> 출력은 처음부터 Python/C++ 와 똑같습니다. 리팩터링 도중 `compare.sh` 가 깨지면 동작이 바뀌었다는 뜻이에요.

기대 출력(앞부분):
```text
== 1. builder ==
helper | model=claude-sonnet | tokens=1024 | temp=0.70 | retries=2 | timeout=30s | tools=[search, calc] | prompt=28 chars
writer | model=claude-opus | tokens=4096 | temp=0.30 | retries=1 | timeout=60s | tools=[] | prompt=none
coder | model=default-model | tokens=1024 | temp=1.00 | retries=2 | timeout=30s | tools=[shell] | prompt=11 chars
== 2. validation ==
  empty name: error: missing required field: name
  hot temperature: error: temperature 3.5 out of range 0.0..=2.0
...
== 3. parse ==
  [3] error: invalid value for max_tokens: 'abc'
...
== 5. report ==
name    | tokens|  temp
coder   |   1024|  1.00
...
```

### 파일 구성

| 파일 | 설명 |
|---|---|
| `python/main.py` | 같은 프로그램의 Python 버전 (빌더, 예외, f-string) |
| `cpp/main.cpp` | C++20 버전 (`Builder&` 체인, `std::optional`, `std::ranges`) |
| `rust/src/main.rs` | **실습 파일**: 동작하는 비관용적 코드 + 새 API 의 빈칸 (TODO 12개) |
| `rust/examples/solution.rs` | 정답: 관용적인 버전 (clippy 경고 0) |

### 실행 방법 (모두 이 챕터 폴더에서)

```bash
cargo clippy -p ch12_idiomatic_rust -- -D warnings     # 시작: 경고 40여 개가 보입니다. 목표: 0개
cargo test -p ch12_idiomatic_rust                      # 시작: 7개 통과 + 4개 ignored
cargo run -q -p ch12_idiomatic_rust                    # 실습 (처음부터 출력이 완성되어 있음)
cargo run -q -p ch12_idiomatic_rust --example solution # 정답
cargo clippy -p ch12_idiomatic_rust --examples -- -D warnings   # 정답은 깨끗
../../scripts/compare.sh ch12                          # 3개 언어 비교
cargo fmt -p ch12_idiomatic_rust                       # 포맷
```

### TODO 목록

| 번호 | 내용 | 관련 개념 | 잡히는 clippy |
|---|---|---|---|
| 1 | `&Vec<T>` / `&String` 파라미터 → `&[T]` / `&str` | 슬라이스, Deref 강제 변환 | `ptr_arg` |
| 2 | 인덱스 루프 → 이터레이터 (join/count/find/report) | `filter`, `position`, `sum`, `max_by_key`, `sort_by` | `needless_range_loop`, `manual_find` |
| 3 | `is_some()+unwrap()`, `match` 로 감싸기 → 콤비네이터 | `as_deref`, `map_or`, `map` | `unnecessary_unwrap`, `manual_map` |
| 4 | `return x;` / if-else 사슬 → 표현식, `match` | 표현식 기반 언어 | `needless_return` |
| 5 | 직접 만든 `to_string` → `impl Display` | `fmt::Display`, `write!` | `inherent_to_string` |
| 6 | 작은 정리들 | 범위 `contains`, `+=`, 필드 축약, `is_empty` | `manual_range_contains`, `assign_op_pattern`, `redundant_field_names`, `len_zero`, `bool_comparison`, `clone_on_copy` |
| 7 | `String` 에러 → `enum ConfigError` | `Display`, `std::error::Error`, enum 데이터 | |
| 8 | 뉴타입 `Temperature` + `TryFrom<f64>` | 뉴타입, 검증된 값 | |
| 9 | `Default` + `AgentConfigBuilder` | 빌더, `impl Into<String>`, `mut self` 체인 | `too_many_arguments` 해소 |
| 10 | `parse_config` → `TryFrom<&str>` | `split_once`, `?`, `map_err`, 제네릭 `FromStr` | |
| 11 | `main` 을 빌더/`try_from` 으로 이전하고 옛 코드 삭제 | 마이그레이션 | `useless_vec` 등 |
| 12 | `cargo fmt`, `#![allow]` 제거, clippy 0 | rustfmt, 린트 위생 | 전부 |

권장 순서: **1 → 2 → 3 → 4 → 5 → 6** (기존 코드를 고치는 부분, 테스트 계속 통과) → **7 → 8 → 9 → 10** (새 코드를 쓰고 `#[ignore]` 해제) → **11 → 12**.

### 완료 기준
- `cargo test -p ch12_idiomatic_rust` — 11개 모두 통과, ignored 0개
- `cargo clippy -p ch12_idiomatic_rust -- -D warnings` — 경고 0개
- `../../scripts/compare.sh ch12` — Python / C++ / Rust 출력이 똑같음

### 🔬 실험 목록

| # | 해보기 | 관찰 |
|---|---|---|
| 1 | TODO(1) 전에 `join_tools(&["a".to_string()])` (배열) 호출을 main 에 추가 | `&Vec<String>` 이라서 배열은 `E0308 mismatched types`. 슬라이스로 바꾸면 컴파일됨 |
| 2 | TODO(5) 전에 `println!("{}", cfg)` | `E0277 AgentConfig doesn't implement Display`. `Display` 를 구현하면 통과 |
| 3 | `cargo clippy --fix --allow-dirty -p ch12_idiomatic_rust` 를 **복사본에서** 실행 | 자동으로 고쳐지는 린트(`needless_return`, `redundant_field_names` 등)와 안 고쳐지는 린트(`to_string`, `too_many_arguments`) 구분하기 |
| 4 | `Temperature` 정의(와 impl)를 `mod config { ... }` 로 감싸 `pub` 을 붙인 뒤, 밖에서 `config::Temperature(2.5)` 시도 | `E0603 tuple struct constructor is private`. 그래서 `try_from` 이 유일한 입구 (같은 모듈 안에서는 필드가 보이므로 모듈로 감싸야 실험이 됩니다) |
| 5 | 마지막에 `if v.len() == 0 {}` 한 줄을 넣고 clippy 확인 | `len_zero` 경고. 관용구 표와 비교 |

## 🤖 바이브 코딩 포인트

- AI 가 만든 Rust 코드를 받으면 먼저 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` 를 돌리게 하세요. 프롬프트: *"clippy 경고가 0이 될 때까지 고치고, #[allow] 로 숨기지는 마."*
- **`&Vec<T>`, `&String`, `.clone()` 남발, 인덱스 루프, `return x;`** 는 "C++/Python 번역체" 의 대표 냄새입니다. 프롬프트: *"idiomatic Rust 로 리팩터링해 줘: 슬라이스/&str 파라미터, 이터레이터, Option 콤비네이터 사용."*
- **에러가 `String` 이거나 `Box<dyn Error>` 로만 뭉개져** 있으면: 라이브러리 코드는 `enum` 에러(+`thiserror`, Ch16), 앱 코드는 `anyhow`. 프롬프트: *"에러 타입을 enum 으로 만들고 Display/Error 를 구현해."*
- **`.unwrap()` / `.expect()`** 가 많으면 panic 지점입니다. "절대 실패 불가" 고정 입력(main 의 상수 등) 말고는 `?` 로 바꾸게 하세요.
- 필드가 5개 넘는 생성자가 있으면 *"빌더 패턴 + build() 에서 검증, 검증된 값은 뉴타입으로"* 라고 요청하세요. 반대로 필드 2~3개 구조체에 빌더를 만들어 온다면 과합니다. 구조체 리터럴 + `..Default::default()` 면 충분해요.
- 공개 함수 파라미터가 `String` 인데 내부에서 읽기만 한다면 `&str` 로, 저장한다면 `impl Into<String>` 로 바꾸도록 요청하세요.
- `#![allow(clippy::all)]`, `#![allow(warnings)]` 같은 전역 억제가 생겼다면 거부하세요.

## ✅ 셀프 체크

1. `fn f(names: &Vec<String>)` 대신 `fn f(names: &[String])` 이 좋은 이유 두 가지는?
<details><summary>답</summary>(1) 배열, 슬라이스(`&v[1..3]`) 등 더 많은 입력을 받을 수 있습니다. (2) 불필요한 간접 참조(`&Vec` 의 포인터를 따라 또 포인터)가 없습니다. 호출부는 대개 그대로 컴파일됩니다.</details>

2. `impl fmt::Display for T` 만 구현했는데 `t.to_string()` 이 되는 이유는?
<details><summary>답</summary>표준 라이브러리가 "`Display` 를 구현한 모든 타입에 `ToString` 을 자동 구현"(blanket impl)해 두었기 때문입니다.</details>

3. 빌더의 설정 메서드가 `&mut self` 가 아니라 `mut self` 를 받고 `Self` 를 돌려주는 이유는?
<details><summary>답</summary>소유권을 체인으로 넘기면 `AgentConfig::builder("a").model("m").build()` 처럼 한 문장으로 쓸 수 있고, 임시값(temporary)에서도 복사 없이 끝까지 이어집니다. `&mut self` 방식은 중간 변수에 담아야 하는 경우가 많아요.</details>

4. 뉴타입 `Temperature(f64)` 의 필드를 비공개로 두고 `TryFrom<f64>` 만 제공하면 어떤 이득이 있나요?
<details><summary>답</summary>`Temperature` 값이 존재한다는 사실 자체가 "범위 검증을 통과했다"는 증거가 됩니다. 이후 코드가 다시 검사하지 않아도 되고, 검증 안 된 `f64` 를 실수로 넘기면 컴파일 에러가 납니다.</details>

5. 코드 리뷰에서 `#![allow(warnings)]` 가 파일 맨 위에 있으면 왜 먼저 지우게 하나요?
<details><summary>답</summary>컴파일러와 clippy 가 알려 주는 다른 모든 문제(안 쓰는 코드, `unwrap` 오용, 비효율 등)를 가려 버리기 때문입니다. 먼저 지운 뒤 나오는 경고를 하나씩 고쳐야 나머지 리뷰(`Rc<RefCell>` 과용, `unwrap` 남발 등)가 의미 있어요.</details>

## 📚 더 알아보기

- The Rust Book: 관용적 에러/트레이트 — Ch09(에러), Ch10(트레이트), Ch13(이터레이터·클로저), Ch17(OOP 패턴)  <https://doc.rust-lang.org/book/>
- Rust API Guidelines (네이밍, 변환, 트레이트 구현 체크리스트): <https://rust-lang.github.io/api-guidelines/>
- Rust by Example — `From`/`Into`, `TryFrom`: <https://doc.rust-lang.org/rust-by-example/conversion.html>
- Clippy 린트 목록(검색 가능): <https://rust-lang.github.io/rust-clippy/master/>
- std 문서: [`std::fmt`](https://doc.rust-lang.org/std/fmt/), [`std::convert`](https://doc.rust-lang.org/std/convert/), [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html)
- 다음 장: Ch13 동시성
