# Ch08. 트레이트와 제네릭

> "서로 다른 타입을 같은 방식으로 다루는 법" — Rust 설계의 중심 도구  ·  예상 소요: 이론 50분 / 실습 70분  ·  선행: Ch07

이 챕터의 실습은 마지막 Ch17(미니 AI 에이전트)에서 그대로 다시 만나게 될 **`Tool` 트레이트와 `ToolRegistry`** 입니다.
지금은 동기(sync) 버전으로 만들고, Ch17 에서는 같은 이름의 `async` 버전으로 확장합니다.

## 🎯 학습 목표

- `trait` 로 "공통 동작"을 정의하고, 기본 메서드(default method)와 함께 구현할 수 있습니다.
- 제네릭 함수에 **트레이트 바운드**(`T: PartialOrd + Copy`, `where`, `impl Trait`)를 걸 수 있습니다.
- **정적 디스패치**(`<T: Tool>`, 단형화)와 **동적 디스패치**(`&dyn Tool`, `Box<dyn Tool>`, vtable)를 구분해서 고를 수 있습니다.
- `Display`, `From`, `PartialOrd` 같은 표준 트레이트를 직접 구현하거나 바운드로 요구할 수 있습니다.
- AI 가 만든 코드의 `impl<T: ...> ... where ...`, `Box<dyn Trait + Send>` 같은 시그니처를 읽을 수 있습니다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| "이런 메서드를 가진 타입" 약속 | 추상 클래스(순수 가상 함수) / `concept` | `typing.Protocol`, ABC, 덕 타이핑 | `trait` |
| 구현 선언 | `class A : public Tool` (**클래스 안에서**) | 그냥 메서드를 만들면 끝 | `impl Tool for A` (**타입 밖에서 따로**) |
| 기본 구현 | `virtual` 함수 본문 | Protocol/ABC 의 일반 메서드 | trait 안의 본문 있는 `fn` |
| 제네릭 | `template <typename T>` | 타입 힌트 `TypeVar` (실행 시 검사 없음) | `fn f<T: Bound>` |
| 제네릭 제약 | `requires` / `concept` | (없음, 런타임 `TypeError`) | 트레이트 바운드 `T: A + B`, `where` |
| 정적 디스패치 | 템플릿 인스턴스화 | (없음) | 단형화 (`<T: Tool>`, `impl Tool`) |
| 동적 디스패치 | `virtual` + vtable, `unique_ptr<Base>` | 항상 동적 | `dyn Tool`, `Box<dyn Tool>` |
| 상속 | 클래스 상속 (데이터 + 동작) | 클래스 상속 | **없음** (동작만 trait 로 공유, 데이터는 상속 안 함) |
| 연산자/출력 | `operator<<`, `operator==` | `__str__`, `__eq__` | `Display`, `PartialEq` 등의 trait |
| 변환 | 변환 생성자, `explicit` | `int("3")`, 생성자 | `From` / `Into` / `TryFrom` |
| 오류 검사 시점 | 템플릿 **사용할 때**(늦음, 길고 어려운 에러) | 실행할 때 | 제네릭 **선언만 보고**(이른 검사, 짧은 에러) |

## 📘 이론

> 이 챕터의 코드 조각은 std 만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있습니다.
> (`fn main() { ... }` 를 감싸 주세요.)

### 1. 트레이트(trait) 란?

트레이트는 "이 타입은 이런 일을 할 수 있어요"라는 **약속서**입니다.

```rust
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Result<String, String>;
}

struct Reverser;

impl Tool for Reverser {            // "Reverser 는 Tool 약속을 지킵니다"
    fn name(&self) -> &str { "reverser" }
    fn description(&self) -> &str { "reverses text" }
    fn run(&self, input: &str) -> Result<String, String> {
        Ok(input.chars().rev().collect())
    }
}
```

> 💡 **쉬운 설명**: 식당 주방 규칙을 생각해 보세요. "주문을 받으면(`run`) 결과를 내놓는다"는 규칙이 trait,
> 한식 요리사·양식 요리사가 각자 방식으로 규칙을 지키는 것이 `impl` 입니다.
> 홀 직원(호출하는 코드)은 요리사가 누구인지 몰라도 규칙만 알면 주문을 넣을 수 있어요.

**기본 메서드**: trait 안에 본문을 적으면 구현하는 쪽이 생략할 수 있습니다.

```rust
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn describe(&self) -> String {                 // 기본 구현
        format!("{}: {}", self.name(), self.description())
    }
}
```

⚠️ **C++ 개발자가 헷갈리는 점**

- Rust 에는 **클래스 상속이 없습니다.** trait 는 "동작의 약속"만 담고, 데이터 필드는 물려주지 못해요.
  "공통 필드가 필요하다" → 구조체 안에 다른 구조체를 **멤버로 포함**(컴포지션)합니다.
- `impl Trait for Type` 은 타입을 정의한 곳과 **다른 곳**에서 할 수 있습니다. (단, 트레이트나 타입 둘 중 하나는 내 크레이트 것이어야 해요 — "고아 규칙".)
- `virtual ~Tool() = default;` 같은 소멸자 의식이 필요 없습니다. `Box<dyn Tool>` 이 해제될 때 실제 타입의 `Drop` 이 알아서 불립니다.

### 2. 제네릭과 트레이트 바운드

```rust
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter().copied();
    let first = iter.next()?;
    Some(iter.fold(first, |best, x| if x > best { x } else { best }))
}
```

- `T` 는 "아무 타입"이지만, `: PartialOrd + Copy` 로 **"비교 가능하고 복사 가능한 타입만"** 으로 좁힙니다.
- 바운드가 길어지면 `where` 로 옮겨 적습니다. 같은 뜻이에요.

```rust
fn largest<T>(items: &[T]) -> Option<T>
where
    T: PartialOrd + Copy,
{ /* ... */ }
```

- 인자 자리에서만 쓰는 간단한 경우 `impl Trait` 문법이 짧습니다: `fn run_it(tool: &impl Tool)` ≡ `fn run_it<T: Tool>(tool: &T)`.

C++20 과 대응하면 이렇습니다.

```cpp
template <typename T>
    requires std::totally_ordered<T> && std::copyable<T>
std::optional<T> largest(const std::vector<T>& items);
```

⚠️ **C++ 개발자가 헷갈리는 점**

- C++ 템플릿(C++20 이전)은 "일단 펼쳐 보고 안 되면 에러"라서 에러가 수십 줄입니다. concept 가 이를 개선했지만 **선택 사항**이죠.
  Rust 는 **바운드가 필수**입니다. `T` 에 `>` 를 쓰려면 반드시 `T: PartialOrd` 라고 선언해야 해요. 대신 에러가 짧고 정확합니다(아래 🧯 참고).
- 왜 `Ord` 가 아니라 `PartialOrd` 일까요? `f64` 는 `NaN` 이 있어서 "모든 값을 순서 지을 수 있는" `Ord` 가 아니기 때문입니다.
  (`f64` 는 `PartialOrd` 만 구현.) `largest(&[1.5, 3.5])` 가 되려면 `PartialOrd` 여야 해요.

### 3. 정적 디스패치 vs 동적 디스패치

같은 trait 를 "어떻게 호출하느냐"에 따라 두 방식이 있습니다.

```rust
// (1) 정적 디스패치: 컴파일 때 타입이 정해짐. T 마다 함수가 따로 생김(단형화, monomorphization)
fn run_static<T: Tool>(tool: &T, input: &str) -> String { /* ... */ }

// (2) 동적 디스패치: 실행 중에 vtable 을 보고 호출. 함수는 하나.
fn run_dyn(tool: &dyn Tool, input: &str) -> String { /* ... */ }
```

| | 정적 (`<T: Tool>` / `impl Tool`) | 동적 (`&dyn Tool` / `Box<dyn Tool>`) |
|---|---|---|
| C++ 대응 | 템플릿 + concept | `virtual` + 기반 클래스 포인터/참조 |
| 호출 비용 | 0 (인라인 가능) | vtable 간접 호출 1회 |
| 바이너리 크기 | 타입마다 코드 복제 (커질 수 있음) | 작음 |
| 서로 다른 타입을 한 `Vec` 에? | ❌ 불가 | ✅ `Vec<Box<dyn Tool>>` |
| 타입 정보 | 컴파일 때 앎 | 실행 때만 앎 |

`&dyn Tool` 은 **포인터 2개**짜리 "뚱뚱한 포인터(fat pointer)"입니다: (데이터 주소, vtable 주소).
C++ 은 객체 **안에** vptr 이 숨어 있지만, Rust 는 객체에는 아무것도 안 넣고 **참조 쪽에** vtable 포인터를 붙여 다닙니다.
그래서 `struct Reverser;` 같은 값 자체는 평범한 구조체 그대로이고, `dyn` 이 필요할 때만 비용을 냅니다.

```rust
struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,   // ⚙️ C++: std::vector<std::unique_ptr<Tool>>
}
```

> 💡 **쉬운 설명**: 정적 = "주문서마다 전담 요리사 매뉴얼을 따로 인쇄"(빠르지만 종이가 늘어남),
> 동적 = "공용 매뉴얼 한 권 + 실행 시 색인표(vtable)로 찾아보기"(약간 느리지만 종이가 하나).
> 서로 다른 요리사를 **한 줄로 세워야** 하면 동적이 유일한 선택입니다.

**고르는 기준**: 기본은 정적(제네릭). "이질적인 것들을 컬렉션에 담거나, 플러그인처럼 런타임에 정해진다" → `dyn`.
AI 에이전트의 도구 목록은 정확히 후자라서 `Box<dyn Tool>` 을 씁니다.

#### 객체 안전(dyn 호환성)

모든 trait 를 `dyn` 으로 쓸 수 있는 것은 아닙니다. vtable 은 "크기가 정해진 함수 포인터 표"라서, 다음 메서드가 있으면 `dyn Trait` 이 안 됩니다.

- `Self` 를 **반환**하는 메서드 (`fn duplicate(&self) -> Self`) — 호출자는 `Self` 의 크기를 모릅니다.
- **제네릭 메서드** (`fn map<U>(&self, ...)`) — vtable 칸을 무한히 만들 수 없어요.

(예전에는 "object safety", 요즘 컴파일러는 "dyn compatible" 이라고 부릅니다. 🔬 실험 2 참고.)
실무 팁: AI 가 `dyn` 에러를 내면 "메서드를 `where Self: Sized` 로 제외" 하거나 "enum 으로 바꾸기"가 흔한 해법입니다.

### 4. 표준 트레이트로 "언어와 어울리기"

Rust 의 연산자와 출력, 변환은 모두 trait 로 연결됩니다. 직접 구현해 두면 언어 전체와 자연스럽게 맞물려요.

```rust
use std::fmt;

struct ToolCall { tool: String, input: String }

impl fmt::Display for ToolCall {                       // println!("{call}") 가능, .to_string() 공짜
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(\"{}\")", self.tool, self.input)
    }
}

impl From<(&str, &str)> for ToolCall {                 // ToolCall::from(..) 와 (..).into() 공짜
    fn from((tool, input): (&str, &str)) -> Self {
        ToolCall { tool: tool.to_string(), input: input.to_string() }
    }
}
```

| trait | C++ 비슷한 것 | Python 비슷한 것 | 얻는 것 |
|---|---|---|---|
| `Display` | `operator<<` | `__str__` | `{}` 출력, `to_string()` |
| `Debug` (`#[derive(Debug)]`) | (없음) | `__repr__` | `{:?}` 출력 |
| `From` / `Into` | 변환 생성자 | 생성자/`int()` | `.into()` |
| `PartialEq` / `Eq` | `operator==` | `__eq__` | `==` |
| `PartialOrd` / `Ord` | `operator<=>` | `__lt__` | `<`, `sort()`, `max()` |
| `Clone` / `Copy` | 복사 생성자 | `copy.copy` | `.clone()` / 암묵 복사 |
| `Default` | 기본 생성자 | 기본 인자 | `T::default()` |

> 💡 `#[derive(...)]` 는 "정해진 방식의 구현을 컴파일러가 대신 써 줘요"라는 뜻입니다. (Ch05 에서 본 것!)

### 5. 에이전트 도구 모델 (실습 미리보기)

```text
 ┌──────────────┐    register(Box::new(..))     ┌─────────────────────────┐
 │ Calculator   │ ─────────────────────────────▶ │ ToolRegistry            │
 │ WordCounter  │                                │ tools: Vec<Box<dyn Tool>>│
 │ Reverser     │                                └───────────┬─────────────┘
 └──────────────┘        call("calculator","12 * 3")        │ 이름으로 찾고 run() = vtable 호출
                                                             ▼
                                                    Ok("36") / Err("...")
```

LLM 이 "calculator 도구를 `12 * 3` 입력으로 써 줘"라고 요청하면, 에이전트는 **이름 → 도구 찾기 → 실행 → 결과 반환**을 합니다.
Ch17 에서는 `run` 이 `async fn` 이 되고(그러려면 몇 가지 추가 지식이 필요해요), 이름 `Tool` / `ToolRegistry` 는 그대로 유지됩니다.

## 🧯 자주 만나는 컴파일 에러

### 1) E0369 — 바운드를 안 걸고 `>` 를 썼어요

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
 --> a.rs:4:14
  |
4 |         if x > best {
  |            - ^ ---- &T
  |            |
  |            &T
  |
help: consider restricting type parameter `T` with trait `PartialOrd`
  |
1 | fn largest_no_bound<T: std::cmp::PartialOrd>(items: &[T]) -> &T {
  |                      ++++++++++++++++++++++
```

- **의미**: `T` 가 뭔지 모르는데 `>` 를 쓸 수 있다고 보장 못 해요. (C++ 템플릿은 인스턴스화 시점에 에러.)
- **고치기**: 컴파일러 제안대로 `T: PartialOrd` 를 붙입니다. 값을 복사해 쓰려면 `Copy` 도.

### 2) E0277 — 그 타입은 트레이트를 구현하지 않았어요

```text
error[E0277]: the trait bound `{integer}: Tool` is not satisfied
 --> d.rs:3:23
  |
3 | fn main(){ run_static(&5); }
  |            ---------- ^^ the trait `Tool` is not implemented for `{integer}`
  |            |
  |            required by a bound introduced by this call
```

- **의미**: `run_static<T: Tool>` 에 `Tool` 을 구현하지 않은 타입(`5`)을 넣었어요.
- **고치기**: 그 타입에 `impl Tool for ...` 를 쓰거나, 올바른 타입을 넘기세요. AI 코드에서 가장 흔한 trait 에러입니다.

### 3) E0038 — 이 트레이트는 `dyn` 으로 쓸 수 없어요 (dyn 호환 아님)

```text
error[E0038]: the trait `Cloner` is not dyn compatible
 --> b.rs:4:15
  |
4 | fn use_it(c: &dyn Cloner) {}
  |               ^^^^^^^^^^ `Cloner` is not dyn compatible
  ...
2 |     fn duplicate(&self) -> Self;
  |                            ^^^^ ...because method `duplicate` references the `Self` type in its return type
```

- **의미**: `Self` 반환 / 제네릭 메서드가 있는 trait 는 vtable 을 못 만들어요.
- **고치기**: 해당 메서드에 `where Self: Sized` 를 붙이거나, `dyn` 대신 제네릭/enum 을 씁니다.

### 4) E0782 — `dyn` 을 빼먹었어요

```text
error[E0782]: expected a type, found a trait
 --> e.rs:3:23
  |
3 | fn main(){ let v: Vec<Tool> = Vec::new(); }
  |                       ^^^^
help: you can add the `dyn` keyword if you want a trait object
  |
3 | fn main(){ let v: Vec<dyn Tool> = Vec::new(); }
  |                       +++
```

- **의미**: trait 이름만으로는 타입이 아니에요. (그리고 `Vec<dyn Tool>` 도 크기를 몰라 안 됩니다.)
- **고치기**: `Vec<Box<dyn Tool>>` — 힙에 두고 `Box` 포인터로 들고 있기.

### 5) E0046 — 필수 메서드를 빼먹었어요

```text
error[E0046]: not all trait items implemented, missing: `description`
 --> c.rs:3:1
  |
3 | impl Tool for R { fn name(&self)->&str {"r"} }
  | ^^^^^^^^^^^^^^^ missing `description` in implementation
```

- **의미**: 기본 구현이 없는 메서드는 모두 구현해야 합니다. (C++ 은 순수 가상 함수를 빼먹으면 "추상 클래스 인스턴스화 불가" 에러.)

## 🛠️ 실습: 에이전트 도구 레지스트리 (동기)

### 무엇을 만드나요

세 가지 도구(`calculator`, `word_counter`, `reverser`)를 만들고 `ToolRegistry` 에 등록한 뒤,
스크립트로 정해 둔 호출 목록(알 수 없는 도구, 잘못된 입력 포함)을 실행합니다. 예상 출력(일부):

```text
== registry ==
tools: calculator, word_counter, reverser
calculator: evaluates 'a op b' on integers (+ - * /)
...
== calls ==
calculator("12 * 3") -> ok: 36
calculator("7 / 0") -> error: division by zero
translator("hi") -> error: unknown tool: translator
== dispatch ==
static : reverser("abc") -> ok: cba
dynamic: reverser("abc") -> ok: cba
== generics ==
largest ints: 42
largest floats: 3.50
```

### 파일 구성

| 언어 | 파일 |
|---|---|
| Python | `python/main.py` (Protocol + 클래스) |
| C++ | `cpp/main.cpp` (추상 클래스 + `unique_ptr`, template + concept) |
| Rust | `rust/src/main.rs` (**여기를 채웁니다**), `rust/examples/solution.rs` (정답) |

### 실행 방법

```bash
cd chapters/ch08_traits_generics
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch08_traits_generics               # 내 풀이
cargo test -p ch08_traits_generics                 # 내 풀이 테스트
cargo run -q -p ch08_traits_generics --example solution   # 정답
../../scripts/compare.sh ch08                      # 세 언어 출력 비교 (내 풀이)
../../scripts/compare.sh ch08 --solution           # 정답 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `Tool::describe` 기본 메서드 | 기본 메서드, `self.name()` 호출 |
| 2 | `Calculator::run` | `impl Trait for`, 슬라이스 패턴, `parse`, `Result` |
| 3 | `WordCounter::run` | `split_whitespace`, `Err` 반환 |
| 4 | `Reverser::run` | `chars().rev().collect()` |
| 5 | `impl Display for ToolCall` | 표준 트레이트, `write!` |
| 6 | `impl From<(&str,&str)> for ToolCall` | `From`/`into`, 튜플 분해 |
| 7 | `ToolRegistry::register` | `Box<dyn Tool>`, 소유권 이동 |
| 8 | `ToolRegistry::names` | 이터레이터, 빌린 `&str` 반환 |
| 9 | `ToolRegistry::call` | 동적 디스패치, `ok_or_else`, `?` |
| 10 | `run_static<T: Tool>` | 정적 디스패치(단형화) |
| 11 | `run_dyn(&dyn Tool)` | 동적 디스패치, fat pointer |
| 12 | `largest<T: PartialOrd + Copy>` | 제네릭 바운드, `Option` + `?` |

### 완료 기준

- `cargo test -p ch08_traits_generics` 전부 통과
- `../../scripts/compare.sh ch08` 에서 Python / C++ / Rust 출력이 동일

### 🔬 실험 목록 (`rust/src/main.rs` 안의 주석 블록)

1. **bound 없는 `largest`** → `E0369`. "왜 C++ 템플릿보다 에러가 짧을까?"
2. **`Self` 를 반환하는 trait 를 `&dyn` 로** → `E0038`. "왜 vtable 에는 `Self` 반환 메서드를 못 넣을까?"
3. **필수 메서드 하나 지우기** → `E0046`.
4. (추가) TODO(11) 의 `&dyn Tool` 을 `&impl Tool` 로 바꿔도 `main` 이 컴파일되는지 보고, 무엇이 정적/동적으로 바뀌는지 설명해 보세요.

## 🤖 바이브 코딩 포인트

AI 가 만든 코드를 리뷰할 때:

- **`Box<dyn Trait>` 가 보이면** 이유를 물어보세요: "이질적인 타입을 섞는 건가, 아니면 그냥 제네릭으로 충분한가?" 불필요한 `dyn` 은 성능·표현력 손해입니다.
- **`dyn Trait + Send + Sync`** 가 붙어 있으면 스레드/async 와 함께 쓰려는 것입니다 (Ch13–14). 빠지면 컴파일 에러가 나요: "future cannot be sent between threads".
- **`impl<T: Display + ?Sized> ...`, `where` 절** 이 길어지면 "이 바운드가 정말 다 필요한가요? 줄여 주세요"라고 요청해도 좋습니다. 과한 바운드는 호출을 어렵게 합니다.
- **`.clone()` 을 남발해 바운드를 피하는지** 보세요. `T: Clone` 대신 참조(`&T`)로 해결되는 경우가 많습니다.
- 프롬프트 팁: "`trait Tool` 로 추상화하고, 레지스트리는 `Vec<Box<dyn Tool>>` 로 해 줘. 도구 입력 오류는 `Result<String, String>` 로 돌려줘 (패닉 금지)."
- `impl Display` 는 사람이 읽는 출력, `Debug` 는 개발자용입니다. 사용자 메시지에 `{:?}` 가 쓰이면 의심해 보세요.

## ✅ 셀프 체크

**Q1.** `Vec<Box<dyn Tool>>` 에서 왜 `Box` 가 필요한가요? `Vec<dyn Tool>` 은 왜 안 되나요?

<details><summary>답</summary>

`dyn Tool` 은 구현 타입마다 크기가 달라서 컴파일 때 크기를 알 수 없는 타입입니다. `Vec` 은 원소 크기가 고정이어야 하므로,
크기가 일정한 포인터(`Box`)를 담고 실제 값은 힙에 둡니다. C++ 의 `vector<unique_ptr<Base>>` 와 같은 이유예요.
</details>

**Q2.** 정적 디스패치와 동적 디스패치의 차이를 한 문장씩 말해 보세요. 에이전트의 도구 목록에 동적을 쓰는 이유는?

<details><summary>답</summary>

정적: 컴파일러가 타입별로 함수를 복제(단형화)해 직접 호출. 동적: vtable 포인터를 통해 실행 중에 호출 대상을 결정.
도구는 계산기/검색/파일 등 서로 다른 타입을 **하나의 목록**에 담아 이름으로 골라 호출해야 하므로 동적 디스패치가 필요합니다.
</details>

**Q3.** `fn largest<T: PartialOrd>(items: &[T]) -> T` 를 `T: Copy` 없이 쓰면 어떤 문제가 생길까요?

<details><summary>답</summary>

슬라이스 안의 값을 밖으로 꺼내 반환하려면 소유권을 가져와야 하는데, 빌린 슬라이스에서는 이동할 수 없어요(E0508 등).
`Copy` 로 복사하거나, `&T` 를 반환하거나, `Clone` 으로 복제해야 합니다.
</details>

**Q4.** `impl Display for ToolCall` 만 썼는데 `call.to_string()` 이 동작하는 이유는?

<details><summary>답</summary>

표준 라이브러리에 `impl<T: Display> ToString for T` 라는 **포괄(blanket) 구현**이 있어서, `Display` 를 구현한 모든 타입이 자동으로 `ToString` 을 얻습니다.
</details>

**Q5.** `fn duplicate(&self) -> Self` 가 있는 trait 로 `&dyn Trait` 를 만들 수 없는 이유는?

<details><summary>답</summary>

`dyn` 에서는 실제 타입을 모르므로 `Self` 의 크기를 알 수 없어서 반환 값을 받을 자리를 만들 수 없기 때문입니다 (dyn 호환 규칙 위반, E0038).
</details>

## 📚 더 알아보기

- The Rust Book Ch10 (제네릭·트레이트): <https://doc.rust-lang.org/book/ch10-00-generics.html>
- The Rust Book Ch18.2 (트레이트 객체): <https://doc.rust-lang.org/book/ch18-02-trait-objects.html>
- Rust by Example — Traits: <https://doc.rust-lang.org/rust-by-example/trait.html>
- std 문서 — `Display`: <https://doc.rust-lang.org/std/fmt/trait.Display.html>, `From`: <https://doc.rust-lang.org/std/convert/trait.From.html>
- Dyn compatibility(구 object safety): <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
