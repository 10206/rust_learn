# 01. C++20 / Python / Rust 대응표 (치트시트)

> "C++로는 이렇게 썼는데 Rust에서는 뭐라고 쓰지?" 할 때 펼쳐 보는 표입니다. 각 줄은 **C++20 | Python | Rust | 한 줄 메모** 순서예요. MATLAB 경험이 도움이 되는 곳에는 따로 메모를 달았습니다.
>
> 표 아래의 코드 블록은 모두 `rustc 1.97`(edition 2024)에서 실제로 컴파일해 확인한 예제입니다(비동기/HTTP 예제는 `tokio`, `reqwest`, `serde` 의존성을 넣고 확인했어요). 표 안의 짧은 조각은 표현 이해를 위한 것이고, 의미가 헷갈리면 해당 챕터를 보세요.

## 목차

1. [변수와 타입](#1-변수와-타입)
2. [문자열](#2-문자열)
3. [함수](#3-함수)
4. [제어 흐름](#4-제어-흐름)
5. [구조체 / 클래스](#5-구조체--클래스)
6. [열거형 / variant](#6-열거형--variant)
7. [제네릭 / 템플릿 / concepts](#7-제네릭--템플릿--concepts)
8. [트레이트 / virtual / Protocol](#8-트레이트--virtual--protocol)
9. [메모리와 스마트 포인터](#9-메모리와-스마트-포인터)
10. [참조와 빌림](#10-참조와-빌림)
11. [에러 처리](#11-에러-처리)
12. [컬렉션](#12-컬렉션)
13. [이터레이터 / ranges / 컴프리헨션](#13-이터레이터--ranges--컴프리헨션)
14. [클로저 / 람다](#14-클로저--람다)
15. [모듈 / 헤더 / 패키지](#15-모듈--헤더--패키지)
16. [빌드 도구](#16-빌드-도구)
17. [테스트](#17-테스트)
18. [동시성](#18-동시성)
19. [비동기](#19-비동기)
20. [JSON](#20-json)
21. [HTTP](#21-http)
22. [MATLAB 사용자를 위한 메모](#22-matlab-사용자를-위한-메모)

> 표 읽는 법: 표의 `\|` 는 클로저의 `|` 기호를 표 구분선과 구별하기 위해 쓴 것입니다. 실제 코드에서는 `|x| x + 1`처럼 씁니다.

---

## 1. 변수와 타입

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 불변 변수 | `const int x = 5;` | `x = 5` (관례로만 상수) | `let x = 5;` | Rust는 **기본이 불변**. |
| 가변 변수 | `int x = 5;` | `x = 5` | `let mut x = 5;` | 바꾸려면 `mut`를 명시. |
| 상수 | `constexpr int N = 10;` | `N = 10` | `const N: usize = 10;` | 타입 표기 필수, 컴파일 타임 값. 전역은 `static`. |
| 타입 추론 | `auto x = 5;` | (동적 타입) | `let x = 5;` | 추론은 되지만 **정적 타입**. 못 정하면 E0282. |
| 타입 표기 | `int64_t x = 5;` | `x: int = 5` | `let x: i64 = 5;` | 이름 뒤에 `: 타입`. |
| 정수형 | `int8_t`…`uint64_t`, `size_t` | `int` (무제한) | `i8 i16 i32 i64 i128 isize` / `u8 … usize` | 기본 정수는 `i32`. 인덱스는 `usize`. |
| 실수형 | `float`, `double` | `float` (=double) | `f32`, `f64` | 기본은 `f64`. |
| 불리언/문자 | `bool`, `char` (1바이트) | `bool`, `str` (길이 1) | `bool`, `char` (4바이트 유니코드) | Rust `char`는 한글 한 글자도 담아요. |
| 형 변환 | `static_cast<double>(i)` | `float(i)` | `i as f64` / `f64::from(i)` | 암묵적 변환 **없음**. `as`는 잘릴 수 있어요. |
| 실패 가능한 변환 | (직접 검사) | `int(s)` → 예외 | `i32::try_from(x)?`, `s.parse::<i32>()` | `Result`를 돌려줘요. |
| 튜플 | `std::tuple<int,double>` | `(1, 2.0)` | `(1, 2.0)`, `t.0` | 접근은 `.0`, `.1`. |
| 고정 배열 | `std::array<int,3>` | (없음, list) | `[i32; 3]`, `[0; 3]` | 길이가 **타입의 일부**. |
| 섀도잉 | (같은 스코프 재선언 불가) | `x = x + 1` (덮어씀) | `let x = x + 1;` | 같은 이름으로 새 변수. 타입도 바꿀 수 있음. |
| 오버플로 | 부호 있는 정수는 UB | 무제한 정수 | debug 빌드 panic, `checked_add` / `wrapping_add` / `saturating_add` | 의도를 메서드 이름으로 선택. |
| 기본값 | 값 초기화 `T{}` | `None`/0 등 | `Default::default()` | `#[derive(Default)]`. **초기화 안 한 변수는 못 씁니다**. |
| null | `nullptr`, `std::optional` | `None` | `Option<T>` (`Some`/`None`) | null 포인터가 언어에 없습니다. |

```rust
fn main() {
    let x = 5; // 불변
    let mut y = 10; // 가변
    y += x;

    let big: u64 = 3_000_000_000; // 숫자 사이 _ 는 가독성용
    let small = big as u8; // as 는 잘라 버립니다(주의)
    let safe = u8::try_from(big); // 실패하면 Err
    println!("{y} {small} {}", safe.is_err());

    // 섀도잉: 타입까지 바꿀 수 있어요
    let input = "42";
    let input: i32 = input.parse().unwrap();
    println!("{}", input + 1);

    // 오버플로를 의도에 맞게
    let a: u8 = 250;
    println!("{:?} {} {}", a.checked_add(10), a.wrapping_add(10), a.saturating_add(10));

    let t = (1, 2.5, 'x');
    let arr = [0u8; 4];
    println!("{} {} {} {}", t.0, t.1, t.2, arr.len());
}
```

---

## 2. 문자열

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 소유 문자열 | `std::string` | `str` | `String` | 힙에 있는 UTF-8, 늘어날 수 있음. |
| 빌린 문자열 | `std::string_view` | (슬라이스 `s[a:b]`는 복사) | `&str` | **뷰**. 함수 매개변수는 `&str`가 관례. |
| 리터럴 | `"abc"` (`const char*`) | `"abc"` | `"abc"` (`&'static str`) | 프로그램 바이너리에 박혀 있는 `&str`. |
| 만들기 | `std::string("a")` | `str(x)` | `String::from("a")`, `"a".to_string()`, `format!(..)` | `format!`은 `println!`의 문자열 반환판. |
| 이어 붙이기 | `s + t`, `s += t` | `s + t` | `s + &t`, `s.push_str(&t)`, `format!("{s}{t}")` | `+`는 왼쪽 `String`의 소유권을 **가져가요**. |
| 길이 | `s.size()` (바이트) | `len(s)` (글자 수) | `s.len()` (바이트), `s.chars().count()` (글자) | 한글 한 글자는 3바이트. |
| 인덱싱 | `s[i]` | `s[i]` | **불가**: `s.chars().nth(i)`, `&s[a..b]` | 바이트 경계가 글자 경계가 아니면 panic. |
| 부분 문자열 | `s.substr(a, n)` | `s[a:b]` | `&s[a..b]` | 복사 없는 `&str`. |
| 분리 | (직접 구현/ranges) | `s.split(",")` | `s.split(',')`, `s.split_whitespace()` | 이터레이터를 돌려줘요. |
| 공백 제거 | (직접 구현) | `s.strip()` | `s.trim()` | `&str`를 돌려줘요. |
| 포함/시작 | `s.contains(x)` (C++23), `starts_with` | `x in s`, `s.startswith` | `s.contains(x)`, `s.starts_with(x)` | |
| 치환 | `std::regex_replace` 등 | `s.replace(a, b)` | `s.replace(a, b)` | 새 `String` 반환. |
| 대소문자 | `std::toupper` 반복 | `s.upper()` | `s.to_uppercase()` | 새 `String`. |
| 숫자 → 문자열 | `std::to_string(n)` | `str(n)` | `n.to_string()`, `format!("{n}")` | |
| 문자열 → 숫자 | `std::stoi(s)` (예외) | `int(s)` (예외) | `s.parse::<i32>()` → `Result` | 실패를 타입으로 알려 줘요. |
| 포매팅 | `std::format("{:.2f}", x)` | `f"{x:.2f}"` | `format!("{x:.2}")`, `println!("{:>5}", x)` | 문법이 C++20 `std::format`/Python과 거의 같아요. |
| 디버그 출력 | (직접 `operator<<`) | `repr(x)` | `{:?}`, `{:#?}` | `#[derive(Debug)]` 필요. |
| 바이트 | `std::vector<uint8_t>` | `bytes` | `Vec<u8>`, `&[u8]`, `s.as_bytes()` | |

```rust
fn longest_word(text: &str) -> &str {
    text.split_whitespace().max_by_key(|w| w.len()).unwrap_or("")
}

fn main() {
    let mut s = String::from("hello");
    s.push_str(", world"); // 가변 String
    let view: &str = &s[0..5]; // 복사 없는 뷰
    println!("{view} {}", s.len());

    let kor = "안녕";
    println!("{} 바이트, {} 글자", kor.len(), kor.chars().count());

    let n: i32 = "42".parse().unwrap();
    let r: Result<i32, _> = "abc".parse::<i32>();
    println!("{n} {}", r.is_err());

    let name = "Rust";
    let pi = 3.14159;
    println!("{}", format!("{name:>6}|{pi:.2}|{:05}", 42));
    println!("{}", longest_word("a quick brownish fox"));
}
```

> ⚠️ **C++/Python 개발자가 헷갈리는 점**: `&String`을 받는 함수보다 `&str`을 받는 함수가 더 범용적입니다(`String`, 리터럴 모두 넘길 수 있어요). 자세한 내용은 ch04.

---

## 3. 함수

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 정의 | `int add(int a, int b) { return a+b; }` | `def add(a, b): return a+b` | `fn add(a: i32, b: i32) -> i32 { a + b }` | **마지막 식에 `;`를 안 붙이면** 그 값이 반환값. |
| 반환값 없음 | `void` | `None` | `()` (유닛 타입, 생략 가능) | |
| 조기 반환 | `return x;` | `return x` | `return x;` | |
| 기본 인자 | `int f(int a = 1)` | `def f(a=1)` | **없음** | 빌더 패턴/`Option<T>`/오버로드 대신 별도 함수. |
| 오버로드 | 가능 | 불가 | **없음** | 이름을 달리하거나 제네릭/트레이트 사용. |
| 가변 인자 | `Args&&...`, `...` | `*args` | **없음** (매크로 `println!` 등) | 슬라이스 `&[T]` 를 받으세요. |
| 값 전달 | 복사 | 참조(객체) | **이동**(`T`) | `T`를 받으면 호출자는 소유권을 잃어요. |
| 읽기 전용 전달 | `const T&` | (관례) | `&T` | |
| 수정용 전달 | `T&` | (가변 객체) | `&mut T` | 호출부에도 `&mut x`를 **적어야** 해요. |
| 여러 값 반환 | `std::pair`, `tuple`, 구조화 바인딩 | `return a, b` | `-> (i32, i32)`, `let (a, b) = f();` | |
| 함수 포인터 | `int(*)(int)` | 함수 객체 | `fn(i32) -> i32` | 클로저는 `impl Fn(i32) -> i32`. |
| 메서드 | `struct S { int f(); }` | `def f(self)` | `impl S { fn f(&self) {} }` | `self`를 명시적으로 적어요. |
| 연관 함수(정적) | `static S make();` | `@staticmethod` | `impl S { fn new() -> S {..} }` → `S::new()` | 생성자는 관례상 `new`. |

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b // 세미콜론 없음 = 이 값이 반환값
}

fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}

fn push_twice(v: &mut Vec<i32>, x: i32) {
    v.push(x);
    v.push(x);
}

fn sum(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

fn main() {
    let (q, r) = divmod(17, 5);
    let mut v = vec![];
    push_twice(&mut v, 7); // 호출부에도 &mut
    println!("{} {q} {r} {}", add(1, 2), sum(&v));
}
```

---

## 4. 제어 흐름

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| if | `if (c) {..} else {..}` | `if c: ... else: ...` | `if c { .. } else { .. }` | 조건에 **괄호 불필요**, `bool`만 가능(정수 불가). |
| 삼항 | `c ? a : b` | `a if c else b` | `if c { a } else { b }` | `if`는 **식(expression)**. |
| switch | `switch (x) {..}` | `match x:` (3.10+) | `match x { 1 => .., _ => .. }` | **빠짐없이** 다뤄야 합니다(E0004). |
| 범위 반복 | `for (int i=0;i<n;i++)` | `for i in range(n)` | `for i in 0..n` | `0..=n`은 끝 포함. |
| 컬렉션 반복 | `for (auto& x : v)` | `for x in v` | `for x in &v` / `&mut v` / `v` | `&`, `&mut`, 값 중 무엇을 반복하는지 선택. |
| 인덱스와 값 | (직접) | `enumerate(v)` | `for (i, x) in v.iter().enumerate()` | |
| while | `while (c)` | `while c:` | `while c { }` | |
| 무한 루프 | `for(;;)`, `while(true)` | `while True:` | `loop { }` | `break 값;`으로 값을 돌려줄 수 있음. |
| 라벨 break | `goto` | (없음) | `'outer: loop { break 'outer; }` | 중첩 루프 탈출. |
| 패턴 조건 | `if (auto it = ...; it != end)` | `if (m := f()) is not None:` | `if let Some(x) = opt { }` | |
| 조기 탈출 | `if (!x) return;` | `if x is None: return` | `let Some(x) = opt else { return; };` | `let … else` (Rust 1.65+). |
| match 가드 | `if` 따로 | `case n if n>0:` | `n if n > 0 => ..` | |
| 범위 패턴 | (직접) | (직접) | `1..=5 => ..` | |

```rust
fn classify(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        x if x % 2 == 0 => "even",
        _ => "odd",
    }
}

fn first_even(v: &[i32]) -> Option<i32> {
    let Some(&x) = v.iter().find(|&&x| x % 2 == 0) else {
        return None;
    };
    Some(x)
}

fn main() {
    let label = if 3 > 2 { "yes" } else { "no" }; // if 도 식
    let mut n = 0;
    let found = loop {
        n += 1;
        if n * n > 50 {
            break n; // loop 는 값을 돌려줄 수 있음
        }
    };
    'outer: for i in 0..5 {
        for j in 0..5 {
            if i * j == 6 {
                println!("found {i} {j}");
                break 'outer;
            }
        }
    }
    println!("{label} {found} {} {:?}", classify(4), first_even(&[1, 3, 8]));
}
```

---

## 5. 구조체 / 클래스

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 데이터 묶음 | `struct P { int x; int y; };` | `@dataclass class P: x: int; y: int` | `struct P { x: i32, y: i32 }` | |
| 생성 | `P p{1, 2};` | `P(1, 2)` | `P { x: 1, y: 2 }` | **모든 필드**를 지정해야 해요. |
| 일부만 바꿔 복사 | (복사 후 수정) | `dataclasses.replace(p, x=3)` | `P { x: 3, ..p }` | 구조체 갱신 문법. 이동/복사에 주의. |
| 메서드 | 멤버 함수 | `def f(self)` | `impl P { fn f(&self) {} }` | `&self` 읽기 / `&mut self` 수정 / `self` 소비. |
| 생성자 | `P(int x, int y)` | `__init__` | `fn new(x: i32, y: i32) -> P` | 관례일 뿐 문법이 아니에요. |
| 소멸자 | `~P()` | `__del__` (비권장) | `impl Drop for P { fn drop(&mut self) {} }` | 스코프를 벗어날 때 자동 호출(RAII). |
| 비교 | `auto operator<=>(...) = default;` | `@dataclass(order=True)` | `#[derive(PartialEq, Eq, PartialOrd, Ord)]` | |
| 출력 | `operator<<` | `__repr__` | `#[derive(Debug)]` → `{:?}`, `impl Display` | |
| 복사 | 복사 생성자 (기본 복사) | `copy.copy` | `#[derive(Clone)]` → `p.clone()`, `#[derive(Clone, Copy)]` | **복사는 명시적**(Copy는 비트 복사로 값이 작을 때만). |
| 접근 제어 | `private:` / `public:` | `_name` (관례) | 기본 비공개, `pub` 로 공개 | 비공개 단위는 **클래스가 아니라 모듈**. |
| 상속 | `class D : public B` | `class D(B)` | **없음** (구성 + 트레이트) | 데이터 상속이 없어요. 필드로 포함(composition). |
| 튜플 구조체 | (직접) | `NamedTuple` | `struct Meters(f64);` | newtype 패턴. |

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y } // 필드 이름이 같으면 줄여 쓸 수 있어요
    }
    fn dist2(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
    fn shift(&mut self, dx: i32) {
        self.x += dx;
    }
    fn into_tuple(self) -> (i32, i32) {
        (self.x, self.y) // self 를 소비(이동)
    }
}

struct Guard(&'static str);
impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop: {}", self.0);
    }
}

fn main() {
    let mut p = Point::new(3, 4);
    p.shift(1);
    let q = Point { y: 0, ..p.clone() };
    println!("{:?} {:?} {}", p, q, p.dist2());
    println!("{:?}", p.into_tuple());
    let _a = Guard("a");
    let _b = Guard("b"); // 선언의 역순으로 b, a 가 drop 됩니다
    println!("{:?}", Point::default());
}
```

---

## 6. 열거형 / variant

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 단순 열거 | `enum class Color { Red, Green };` | `class Color(Enum)` | `enum Color { Red, Green }` | |
| **데이터가 있는 열거** | `std::variant<A, B>` | 여러 클래스 + `Union` | `enum Shape { Circle(f64), Rect { w: f64, h: f64 } }` | Rust enum이 곧 **합 타입(sum type)**. |
| 분기 | `std::visit(overloaded{..}, v)` | `match v: case Circle(r):` | `match s { Shape::Circle(r) => .., Shape::Rect { w, h } => .. }` | 빠짐없는 검사(exhaustive). |
| 선택적 값 | `std::optional<T>` | `T \| None` | `Option<T>` | 표준 enum. `Some(x)` / `None`. |
| 성공/실패 | `std::expected<T,E>` (C++23) | 예외 | `Result<T, E>` | 표준 enum. `Ok(x)` / `Err(e)`. |
| 현재 값 검사 | `v.index()`, `holds_alternative` | `isinstance` | `matches!(s, Shape::Circle(_))`, `if let` | |
| 정수 값 | `enum class E : int { A = 1 }` | `IntEnum` | `enum E { A = 1 }` → `E::A as i32` | |
| 열거형에 메서드 | (자유 함수) | 메서드 | `impl Shape { fn area(&self) -> f64 {..} }` | |

```rust
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Empty,
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.14 * r * r,
            Shape::Rect { w, h } => w * h,
            Shape::Empty => 0.0,
        }
    }
}

fn describe(opt: Option<&Shape>) -> String {
    match opt {
        Some(Shape::Circle(r)) if *r > 10.0 => "big circle".to_string(),
        Some(s) => format!("area {:.1}", s.area()),
        None => "nothing".to_string(),
    }
}

fn main() {
    let shapes = vec![Shape::Circle(2.0), Shape::Rect { w: 2.0, h: 3.0 }, Shape::Empty];
    for s in &shapes {
        println!("{:?} -> {:.2}", s, s.area());
    }
    println!("{}", describe(shapes.first()));
    println!("{}", describe(None));
    println!("{}", matches!(shapes[2], Shape::Empty));
}
```

---

## 7. 제네릭 / 템플릿 / concepts

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 제네릭 함수 | `template<class T> T f(T x)` | (동적이라 불필요) / `TypeVar` | `fn f<T>(x: T) -> T` | |
| 제약 | `requires std::totally_ordered<T>` | `Protocol` (정적 검사 도구용) | `fn f<T: PartialOrd>(x: T)` | **트레이트 경계(bound)**. |
| 여러 제약 | `requires A<T> && B<T>` | | `T: A + B`, `where T: A + B` | 경계가 길면 `where`절. |
| 제네릭 타입 | `template<class T> struct S` | `class S(Generic[T])` | `struct S<T> { v: T }` | |
| 간단한 제약 | `void f(std::integral auto x)` | | `fn f(x: impl Display)` | `impl Trait` 인자 = 익명 제네릭. |
| 에러 시점 | 인스턴스화 시 (긴 에러) | 실행 시 | **정의 시점**에 경계로 검사 | 경계에 없는 연산을 쓰면 즉시 에러. |
| 특수화 | 템플릿 특수화 | | **없음** (트레이트 구현으로 대체) | |
| 비타입 인자 | `template<size_t N>` | | `struct S<const N: usize>` | const generics. |
| 컴파일 방식 | 인스턴스화(코드 복제) | | **단형화(monomorphization)** | C++ 템플릿과 같은 방식. 빠르지만 바이너리 증가. |

```rust
use std::fmt::Display;

fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T {
    let mut m = xs[0];
    for &x in xs {
        if x > m {
            m = x;
        }
    }
    m
}

fn show_all<T>(items: &[T]) -> String
where
    T: Display,
{
    items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
}

struct Pair<T> {
    a: T,
    b: T,
}

impl<T: PartialOrd + Display> Pair<T> {
    fn bigger(&self) -> &T {
        if self.a > self.b { &self.a } else { &self.b }
    }
}

fn print_it(x: impl Display) {
    println!("{x}");
}

fn main() {
    println!("{} {:.1}", largest(&[3, 9, 2]), largest(&[1.5, 0.2]));
    println!("{}", show_all(&["a", "b"]));
    println!("{}", Pair { a: 3, b: 8 }.bigger());
    print_it("hello");
}
```

---

## 8. 트레이트 / virtual / Protocol

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 인터페이스 정의 | 순수 가상 클래스 | `Protocol` / `ABC` | `trait Tool { fn run(&self) -> String; }` | |
| 구현 | `class A : public Tool` | 상속 또는 덕 타이핑 | `impl Tool for A { .. }` | 타입 **밖에서** 구현을 붙일 수 있어요(고아 규칙 제한 있음). |
| 기본 구현 | 가상 함수 본문 | 메서드 본문 | trait 안의 `fn name(&self) -> String { .. }` | |
| 동적 디스패치 | `std::unique_ptr<Base>` + `virtual` | 기본 동작 | `Box<dyn Tool>`, `&dyn Tool` | vtable 방식. |
| 정적 디스패치 | 템플릿 / concepts | | `fn f<T: Tool>(t: &T)`, `impl Tool` | 단형화, 인라인 가능. |
| 이질적 컬렉션 | `vector<unique_ptr<Base>>` | `list[Base]` | `Vec<Box<dyn Tool>>` | |
| 연산자 오버로드 | `operator+` | `__add__` | `impl Add for P` | `std::ops` 트레이트. |
| 변환 | 변환 생성자/`operator T()` | `__init__`/`int()` | `impl From<A> for B`, `.into()` | `From`을 구현하면 `Into`는 자동. |
| 출력 | `operator<<` | `__str__` | `impl fmt::Display for T` | |
| 소멸 | `~T()` | `__del__` | `impl Drop for T` | |
| 마커 | (개념 없음) | | `Send`, `Sync`, `Copy` | 구현할 메서드가 없는 트레이트. |
| `dyn` 제약 | | | 제네릭 메서드/`Self` 반환이 있으면 `dyn` 불가 (object safety) | 에러가 나면 설계를 `enum`이나 제네릭으로. |

```rust
use std::fmt;

trait Tool {
    fn name(&self) -> &str;
    fn run(&self, input: &str) -> Result<String, String>;
    fn describe(&self) -> String {
        format!("tool `{}`", self.name()) // 기본 구현
    }
}

struct Reverser;
struct Upper;

impl Tool for Reverser {
    fn name(&self) -> &str { "reverse" }
    fn run(&self, input: &str) -> Result<String, String> {
        Ok(input.chars().rev().collect())
    }
}

impl Tool for Upper {
    fn name(&self) -> &str { "upper" }
    fn run(&self, input: &str) -> Result<String, String> {
        if input.is_empty() { Err("empty".into()) } else { Ok(input.to_uppercase()) }
    }
}

struct Celsius(f64);
impl fmt::Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}C", self.0)
    }
}

fn run_static<T: Tool>(t: &T, s: &str) -> Result<String, String> {
    t.run(s) // 단형화: 호출이 인라인될 수 있어요
}

fn main() {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(Reverser), Box::new(Upper)];
    for t in &tools {
        println!("{} -> {:?}", t.describe(), t.run("abc"));
    }
    println!("{:?} {}", run_static(&Upper, ""), Celsius(21.55));
}
```

---

## 9. 메모리와 스마트 포인터

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 힙 단일 소유 | `std::unique_ptr<T>` / `make_unique` | (모든 객체가 힙) | `Box<T>` / `Box::new(x)` | 소유자가 하나. 이동만 가능. |
| 공유 소유 | `std::shared_ptr<T>` (원자적) | 참조 카운트(자동) | `Rc<T>` (단일 스레드) / `Arc<T>` (원자적) | 스레드를 넘길 땐 `Arc`. |
| 약한 참조 | `std::weak_ptr<T>` | `weakref` | `Weak<T>` (`Rc::downgrade`) | 순환 참조를 끊을 때. |
| 내부 가변성 | `mutable` 멤버 | (항상 가변) | `Cell<T>`, `RefCell<T>` (런타임 빌림 검사) | 규칙 위반 시 **panic**. |
| 공유 + 가변 | `shared_ptr<T>` + 락 | | `Rc<RefCell<T>>` (단일 스레드), `Arc<Mutex<T>>` (멀티스레드) | 남용 주의. 02 문서 참고. |
| 소유권 이동 | `std::move(x)` (원본은 "유효하지만 미정") | 참조 복사 | `let b = a;` (원본 **사용 불가**) | 기본이 이동. |
| 복사 | 기본이 복사 | 얕은 복사 | `a.clone()` (명시적), `Copy` 타입은 자동 | |
| 소멸 시점 | 스코프 끝 (RAII) | GC/참조카운트 | 스코프 끝 (`Drop`) | C++과 같은 RAII, 순서는 선언의 역순. |
| 명시적 해제 | `delete` / 스코프 | `del x` | `drop(x)` | 빠르게 소유권을 끝내고 싶을 때. |
| raw 포인터 | `T*` | | `*const T` / `*mut T` (`unsafe`에서만 역참조) | 일반 코드에서는 쓸 일이 없어요. |
| 힙 배열 | `std::vector<T>` | `list` | `Vec<T>` | |
| 메모리 오류 | UB (use-after-free 등) | 없음 | **컴파일 에러** (`unsafe` 밖에서) | Rust의 핵심 약속. |

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>), // 재귀 타입은 Box 필요
}

fn eval(e: &Expr) -> f64 {
    match e {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
    }
}

struct Node {
    name: String,
    parent: RefCell<Weak<Node>>, // 부모는 약한 참조로 순환 방지
    children: RefCell<Vec<Rc<Node>>>,
}

fn main() {
    let e = Expr::Add(Box::new(Expr::Num(1.5)), Box::new(Expr::Num(2.0)));
    println!("{}", eval(&e));

    let a = Rc::new(String::from("shared"));
    let b = Rc::clone(&a);
    println!("{} {}", Rc::strong_count(&a), b);

    let shared = Rc::new(RefCell::new(vec![1]));
    shared.borrow_mut().push(2);
    println!("{:?}", shared.borrow());

    let root = Rc::new(Node {
        name: "root".into(),
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    let leaf = Rc::new(Node {
        name: "leaf".into(),
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    *leaf.parent.borrow_mut() = Rc::downgrade(&root);
    root.children.borrow_mut().push(Rc::clone(&leaf));
    println!("{} <- {}", leaf.parent.borrow().upgrade().unwrap().name, leaf.name);
    println!("{}", root.children.borrow().len());
}
```

---

## 10. 참조와 빌림

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 읽기 참조 | `const T&` | (객체 참조) | `&T` | 여러 개 동시에 가능. |
| 쓰기 참조 | `T&` | (객체 참조) | `&mut T` | **한 번에 하나만**, 읽기 참조와 공존 불가. |
| 참조 만들기 | `T& r = x;` | | `let r = &x;`, `let r = &mut x;` | |
| 값 꺼내기 | `*p` | | `*r` | 메서드 호출은 자동 역참조라 `*`가 생략되곤 해요. |
| 널 참조 | 가능한 `T*` | `None` | **불가능** (`Option<&T>`) | |
| 댕글링 | UB | 없음 | **컴파일 에러** (E0597, E0106) | |
| 반복 중 수정 | iterator 무효화 (UB) | 런타임 예외/버그 | **컴파일 에러** (E0502) | |
| 슬라이스 | `std::span<T>` | `lst[a:b]` (복사) | `&v[a..b]`, `&[T]`, `&mut [T]` | 복사 없는 뷰. |
| 수명 표기 | (개념만 존재) | | `fn f<'a>(x: &'a str) -> &'a str` | 참조 사이의 **관계를 서술**. 대부분 생략(elision)돼요. |
| 구조체 안 참조 | 멤버 `const T&` / `string_view` | | `struct S<'a> { s: &'a str }` | 원본보다 오래 살 수 없어요. |
| `'static` | 정적 저장 기간 | | `&'static str` | "프로그램 끝까지 유효할 수 있다"는 뜻. 리터럴이 그래요. |

```rust
fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn double_all(xs: &mut [i32]) {
    for x in xs.iter_mut() {
        *x *= 2;
    }
}

struct Entry<'a> {
    level: &'a str,
    msg: &'a str,
}

fn parse(line: &str) -> Option<Entry<'_>> {
    let (level, msg) = line.split_once(' ')?;
    Some(Entry { level, msg })
}

fn main() {
    let text = String::from("hello big world");
    println!("{} {}", first_word(&text), longest("ab", "xyz"));

    let mut v = vec![1, 2, 3, 4];
    double_all(&mut v[1..3]); // 일부만 가변으로 빌려 주기
    println!("{v:?}");

    let line = String::from("INFO started ok");
    let e = parse(&line).unwrap();
    println!("{} | {}", e.level, e.msg);
}
```

---

## 11. 에러 처리

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 기본 방식 | 예외 `throw` / `try-catch` | 예외 `raise` / `try-except` | `Result<T, E>` 반환 | 에러가 **함수 시그니처에 보여요**. |
| 값 없음 | `std::optional<T>` | `None` | `Option<T>` | |
| 전파 | (예외가 자동 전파) | (예외가 자동 전파) | `let x = f()?;` | `?`가 `Err`면 즉시 반환. |
| 복구 불가 오류 | `std::terminate`, `assert` | `raise SystemExit`, `assert` | `panic!(..)`, `unreachable!()` | 라이브러리는 panic 대신 `Result`. |
| 값 꺼내기(위험) | `opt.value()` (예외) | | `.unwrap()`, `.expect("msg")` | 실패 시 panic. 프로토타입/테스트에서만. |
| 기본값 | `opt.value_or(0)` | `x or 0` | `opt.unwrap_or(0)`, `unwrap_or_else(\|\| ..)`, `unwrap_or_default()` | |
| 변환 | `std::transform`(C++23) | | `.map(\|x\| ..)`, `.and_then(..)`, `.ok_or(e)`, `.map_err(..)` | 콤비네이터. |
| 에러 타입 | `std::exception` 계층 | `Exception` 계층 | 직접 만든 `enum` + `impl Error` | `thiserror`가 보일러플레이트를 줄여 줘요. |
| 에러 변환 | `catch` 후 재던짐 | `raise X from e` | `impl From<A> for MyError` → `?`가 자동 변환 | |
| 앱 수준 에러 | | | `anyhow::Result<T>`, `.context("..")` | 앱: `anyhow`, 라이브러리: `thiserror`. |
| 에러 출력 | `e.what()` | `str(e)` | `{}`(Display), `{:?}`(Debug) | |
| 리소스 정리 | RAII | `with`/`finally` | `Drop` (RAII) | panic/`?` 반환에도 `Drop`은 실행됩니다. |

```rust
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    BadNumber(ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(k) => write!(f, "missing key: {k}"),
            ConfigError::BadNumber(e) => write!(f, "bad number: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::BadNumber(e)
    }
}

fn parse_port(line: &str) -> Result<u16, ConfigError> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| ConfigError::Missing("=".to_string()))?;
    if key.trim() != "port" {
        return Err(ConfigError::Missing("port".to_string()));
    }
    let port = value.trim().parse::<u16>()?; // ParseIntError -> ConfigError 자동 변환
    Ok(port)
}

fn main() {
    for line in ["port = 8080", "port = abc", "host = x", "nothing"] {
        match parse_port(line) {
            Ok(p) => println!("ok {p}"),
            Err(e) => println!("error: {e}"),
        }
    }
    let maybe: Option<i32> = None;
    println!("{}", maybe.map(|x| x * 2).unwrap_or(-1));
}
```

---

## 12. 컬렉션

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 동적 배열 | `std::vector<T>` | `list` | `Vec<T>`, `vec![1, 2, 3]` | |
| 추가/제거 | `push_back`, `pop_back` | `append`, `pop` | `push`, `pop` (→ `Option`) | |
| 접근 | `v[i]`, `v.at(i)` | `v[i]` | `v[i]` (범위 밖 panic), `v.get(i)` (→ `Option`) | 안전하게 접근하려면 `get`. |
| 크기 | `v.size()` | `len(v)` | `v.len()`, `v.is_empty()` | |
| 마지막 | `v.back()` | `v[-1]` | `v.last()` (→ `Option<&T>`) | 음수 인덱스 없음. |
| 정렬 | `std::sort(v.begin(), v.end())` | `v.sort()` | `v.sort()`, `v.sort_by_key(\|x\| ..)`, `v.sort_by(\|a, b\| a.partial_cmp(b).unwrap())` | `f64`는 `Ord`가 아니라 `partial_cmp`. |
| 해시맵 | `std::unordered_map<K,V>` | `dict` | `HashMap<K, V>` | 순서 **보장 없음**. 출력할 땐 정렬! |
| 정렬 맵 | `std::map<K,V>` | (dict 삽입 순서) | `BTreeMap<K, V>` | 키 순 정렬. |
| 집합 | `std::unordered_set<T>` | `set` | `HashSet<T>`, `BTreeSet<T>` | |
| 맵 조회 | `m.find(k)`, `m[k]` | `m.get(k)`, `m[k]` | `m.get(&k)` (→ `Option<&V>`), `m[&k]` (없으면 panic) | |
| 없으면 삽입 | `m.try_emplace`, `m[k]++` | `defaultdict`, `setdefault` | `*m.entry(k).or_insert(0) += 1;` | **entry API**. |
| 큐/덱 | `std::deque` | `collections.deque` | `VecDeque<T>` | |
| 우선순위 큐 | `std::priority_queue` | `heapq` | `BinaryHeap<T>` | 최대 힙. `Reverse`로 최소 힙. |
| 부분 | `std::span` | `v[a:b]` | `&v[a..b]` | |
| 중복 제거 | `std::unique` | `list(set(v))` | `v.dedup()` (정렬 후) | |

```rust
use std::collections::{BTreeMap, HashMap, HashSet};

fn main() {
    let mut v = vec![5, 3, 9, 1];
    v.push(7);
    v.sort();
    println!("{:?} {:?} {:?}", v, v.get(10), v.last());

    let mut counts: HashMap<String, i32> = HashMap::new();
    for w in "a b a c b a".split_whitespace() {
        *counts.entry(w.to_string()).or_insert(0) += 1;
    }
    let mut keys: Vec<_> = counts.iter().collect();
    keys.sort(); // 해시맵은 순서가 없으니 정렬해서 출력
    println!("{:?}", keys);

    let sorted: BTreeMap<_, _> = counts.into_iter().collect();
    println!("{:?}", sorted);

    let set: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();
    println!("{}", set.len());

    let mut fl = vec![2.5, 1.0, 9.9];
    fl.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:?}", fl);
}
```

---

## 13. 이터레이터 / ranges / 컴프리헨션

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 변환 | `v \| std::views::transform(f)` | `[f(x) for x in v]` | `v.iter().map(f)` | **지연 평가**. `collect()`가 있어야 실행돼요. |
| 거르기 | `views::filter(p)` | `[x for x in v if p(x)]` | `.filter(\|x\| p(x))` | |
| 수집 | `std::ranges::to<vector>` (C++23) | `list(...)` | `.collect::<Vec<_>>()` | 타입 표기가 필요. |
| 합계 | `std::accumulate` | `sum(v)` | `.sum::<i32>()` | |
| 접기 | `std::accumulate(.., init, op)` | `functools.reduce` | `.fold(init, \|acc, x\| ..)` | |
| 개수 | `std::count_if` | `sum(1 for ..)` | `.filter(..).count()` | |
| 있나/모두 | `any_of` / `all_of` | `any(..)` / `all(..)` | `.any(..)` / `.all(..)` | |
| 찾기 | `std::find_if` | `next((x for ..), None)` | `.find(..)` → `Option` | |
| 위치 | `std::distance(..)` | `v.index(x)` | `.position(..)` → `Option<usize>` | |
| 최대/최소 | `std::max_element` | `max(v)` | `.max()`, `.min()`, `.max_by_key(..)` | `f64`는 `fold(f64::MIN, f64::max)`. |
| 인덱스 | | `enumerate(v)` | `.enumerate()` | |
| 묶기 | `views::zip` (C++23) | `zip(a, b)` | `a.iter().zip(b.iter())` | |
| 앞에서 N개 | `views::take(n)` | `itertools.islice` | `.take(n)`, `.skip(n)` | |
| 평탄화 | `views::join` | `itertools.chain` | `.flat_map(..)`, `.flatten()`, `.chain(..)` | |
| 윈도우/청크 | | | `v.windows(2)`, `v.chunks(3)` | |
| 딕셔너리 컴프리헨션 | | `{k: f(v) for ..}` | `.map(\|..\| (k, f(v))).collect::<HashMap<_, _>>()` | |
| 종류 | | | `iter()` → `&T`, `iter_mut()` → `&mut T`, `into_iter()` → `T` | **값을 소비하는지** 구분. |
| 직접 만들기 | | `__iter__`/제너레이터 | `impl Iterator for X { type Item; fn next(&mut self) }` | |

```rust
use std::collections::HashMap;

fn main() {
    let v = vec![1, 2, 3, 4, 5, 6];

    let squares_of_evens: Vec<i32> = v.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).collect();
    let total: i32 = v.iter().sum();
    let product = v.iter().fold(1, |acc, x| acc * x);
    println!("{:?} {total} {product}", squares_of_evens);

    let names = ["a", "b", "c"];
    let pairs: Vec<(usize, &&str)> = names.iter().enumerate().collect();
    println!("{:?}", pairs);

    let map: HashMap<&str, usize> = names.iter().map(|n| (*n, n.len())).collect();
    println!("{}", map["a"]);

    let first_big = v.iter().find(|&&x| x > 3);
    let pos = v.iter().position(|&x| x == 4);
    println!("{:?} {:?} {}", first_big, pos, v.iter().any(|&x| x > 5));

    let max = [1.5_f64, 9.5, 3.0].iter().cloned().fold(f64::MIN, f64::max);
    println!("{max}");

    // 지연 평가: collect/sum/for 가 있어야 실제로 실행됩니다
    let lazy = v.iter().map(|x| x * 10);
    println!("{:?}", lazy.take(2).collect::<Vec<_>>());

    for w in v.windows(2).take(2) {
        println!("{:?}", w);
    }
}
```

> 💡 **쉬운 설명**: Python의 리스트 컴프리헨션은 곧바로 리스트를 만들지만, Rust의 `map`/`filter`는 "이런 가공을 할 거예요"라는 **계획서**만 만듭니다. `collect()`나 `for`로 끝을 내야 실제로 돌아가요. C++20 ranges의 view와 같은 개념입니다.

---

## 14. 클로저 / 람다

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 기본 | `[](int x){ return x+1; }` | `lambda x: x + 1` | `\|x\| x + 1` | 타입은 대부분 추론. |
| 캡처(참조) | `[&]` | (자동, 변수 공유) | 기본: 필요한 만큼 **자동으로 빌림** | |
| 캡처(복사/이동) | `[=]`, `[x = std::move(x)]` | | `move \|\| ..` | 소유권을 클로저 안으로 이동. 스레드에 필수. |
| 호출 가능 타입 | `std::function<int(int)>` | `Callable` | `Fn`, `FnMut`, `FnOnce` | 아래 설명 참고. |
| 인자로 받기 | `template<class F>` | `f: Callable` | `fn apply(f: impl Fn(i32) -> i32)` | |
| 반환 | `auto` / `std::function` | | `fn make() -> impl Fn(i32) -> i32`, `Box<dyn Fn(i32) -> i32>` | 클로저마다 타입이 달라서 `impl`/`Box` 필요. |
| 상태 변경 | `mutable` 람다 | `nonlocal` | `FnMut` 클로저 (`let mut c = \|\| count += 1;`) | |
| 한 번만 호출 | | | `FnOnce` (캡처한 값을 소비) | |

> 💡 **쉬운 설명**: `Fn` = 몇 번이든 호출 가능(읽기만), `FnMut` = 몇 번이든 호출 가능하지만 내부 상태를 바꿈, `FnOnce` = 한 번만 호출 가능(가져온 값을 써서 없앰). 컴파일러가 클로저의 내용을 보고 자동으로 정해 줍니다.

```rust
fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n // n 을 클로저로 이동(복사)
}

fn make_boxed(kind: &str) -> Box<dyn Fn(i32) -> i32> {
    match kind {
        "double" => Box::new(|x| x * 2),
        _ => Box::new(|x| x),
    }
}

fn call_twice(mut f: impl FnMut()) {
    f();
    f();
}

fn consume(f: impl FnOnce() -> String) -> String {
    f()
}

fn main() {
    let offset = 10;
    println!("{}", apply(|x| x + offset, 5)); // offset 을 빌림
    println!("{}", make_adder(3)(4));
    println!("{}", make_boxed("double")(21));

    let mut count = 0;
    call_twice(|| count += 1);
    println!("{count}");

    let s = String::from("moved");
    println!("{}", consume(move || s)); // s 를 소비
}
```

---

## 15. 모듈 / 헤더 / 패키지

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 선언/정의 분리 | `.h` + `.cpp` | (없음) | **없음** (한 파일에 정의) | 헤더 파일이 필요 없어요. |
| 파일 가져오기 | `#include "x.h"` | `import x` | `mod x;` (선언) + `use` (이름 가져오기) | `mod`가 파일을 트리에 **등록**해요. |
| 이름 가져오기 | `using std::string;` | `from m import f` | `use std::collections::HashMap;` | |
| 네임스페이스 | `namespace a { }` | 모듈/패키지 | `mod a { }` | 파일 `a.rs` 또는 `a/mod.rs`가 `mod a`. |
| 공개 | `public:` / 헤더에 선언 | (관례 `_x`) | `pub`, `pub(crate)` | **기본은 비공개**. |
| 별칭 | `using A = B;` | `import x as y` | `use x as y;`, `type A = B;` | |
| 상위/같은 크레이트 | | `from . import` | `super::x`, `crate::x`, `self::x` | |
| 배포 단위 | 라이브러리(.a/.so) | 패키지(wheel) | **crate** (`lib.rs` 또는 `main.rs`) | |
| 의존성 | CMake `find_package`/vcpkg/conan | `pip` + requirements | `Cargo.toml`의 `[dependencies]` | 버전 고정은 `Cargo.lock`. |
| 매크로 | `#define` | | `macro_rules!`, 호출은 `name!(..)` | 전처리기와 달리 문법 단위로 안전. |
| 조건부 컴파일 | `#ifdef` | | `#[cfg(test)]`, `#[cfg(target_os = "linux")]`, Cargo features | |
| 프로그램 시작 | `int main()` | `if __name__ == "__main__"` | `fn main()` (`src/main.rs`) | |

```rust
mod geometry {
    pub const UNIT: f64 = 1.0;

    pub mod shapes {
        pub struct Square(pub f64);
        impl Square {
            pub fn area(&self) -> f64 {
                helper(self.0) * super::UNIT
            }
        }
        fn helper(x: f64) -> f64 { x * x } // 비공개: 이 모듈 안에서만
    }

    pub(crate) fn describe() -> &'static str { "geometry" }
}

use geometry::shapes::Square;

fn main() {
    let s = Square(3.0);
    println!("{} {}", s.area(), geometry::describe());
}
```

실제 프로젝트에서는 `mod geometry;` 한 줄만 쓰고 내용은 `src/geometry.rs`(또는 `src/geometry/mod.rs`)에 둡니다. 이 구조는 ch10에서 직접 만들어 봅니다.

---

## 16. 빌드 도구

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 새 프로젝트 | 직접 `CMakeLists.txt` 작성 | `python -m venv` + 파일 | `cargo new myapp` | 디렉터리와 `Cargo.toml` 자동 생성. |
| 빌드 | `cmake --build build` | (불필요) | `cargo build` / `cargo build --release` | release는 최적화 (`-O2`급). |
| 실행 | `./build/app` | `python main.py` | `cargo run` / `cargo run --example name` | |
| 빠른 검사 | `-fsyntax-only` | `pyright`/`mypy` | `cargo check` | 코드 생성을 생략해 빠릅니다. |
| 의존성 추가 | vcpkg/conan/FetchContent | `pip install x` | `cargo add serde` | `Cargo.toml`에 기록됨. |
| 의존성 잠금 | (직접) | `requirements.txt`, `uv.lock` | `Cargo.lock` | 앱은 커밋, 라이브러리는 선택. |
| 포맷터 | `clang-format` | `black`, `ruff format` | `cargo fmt` (rustfmt) | 설정 논쟁 거의 없음. |
| 린터 | `clang-tidy` | `ruff`, `pylint` | `cargo clippy` | 이 저장소 기준: `cargo clippy -- -D warnings`. |
| 문서 | Doxygen | Sphinx | `cargo doc --open` | `///` 문서 주석이 HTML이 됨. |
| 다중 프로젝트 | CMake `add_subdirectory` | | **workspace** (`[workspace] members`) | 이 저장소가 workspace예요. |
| 패키지 지정 | | | `cargo run -p ch03_ownership` | `-p`는 workspace의 특정 패키지. |
| 디버그/릴리스 | `-O0 -g` / `-O2` | | 기본 debug(느림, 오버플로 검사) / `--release` | 성능 측정은 반드시 release. |
| 컴파일 플래그 | `target_compile_options` | | `Cargo.toml`의 `[profile.release]` | |

```toml
# Cargo.toml 한 눈에 보기
[package]
name = "myapp"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
anyhow = "1"
```

---

## 17. 테스트

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 프레임워크 | GoogleTest / Catch2 (외부) | `pytest` / `unittest` | 내장 (`#[test]`) | 별도 설치 없음. |
| 테스트 함수 | `TEST(Suite, Name) { }` | `def test_x():` | `#[test] fn works() { }` | |
| 단언 | `EXPECT_EQ(a, b)` | `assert a == b` | `assert_eq!(a, b)`, `assert!(c)`, `assert_ne!` | 실패 시 좌우 값을 출력해요. |
| 예외 기대 | `EXPECT_THROW` | `pytest.raises` | `#[should_panic]`, 또는 `assert!(r.is_err())` | |
| 위치 | 별도 파일 | `tests/` | 단위: 같은 파일의 `#[cfg(test)] mod tests`, 통합: `tests/` | |
| 실행 | `ctest` | `pytest` | `cargo test`, `cargo test name` | `-- --nocapture`로 println 출력 보기. |
| 문서 테스트 | | `doctest` | `///` 안의 코드 블록이 자동으로 테스트 | |
| Result 반환 | | | `fn t() -> Result<(), Box<dyn Error>> { .. Ok(()) }` | 테스트 안에서 `?` 사용. |
| 이 저장소 | | | `cargo test -p chNN_slug` | 풀기 전에는 `todo!()` 때문에 실패하는 게 정상. |

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("{}", add(2, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn panics_on_overflow() {
        let _ = 255u8.checked_add(1).expect("overflow");
    }

    #[test]
    fn can_use_question_mark() -> Result<(), std::num::ParseIntError> {
        let n: i32 = "12".parse()?;
        assert_eq!(n, 12);
        Ok(())
    }
}
```

---

## 18. 동시성

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 스레드 생성 | `std::thread t(f);` / `std::jthread` | `threading.Thread` | `thread::spawn(move \|\| ..)` | 클로저가 `'static` + `Send`여야 해요. |
| 합류 | `t.join()` | `t.join()` | `handle.join().unwrap()` | 결과값(`T`)도 받아요. |
| 지역 변수 빌려 쓰기 | (위험) | | `thread::scope(\|s\| { s.spawn(..); })` | `'static` 제약 없음. |
| 뮤텍스 | `std::mutex` + `lock_guard` | `threading.Lock` | `Mutex<T>` → `lock().unwrap()` | **데이터가 뮤텍스 안에** 들어 있어 락 없이 접근 불가. |
| 공유 소유 | `shared_ptr<T>` | (모든 객체가 공유) | `Arc<T>` | `Arc<Mutex<T>>`가 기본 조합. |
| 읽기/쓰기 락 | `std::shared_mutex` | | `RwLock<T>` | |
| 채널 | (직접 큐 + condvar) | `queue.Queue` | `mpsc::channel()` → `(Sender, Receiver)` | 송신자는 `clone`으로 여러 개. |
| 원자 변수 | `std::atomic<int>` | | `AtomicUsize` | |
| 데이터 레이스 | UB | GIL이 일부 방지 | **컴파일 에러** (`Send`/`Sync`) | Rust의 핵심 약속 2. |
| CPU 병렬 | `std::execution::par` | `multiprocessing` (GIL 회피) | `rayon` 크레이트 (`par_iter`) | 이 커리큘럼 범위 밖. |
| 마커 트레이트 | | | `Send`: 다른 스레드로 이동 가능, `Sync`: 여러 스레드가 `&T` 공유 가능 | `Rc`는 둘 다 아님. |

```rust
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Arc<Mutex<T>>: 여러 스레드가 하나의 카운터를 안전하게 증가
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..4 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                *c.lock().unwrap() += 1;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("{}", *counter.lock().unwrap());

    // 채널: 워커가 결과를 보내고 메인이 모음
    let (tx, rx) = mpsc::channel();
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || tx.send((id, id * id)).unwrap());
    }
    drop(tx); // 원본 송신자를 닫아야 수신 루프가 끝나요
    let mut results: Vec<(i32, i32)> = rx.iter().collect();
    results.sort(); // 스레드 순서는 비결정적이므로 정렬
    println!("{:?}", results);

    // scope: 지역 변수를 빌려서 스레드에 전달
    let data = vec![1, 2, 3, 4];
    let (left, right) = data.split_at(2);
    let (a, b) = thread::scope(|s| {
        let h1 = s.spawn(|| left.iter().sum::<i32>());
        let h2 = s.spawn(|| right.iter().sum::<i32>());
        (h1.join().unwrap(), h2.join().unwrap())
    });
    println!("{a} {b}");
}
```

---

## 19. 비동기

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 비동기 함수 | 코루틴 `co_await` | `async def` | `async fn f() -> T` | 호출하면 **Future(할 일 상자)**만 만들어요. |
| 기다리기 | `co_await x` | `await x` | `x.await` | `.await`가 있어야 실행이 진행돼요 (후위 표기). |
| 런타임 | (표준에 없음) | `asyncio.run(main())` | `#[tokio::main] async fn main()` | **런타임(실행기)을 직접 고름**: 사실상 `tokio`. |
| 동시 실행 | | `asyncio.gather(a, b)` | `tokio::join!(a, b)` | |
| 백그라운드 작업 | | `asyncio.create_task` | `tokio::spawn(async { .. })` | `JoinHandle`을 `.await`. |
| 여러 작업 모으기 | | `TaskGroup` | `tokio::task::JoinSet` | |
| 먼저 끝난 것 | | `asyncio.wait(FIRST_COMPLETED)` | `tokio::select!` | |
| 타임아웃 | | `asyncio.timeout(1)` | `tokio::time::timeout(dur, fut).await` → `Result` | |
| 대기 | | `await asyncio.sleep(1)` | `tokio::time::sleep(dur).await` | `std::thread::sleep`은 **런타임을 막아요**. |
| 지연 실행 | 코루틴은 보통 lazy | 코루틴도 lazy | Future는 **await/spawn 전엔 아무 일도 안 함** | Python과 매우 비슷. |
| 블로킹 작업 | | `run_in_executor` | `tokio::task::spawn_blocking` | async 안에서 오래 걸리는 동기 코드는 이쪽으로. |

```rust
use std::time::Duration;
use tokio::time::{sleep, timeout};

async fn fetch(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await; // 양보: 다른 작업이 실행됨
    format!("result {id}")
}

#[tokio::main]
async fn main() {
    // 동시 실행
    let (a, b) = tokio::join!(fetch(1, 30), fetch(2, 10));
    println!("{a} / {b}");

    // spawn: 백그라운드 작업
    let h = tokio::spawn(async { fetch(3, 5).await });
    println!("{}", h.await.unwrap());

    // 타임아웃
    let slow = timeout(Duration::from_millis(20), fetch(4, 500)).await;
    println!("{}", slow.is_err());

    // JoinSet: 여러 작업을 한꺼번에
    let mut set = tokio::task::JoinSet::new();
    for i in 0..3 {
        set.spawn(fetch(i, 5));
    }
    let mut out = vec![];
    while let Some(r) = set.join_next().await {
        out.push(r.unwrap());
    }
    out.sort(); // 끝나는 순서는 비결정적
    println!("{:?}", out);
}
```

> ⚠️ **흔한 실수**: `async fn` 안에서 `std::thread::sleep`이나 무거운 계산, 동기 파일 읽기를 오래 하면 같은 런타임의 다른 작업이 전부 멈춥니다. 02 문서의 "async 안에서 블로킹" 항목을 보세요.

---

## 20. JSON

| 주제 | C++20 (nlohmann) | Python | Rust (serde_json) | 한 줄 메모 |
|---|---|---|---|---|
| 파싱(동적) | `json::parse(s)` | `json.loads(s)` | `serde_json::from_str::<Value>(s)?` | `Value`는 파이썬 dict처럼 느슨한 타입. |
| 파싱(타입 있음) | `j.get<T>()` | `Model(**d)` / dataclass | `serde_json::from_str::<T>(s)?` | 필드가 없거나 타입이 틀리면 **Err**. |
| 직렬화 | `j.dump()` | `json.dumps(o)` | `serde_json::to_string(&v)?` | |
| 보기 좋게 | `j.dump(2)` | `json.dumps(o, indent=2)` | `serde_json::to_string_pretty(&v)?` | 2칸 들여쓰기. |
| 접근 | `j["a"]["b"]` | `d["a"]["b"]` | `v["a"]["b"]` (없으면 `Null`), `v.get("a")` | |
| 매크로 | `json{{"a", 1}}` | 딕셔너리 리터럴 | `serde_json::json!({"a": 1})` | |
| 타입 연결 | `NLOHMANN_DEFINE_TYPE_INTRUSIVE` | `@dataclass` | `#[derive(Serialize, Deserialize)]` | 한 줄로 끝. |
| 이름 변경 | | | `#[serde(rename = "type")]`, `#[serde(rename_all = "snake_case")]` | |
| 선택 필드 | `j.value("k", def)` | `d.get("k")` | `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`, `#[serde(default)]` | |
| 태그 있는 합 타입 | 수동 | 수동 | `#[serde(tag = "type")] enum Block { .. }` | Messages API의 content block 모양. |
| 키 순서 | 기본 정렬(`ordered_json`은 삽입 순서) | 삽입 순서 | 기본 정렬(`preserve_order` 기능은 삽입 순서) | 출력 비교 시 주의. |

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<Block>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

fn main() -> Result<(), serde_json::Error> {
    let raw = r#"{
        "role": "assistant",
        "content": [
            {"type": "text", "text": "hi"},
            {"type": "tool_use", "id": "t1", "name": "calc", "input": {"expr": "1+2"}}
        ]
    }"#;
    let msg: Message = serde_json::from_str(raw)?; // note 는 없어도 OK (default)
    println!("{:?}", msg.content[0]);
    println!("{}", serde_json::to_string(&msg)?);

    let v: Value = serde_json::from_str(raw)?; // 타입 없이 파싱
    println!("{}", v["content"][1]["name"]);

    let built = json!({ "model": "x", "max_tokens": 16 });
    println!("{}", serde_json::to_string_pretty(&built)?);

    let bad: Result<Message, _> = serde_json::from_str(r#"{"role": 1}"#);
    println!("{}", bad.is_err());
    Ok(())
}
```

---

## 21. HTTP

| 주제 | C++20 | Python | Rust | 한 줄 메모 |
|---|---|---|---|---|
| 라이브러리 | `libcurl` (C API) | `urllib` / `requests` / `httpx` | `reqwest` (비동기 기반) | 이 저장소: `reqwest` + `tokio`. |
| GET | `curl_easy_perform` 설정 다수 | `requests.get(url)` | `client.get(url).send().await?` | |
| JSON 본문 | 수동 직렬화 + 헤더 | `json=payload` | `.json(&payload)` | `Serialize`를 구현한 값. |
| JSON 응답 | 문자열 → 파싱 | `r.json()` | `resp.json::<T>().await?` | |
| 상태 코드 | `CURLINFO_RESPONSE_CODE` | `r.status_code`, `raise_for_status()` | `resp.status()`, `resp.error_for_status()?` | 4xx/5xx는 **자동으로 Err가 아니에요**. 확인 필요. |
| 헤더 | `curl_slist_append` | `headers={..}` | `.header("x-api-key", key)` | |
| 타임아웃 | `CURLOPT_TIMEOUT` | `timeout=5` | `Client::builder().timeout(dur).build()?` | |
| 재시도 | 직접 | `urllib3.Retry` 등 | 직접(루프 + `sleep`) 또는 미들웨어 | ch14/ch16에서 직접 구현. |
| 환경변수 | `getenv` | `os.environ.get` | `std::env::var("API_KEY")` → `Result` | 키를 코드에 쓰지 마세요. |
| 에러 | `CURLcode` | 예외 | `reqwest::Error` → `anyhow`/`thiserror`로 감싸기 | |

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct NewTodo<'a> {
    title: &'a str,
}

#[derive(Debug, Deserialize)]
struct Todo {
    id: u32,
    title: String,
}

async fn create_todo(client: &reqwest::Client, base: &str, title: &str) -> anyhow::Result<Todo> {
    let resp = client
        .post(format!("{base}/todos"))
        .json(&NewTodo { title })
        .send()
        .await?
        .error_for_status()?; // 4xx/5xx 를 Err 로 바꿈
    Ok(resp.json::<Todo>().await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    match create_todo(&client, &base, "learn rust").await {
        Ok(t) => println!("{} {}", t.id, t.title),
        Err(e) => println!("request failed: {e}"), // 서버가 없으면 여기로 옵니다
    }
    Ok(())
}
```

> 이 예제는 컴파일만 확인했고, 실제 실행에는 `tools/mock_server.py`가 떠 있어야 합니다(ch16, ch17에서 `scripts/compare.sh`가 자동으로 켜 줍니다).

---

## 22. MATLAB 사용자를 위한 메모

MATLAB 경험이 있다면 아래 세 가지 차이만 먼저 기억해 두세요.

| 주제 | MATLAB | Rust | 메모 |
|---|---|---|---|
| 인덱스 시작 | **1부터** (`a(1)`) | **0부터** (`a[0]`) | C++/Python과 같아요. 마지막 원소는 `a(end)` 대신 `a[a.len() - 1]` 또는 `a.last()`. |
| 인덱스 문법 | `a(2)` (소괄호) | `a[2]` (대괄호) | 소괄호는 함수 호출. |
| 범위/슬라이스 | `a(2:4)` (**끝 포함**) | `&a[1..4]` (**끝 제외**), `&a[1..=3]` (끝 포함) | 슬라이스는 복사가 아니라 **뷰**이고, 범위 밖이면 panic. |
| 열 우선 vs 행 우선 | 열 우선 메모리 | 행 우선 (`Vec<Vec<T>>`) | 행렬 연산은 `ndarray`/`nalgebra` 크레이트를 쓰세요(이 커리큘럼 밖). |
| 벡터화 연산 | `y = x .* 2 + 1` | `x.iter().map(\|v\| v * 2.0 + 1.0).collect::<Vec<_>>()` | 연산자가 원소 단위로 자동 적용되지 않아요. 이터레이터로 씁니다. |
| 합/평균 | `sum(x)`, `mean(x)` | `x.iter().sum::<f64>()`, `sum / x.len() as f64` | 평균은 직접. |
| 조건 선택 | `x(x > 0)` | `x.iter().filter(\|&&v\| v > 0.0)` | |
| 범위 생성 | `0:5:100` | `(0..=100).step_by(5)` | |
| 값 전달 | 복사(copy-on-write) | 이동 / 빌림 | 함수에 큰 배열을 넘겨도 복사되지 않고 소유권이 이동해요. |
| 동적 크기 | 배열이 자동 확장 | `Vec::push` | 고정 `[T; N]`은 확장 불가. |
| 타입 | 기본 double 행렬 | 정적 타입 | 정수와 실수를 섞으려면 `as`. |

```rust
fn main() {
    let x = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];

    // y = x .* 2 + 1
    let y: Vec<f64> = x.iter().map(|v| v * 2.0 + 1.0).collect();

    // x(2:4) -> MATLAB 은 끝 포함 / Rust 는 [1..4] 로 끝 제외
    let part = &x[1..4];

    // mean(x)
    let mean = x.iter().sum::<f64>() / x.len() as f64;

    // x(x > 2)
    let big: Vec<f64> = x.iter().copied().filter(|&v| v > 2.0).collect();

    // 0:5:20
    let grid: Vec<i32> = (0..=20).step_by(5).collect();

    println!("{:?}\n{:?}\n{mean}\n{:?}\n{:?}", y, part, big, grid);
}
```

---

## 마무리: 이럴 땐 어디를 볼까요

- 규칙을 어겨서 컴파일이 안 될 때: [03. 컴파일러 에러 읽는 법](03_reading_compiler_errors.md)
- AI가 만든 코드를 검토할 때: [02. AI 에이전트와 Rust 바이브 코딩](02_vibe_coding_with_rust.md)
- 환경 설정이 막혔을 때: [00. 실습 환경 준비](00_environment_setup.md)
- 더 깊이: [The Rust Book](https://doc.rust-lang.org/book/), [Rust by Example](https://doc.rust-lang.org/rust-by-example/), [std 문서](https://doc.rust-lang.org/std/)
