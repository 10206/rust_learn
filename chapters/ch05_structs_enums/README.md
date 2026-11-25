# Ch05. 구조체·열거형·패턴 매칭

> Rust 의 `enum` 은 "데이터를 품은 합 타입(sum type)"이고, `match` 는 빠뜨린 경우를 컴파일러가 잡아 주는 분기문입니다.  ·  예상 소요: 이론 40분 / 실습 60분  ·  선행: Ch04 (참조·빌림·슬라이스)

## 🎯 학습 목표
- `struct` 를 정의하고 `impl` 블록에 메서드(`&self` / `&mut self` / `self`)와 연관 함수(`Type::new()`)를 만든다.
- 데이터를 품은 `enum` 으로 "상태"와 "사건"을 표현하고, `std::variant` / Python 클래스 계층과의 차이를 설명한다.
- `match` 의 **exhaustive(빠짐없음) 검사**, 가드(`if`), 튜플 패턴, `|` 패턴을 쓴다.
- `Option<T>` 로 "값이 없을 수 있음"을 표현하고(null 이 없다!), `if let` / `let else` 로 꺼낸다.
- `#[derive(Debug, Clone, PartialEq)]` 가 무엇을 해 주는지 안다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 데이터 묶음 | `struct S { int x; };` | `@dataclass class S: x: int` | `struct S { x: i32 }` |
| 메서드 | 클래스 안에 선언, 암묵적 `this` | `def f(self)` | `impl S { fn f(&self) }` — `self` 를 **명시** |
| 생성자 | 생성자 함수 | `__init__` | 없음. 관례적으로 `S::new()` 연관 함수 |
| 합 타입(여러 모양 중 하나) | `std::variant<A, B, C>` + `std::visit` | 클래스 여러 개 + `match` (3.10+) | `enum E { A(..), B { .. }, C }` |
| "없을 수 있음" | `std::optional<T>` / 포인터 `nullptr` | `T \| None` | `Option<T>` (null 없음) |
| 분기 | `switch` (fallthrough, 빠뜨려도 경고 정도) | `match` (빠뜨려도 조용함) | `match` — **빠뜨리면 컴파일 에러** |
| 자동 구현 | `= default` 비교 연산자 (C++20) | dataclass 가 `__eq__`, `__repr__` 생성 | `#[derive(Debug, Clone, PartialEq)]` |

## 📘 이론

> 이 문서의 코드 조각은 대부분 <https://play.rust-lang.org> 에 붙여넣어 바로 실행할 수 있습니다 (표준 라이브러리만 사용). `fn main() { ... }` 로 감싸서 시도해 보세요.

### 1. 구조체와 `impl`

```rust
#[derive(Debug)]
struct Counter {
    name: String,
    count: u32,
}

impl Counter {
    // 연관 함수: self 가 없다 → Counter::new("hits") 로 호출
    fn new(name: &str) -> Self {
        Counter { name: name.to_string(), count: 0 }
    }
    // &mut self: 내용을 바꾼다
    fn bump(&mut self) {
        self.count += 1;
    }
    // &self: 읽기만 한다
    fn describe(&self) -> String {
        format!("{} = {}", self.name, self.count)
    }
    // self: 값을 소비(move)한다. 부른 뒤에는 원본을 못 쓴다
    fn into_name(self) -> String {
        self.name
    }
}

fn main() {
    let mut c = Counter::new("hits");
    c.bump();
    println!("{}", c.describe());     // hits = 1
    let name = c.into_name();         // c 는 여기서 소비됨
    println!("{name}");
    // println!("{}", c.describe());  // ❌ E0382: borrow of moved value
}
```

메서드의 첫 인자가 **"이 메서드가 나를 어떻게 다루는가"** 를 알려 줍니다.

| 첫 인자 | 의미 | C++ 비유 |
|---|---|---|
| `&self` | 읽기 전용 빌림 | `const` 멤버 함수 |
| `&mut self` | 수정 가능한 빌림 | 일반(비 const) 멤버 함수 |
| `self` | 소유권을 가져감 | 객체를 `std::move` 로 넘겨받는 함수 (단, 뒤에서 못 쓰도록 컴파일러가 강제) |
| (없음) | 연관 함수 | `static` 멤버 함수 |

💡 **쉬운 설명** — "연관 함수"는 인스턴스 없이 부르는, 타입에 딸린 함수입니다. C++ 의 `static` 함수와 같고, 생성자 대신 `new` 라는 이름을 관례로 씁니다 (언어 문법이 아니라 그냥 이름이에요).

⚠️ **C++ 개발자가 헷갈리는 점**
- Rust 의 `struct` 에는 상속이 없습니다. 코드 재사용은 트레이트(Ch08)와 조합(필드로 포함)으로 합니다.
- 필드가 `private` 인지 `public` 인지는 **모듈 단위**로 정해집니다(Ch10). 한 파일 안에서는 모두 접근 가능해서 이 챕터에서는 신경 쓰지 않아도 됩니다.
- 구조체 필드를 일부만 `mut` 로 만들 수는 없습니다. 변수 전체가 `let mut` 이거나 아니거나입니다.

### 2. 열거형(enum) — 데이터를 품은 sum type

C/C++ 의 `enum` 은 "이름 붙은 정수"입니다. Rust 의 `enum` 은 **"여러 모양 중 정확히 하나"** 이고, 모양마다 다른 데이터를 가질 수 있습니다.

```rust
enum Shape {
    Circle { radius: f64 },        // 이름 있는 필드
    Rect { w: f64, h: f64 },
    Point,                         // 데이터 없음
    Poly(Vec<(f64, f64)>),         // 튜플처럼 이름 없는 필드
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { radius } => 3.14159 * radius * radius,
        Shape::Rect { w, h } => w * h,
        Shape::Point => 0.0,
        Shape::Poly(_) => 0.0, // 일부러 생략한 계산
    }
}
```

같은 것을 C++20 에서 쓰면 이렇습니다.

```cpp
struct Circle { double radius; };
struct Rect { double w, h; };
struct Point {};
using Shape = std::variant<Circle, Rect, Point>;

double area(const Shape& s) {
    return std::visit(overloaded{                 // overloaded 는 직접 정의해야 하는 관용구
        [](const Circle& c) { return 3.14159 * c.radius * c.radius; },
        [](const Rect& r)   { return r.w * r.h; },
        [](const Point&)    { return 0.0; },
    }, s);
}
```

Rust 는 문법이 언어에 들어 있어 보일러플레이트가 없고, 무엇보다 **오류 처리가 다릅니다.**

💡 **쉬운 설명** — "sum type(합 타입)"은 "A 이거나 B 이거나 C 인 값"입니다. 반대로 struct 는 "A 이고 B 이고 C 인 값"이라 product type(곱 타입)이라 부릅니다. 상태 머신, 명령어, 에러 종류처럼 *"몇 가지 경우 중 하나"* 를 표현하는 데 딱 맞습니다.

⚠️ **C++ 개발자가 헷갈리는 점**
- `std::visit` 에 일반(generic) 람다 fallback `[](const auto&) {...}` 을 두면, variant 에 새 타입을 추가해도 **조용히 통과**합니다. Rust 에서는 `_ =>` 를 쓰지 않는 한 모든 `match` 가 컴파일 에러로 알려 줍니다.
- Rust 의 enum 값의 크기는 가장 큰 variant + 구분표(tag) 정도입니다. `std::variant` 와 같은 구조(tagged union)지만 안전하게 감싸져 있습니다.

### 3. `match` — 빠뜨릴 수 없는 분기

```rust
#[derive(Debug, Clone, Copy)]
enum Light { Red, Yellow, Green }

fn next(l: Light) -> Light {
    match l {
        Light::Red => Light::Green,
        Light::Green => Light::Yellow,
        Light::Yellow => Light::Red,
    }
}
```

`Light::Yellow` 줄을 지우면 이런 에러가 납니다.

```
error[E0004]: non-exhaustive patterns: `Light::Yellow` not covered
```

`match` 가 쓸 수 있는 패턴 도구들:

```rust
match (state, event) {                          // 튜플로 두 값을 한 번에
    (State::Idle, Event::Coin(c)) if c >= 100 => { /* 가드: 조건이 참일 때만 */ }
    (State::Idle | State::Broken, Event::Kick) => { /* | : 여러 패턴 중 아무거나 */ }
    (State::Run { speed, .. }, _) => { /* .. : 나머지 필드 무시, speed 로 값 꺼냄 */ }
    _ => {}                                     // _ : 나머지 전부
}
let grade = match score { 90.. => 'A', 80..=89 => 'B', _ => 'F' }; // 범위 패턴
```

- `match` 는 **식(expression)** 이라 값을 돌려줍니다 (Ch02).
- `_ =>` 는 편하지만 *"나중에 variant 가 늘어도 컴파일러가 알려 주지 않는다"* 는 비용이 있습니다. 변하는 enum 에는 가능하면 모든 variant 를 나열하세요.

💡 **쉬운 설명** — 패턴 매칭은 "값의 *모양*을 보고 분기하면서, 동시에 안의 값을 변수로 꺼내는" 기능입니다. `switch` 는 값을 비교만 하지만, `match` 는 열어서 꺼내 줍니다.

### 4. `Option<T>` — null 이 없는 이유

Rust 에는 `null` 이 없습니다. "없을 수 있음"은 표준 라이브러리의 평범한 enum 입니다.

```rust
enum Option<T> {   // 실제 표준 라이브러리 정의
    None,
    Some(T),
}
```

```rust
fn first_even(v: &[i32]) -> Option<i32> {
    for &x in v {
        if x % 2 == 0 {
            return Some(x);
        }
    }
    None
}

fn main() {
    let r = first_even(&[1, 3, 4]);
    // r + 1;                       // ❌ 컴파일 에러: Option<i32> 는 i32 가 아님
    match r {
        Some(x) => println!("found {x}"),
        None => println!("nothing"),
    }
}
```

"없을 수 있는 값"은 **꺼내 보기 전에는 쓸 수 없는 타입**이어서, null 포인터 역참조(C++ 의 UB, Python 의 `AttributeError: 'NoneType'`) 계열의 버그가 컴파일 단계에서 사라집니다.

| 하고 싶은 일 | 방법 |
|---|---|
| 둘 다 처리 | `match opt { Some(x) => .., None => .. }` |
| `Some` 일 때만 처리 | `if let Some(x) = opt { .. }` |
| `Some` 이 아니면 일찍 탈출 | `let Some(x) = opt else { return ..; };` (let-else) |
| 기본값으로 대체 | `opt.unwrap_or(0)` |
| 값을 변환 | `opt.map(\|x\| x + 1)` |
| 확신이 있을 때 (실패하면 panic) | `opt.unwrap()` / `opt.expect("이유")` — 실전 코드에서는 아껴 쓰기 |

```rust
fn describe(v: &[i32]) -> String {
    let Some(first) = v.first() else {
        return "empty".to_string();   // else 블록은 반드시 탈출(return/continue/break/panic)해야 함
    };
    format!("first = {first}")
}
```

⚠️ **C++ 개발자가 헷갈리는 점**
- `std::optional` 은 `*opt` 로 접근할 때 검사가 없어서(UB) null 포인터와 크게 다르지 않습니다. Rust 의 `Option` 은 `match`/`if let` 으로만 안을 볼 수 있습니다 (`unwrap` 은 명시적 panic).
- `Option<&T>` 는 "참조 또는 없음"이고 내부적으로 널 포인터 한 칸으로 표현되어 추가 비용이 없습니다. (null 포인터 최적화)

### 5. `derive` — 자동으로 만들어 주는 trait 구현

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: i32, y: i32 }

let a = Point { x: 1, y: 2 };
let b = a.clone();               // Clone: 깊은 복사
assert_eq!(a, b);                // PartialEq: == 비교
println!("{a:?}");               // Debug: Point { x: 1, y: 2 }
println!("{a:#?}");              // 예쁘게 줄바꿈해서
```

| derive | 하는 일 | 대응 |
|---|---|---|
| `Debug` | `{:?}` 로 개발자용 출력 | Python `__repr__`, dataclass 자동 |
| `Clone` | `.clone()` 으로 명시적 복사 | C++ 복사 생성자 (Rust 는 **명시적**으로 호출) |
| `Copy` | 대입해도 이동 대신 비트 복사 (정수처럼) | C++ trivially copyable |
| `PartialEq` | `==` / `!=` | `operator==` (`= default`) |
| `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default` | 해시맵 키, 정렬, 기본값 | |

💡 **쉬운 설명** — `derive` 는 "이 구조체의 모든 필드가 이 능력을 가지니, 필드별로 알아서 구현해 줘" 라는 주문입니다. 필드 중 하나라도 그 능력이 없으면(예: `f64` 가 있는 구조체에 `Eq`) 컴파일 에러가 납니다.

## 🧯 자주 만나는 컴파일 에러

**1. `E0004` — match 에서 variant 를 빠뜨림**
```
error[E0004]: non-exhaustive patterns: `State::Refunding { .. }` not covered
  = note: the matched value is of type `&State`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern
```
→ 의미: 가능한 모든 모양을 다루지 않았습니다. 고치는 법: 빠진 variant 를 추가하거나, 정말 "나머지 전부"가 같은 처리라면 `_ =>` 를 씁니다.

**2. `E0382` — `self` 를 받는 메서드를 부른 뒤 원본을 사용**
```
error[E0382]: borrow of moved value: `machine`
  note: `Machine::into_sold` takes ownership of the receiver `self`, which moves `machine`
```
→ 의미: `fn xxx(self)` 는 객체를 소비합니다. 고치는 법: 소비 전에 필요한 걸 다 쓰거나, 정말 계속 써야 하면 메서드를 `&self` 로 바꾸거나 `.clone()`.

**3. `E0308` — Option 을 값처럼 사용**
```
error[E0308]: mismatched types
  expected `u32`, found `Option<u32>`
```
→ 의미: `Some(...)` 으로 감싸야 하거나, 반대로 꺼내서 써야 합니다. 고치는 법: `match` / `if let` / `unwrap_or`.

**4. `E0023` / `E0026` — 패턴의 필드 모양이 틀림**
```
error[E0026]: variant `State::Collecting` does not have a field named `cent`
```
→ 의미: 필드 이름 오타. 튜플형 variant(`Some(x)`)와 이름형 variant(`Collecting { cents }`)를 헷갈려도 납니다.

**5. `E0369` — `==` 를 쓰려는데 `PartialEq` 가 없음**
```
error[E0369]: binary operation `==` cannot be applied to type `Event`
help: consider annotating `Event` with `#[derive(PartialEq)]`
```
→ 의미: derive 를 붙이지 않았습니다. 고치는 법: `#[derive(PartialEq)]` (assert_eq! 에는 `Debug` 도 필요).

**6. `E0599` — 메서드를 찾을 수 없음**
```
error[E0599]: no method named `price` found for reference `&Item` in the current scope
```
→ 의미: `impl Item { fn price(..) }` 블록이 없거나 이름이 다릅니다.

## 🛠️ 실습: 자판기 상태 머신

### 무엇을 만드나요
동전을 넣고 상품을 고르고 꺼내는 **자판기**를 enum 으로 모델링합니다. `Event`(사건)를 차례로 흘려 보내며 `State`(상태)가 어떻게 바뀌는지 추적(trace)합니다.

```
step 01: InsertCoin(100)  -> Collecting(100)
step 02: InsertCoin(7)    -> Collecting(100)
step 03: InsertCoin(100)  -> Collecting(200)
step 04: Select(Cola)     -> Dispensing(Cola, change=50)
step 05: Take             -> Idle
...
step 13: ? dance          -> Idle
step 15: Kick             -> OutOfService(kicked)
...
revenue: 220
sold: Cola, Candy
```

상태: `Idle`, `Collecting{cents}`, `Dispensing{item, change}`, `Refunding{cents}`, `OutOfService{reason}`
사건: `InsertCoin(u32)`, `Select(Item)`, `Cancel`, `Take`, `Kick`, `Repair`
(유효한 동전은 10/50/100/500 뿐, 그 외는 무시)

### 파일 구성
| 경로 | 설명 |
|---|---|
| `python/main.py` | dataclass + `match` 참조 구현 |
| `cpp/main.cpp` | `std::variant` + `std::visit` 참조 구현 |
| `rust/src/main.rs` | **실습 파일** (TODO 9개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (모두 챕터 디렉터리 `chapters/ch05_structs_enums` 에서)
```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch05_structs_enums                      # 실습 파일 (TODO 를 풀기 전에는 panic)
cargo test -p ch05_structs_enums                        # 실습 테스트
cargo run -q -p ch05_structs_enums --example solution   # 정답
../../scripts/compare.sh ch05                           # 세 언어 출력 비교 (--solution 은 정답과 비교)
```

### TODO 목록
| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `Item::price` | 데이터 없는 enum 의 `match` |
| 2 | `Item::from_name` | `Option` 반환, 문자열 `match` |
| 3 | `parse_event` | 튜플 패턴, `?` on Option, enum variant 를 함수로 쓰기 |
| 4 | `Display for State` | 필드 있는 variant 구조 분해, exhaustive |
| 5 | `transition` | `(state, event)` 튜플 match, 가드, `\|` 패턴 |
| 6 | `sale_of` | `let else`, `bool::then_some` |
| 7 | `Machine::new` | 연관 함수, 구조체 리터럴 |
| 8 | `Machine::handle` | `&mut self`, `if let` |
| 9 | `Machine::revenue` | `&self`, 이터레이터 맛보기 |

### 완료 기준
- `cargo test -p ch05_structs_enums` 가 모두 통과
- `../../scripts/compare.sh ch05` 에서 Python / C++ / Rust 출력이 동일

### 🔬 실험 목록
1. `main()` 의 주석 처리된 `println!("{}", machine.revenue());` 를 풀어 보세요 → `E0382`. `into_sold(self)` 가 소비했기 때문입니다.
2. `State` 에 `Broken` variant 를 추가해 보세요 → `Display` 의 `match` 에서 `E0004`. 반면 `transition` 은 `_ =>` 때문에 조용히 넘어갑니다. **어디에 와일드카드를 써도 되는지** 생각해 보세요.
3. (선택) `transition` 의 첫 번째 팔과 두 번째 팔 순서를 바꿔 보세요. 어떤 테스트가 실패하나요? (match 는 위에서 아래로 첫 일치)

## 🤖 바이브 코딩 포인트
- AI 가 만든 Rust 코드에서 **`_ =>` 와일드카드**가 enum `match` 에 있는지 보세요. 나중에 variant 가 추가돼도 컴파일러 도움을 못 받는 지점입니다. "상태 enum 의 match 에서는 `_` 대신 모든 variant 를 나열해 줘" 라고 요청할 수 있습니다.
- `unwrap()` / `expect()` 가 `Option` 에 남발되어 있으면 panic 지점입니다. 입력이 외부에서 오는 경우 `match` / `if let` / `ok_or` 로 바꾸라고 요청하세요.
- 상태/명령을 `String` 이나 `bool` 여러 개로 표현했다면 "잘못된 조합이 가능한 설계"입니다. "불가능한 상태를 표현할 수 없도록 enum 으로 모델링해 줘" 가 좋은 프롬프트입니다.
- 메서드 시그니처의 `self` / `&self` / `&mut self` 를 읽는 습관을 들이세요. 호출하는 쪽의 `let mut` 필요 여부와 소유권 이동 여부가 모두 여기서 결정됩니다.
- `#[derive(...)]` 목록은 AI 가 자주 과하게(`Copy`, `Hash` 등) 붙입니다. 큰 구조체에 `Copy` 가 있으면 의도를 물어보세요.

## ✅ 셀프 체크
1. `fn f(self)`, `fn f(&self)`, `fn f(&mut self)` 의 차이를 호출하는 쪽 관점에서 설명해 보세요.
<details><summary>답</summary>`self` 는 값을 소비(move)해서 호출 후 원본을 못 쓰고, `&self` 는 읽기 전용으로 빌려서 여러 곳에서 동시에 가능하며, `&mut self` 는 수정 가능한 배타적 빌림이라 호출하려면 변수가 `mut` 여야 하고 그동안 다른 빌림이 없어야 합니다.</details>

2. Rust 에 `null` 이 없는데 "없는 값"은 어떻게 표현하나요? `Option<T>` 의 장점은?
<details><summary>답</summary>표준 enum `Option<T>`(`Some(T)` / `None`)로 표현합니다. 값을 쓰려면 반드시 `match` 등으로 꺼내야 해서 "None 인데 쓰는" 버그가 컴파일 단계에서 막힙니다.</details>

3. `match` 에서 variant 하나를 빠뜨리면 어떻게 되나요? C++ `switch` 와 `std::visit` 는?
<details><summary>답</summary>Rust 는 `E0004` 컴파일 에러입니다. C++ `switch` 는 경고(`-Wswitch`)에 그치고, `std::visit` 는 모든 타입을 다루지 않으면 컴파일 에러지만 generic fallback 람다를 두면 조용히 통과합니다.</details>

4. `let Some(x) = opt else { return None; };` 는 어떤 상황에서 `if let` 보다 좋은가요?
<details><summary>답</summary>"맞지 않으면 일찍 탈출"하는 흐름에서 이후 코드를 들여쓰기 없이 `x` 를 쓰며 이어갈 수 있어서 가독성이 좋습니다.</details>

5. `(State::Idle | State::Broken, Event::Kick)` 에서 `|` 는 무슨 뜻인가요?
<details><summary>답</summary>"둘 중 아무 패턴이나 일치하면" 같은 팔을 실행한다는 뜻입니다. 단, 각 패턴이 같은 이름·타입의 변수를 바인딩해야 합니다.</details>

## 📚 더 알아보기
- The Rust Book: [Ch05 Structs](https://doc.rust-lang.org/book/ch05-00-structs.html), [Ch06 Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html), [Ch18 Patterns](https://doc.rust-lang.org/book/ch18-00-patterns.html)
- Rust by Example: [Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html), [match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html), [let else](https://doc.rust-lang.org/rust-by-example/flow_control/let_else.html)
- std docs: [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html), [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
