# Ch02. 변수·타입·표현식·제어 흐름

> 변수는 기본이 불변, `if`/`match`/`loop`/블록은 전부 "값을 돌려주는 식", 정수는 넘치면 안전장치가 있다.  ·  예상 소요: 이론 50분 / 실습 60분  ·  선행: Ch01

## 🎯 학습 목표

- `let` / `let mut` / `const` / shadowing 의 차이를 설명하고 상황에 맞게 고른다.
- 정수·실수·`bool`·`char` 스칼라 타입과 튜플·배열을 선언하고, `as` / `From` / `TryFrom` 변환의 차이를 안다.
- **"식(expression)"과 "문(statement)"** 을 구분하고, 세미콜론 한 글자가 코드의 의미를 바꾼다는 것을 이해한다.
- `if` / `match` / `loop` / 블록이 값을 돌려주는 식이라는 점을 활용하고, `match` 의 **빠짐없는 검사(exhaustiveness)** 를 이해한다.
- 정수 오버플로가 debug 와 release 에서 어떻게 다른지 알고, `checked_` / `wrapping_` / `saturating_` 로 의도를 표현한다.

---

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 변수 기본값 | **가변** (`const` 를 붙이면 불변) | 가변 | **불변** (`mut` 를 붙이면 가변) |
| 같은 이름 다시 선언 | 같은 스코프에서는 에러 | 그냥 재대입 | `let x = ...` 를 또 쓰면 **shadowing**(새 변수, 타입 변경 가능) |
| 상수 | `constexpr int N = 5;` | `N = 5` (관례) | `const N: i32 = 5;` (타입 필수, 대문자) |
| 타입 추론 | `auto` | 항상 동적 | `let x = 5;` (함수 시그니처는 명시) |
| 정수 타입 | `int`, `long` (플랫폼별 크기), `uint8_t`… | `int` (무한 정밀도) | `i8 i16 i32 i64 i128 isize` / `u8 … usize` (크기 고정) |
| 정수 오버플로 | 부호 있음: **UB** / 부호 없음: 감김 | 없음 (무한 정밀도) | debug: **패닉** / release: 감김 (둘 다 UB 아님) |
| 형 변환 | 암묵 변환 + `static_cast` | `int(x)`, `float(x)` | **항상 명시**: `x as u8`, `u64::from(x)`, `u8::try_from(x)?` |
| 정수 나눗셈 `-7 / 2` | `-3` (0 쪽으로 버림) | `-7 // 2 == -4` (**내림**) | `-3` (C++ 과 같음) |
| 나머지 `-7 % 3` | `-1` | `2` (피제수가 아닌 제수 부호) | `-1` (+ `rem_euclid` 로 `2`) |
| 튜플 | `std::tuple`, `std::pair` | `(1, "a")` | `(1, "a")`, `t.0` |
| 고정 길이 배열 | `std::array<int, 3>` | (list 는 가변 길이) | `[i32; 3]` |
| 삼항 연산자 | `c ? a : b` | `a if c else b` | `if c { a } else { b }` (**if 자체가 식**) |
| 분기 | `switch` (fallthrough, 빠뜨려도 OK) | `match` (3.10+, 빠뜨려도 OK) | `match` (**빠뜨리면 컴파일 에러**) |
| 바깥 루프 탈출 | `goto` / 플래그 | 함수 분리 / 플래그 | `'label: for … { break 'label; }` |

---

## 📘 이론

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있어요
> (조각이 `fn main` 없이 몇 줄만 있다면 `fn main() { ... }` 안에 넣으세요).

### 1. 변수: `let`, `mut`, shadowing, `const`

```rust
fn main() {
    let x = 5;          // 불변. 타입은 i32 로 추론
    // x = 6;           // ❌ E0384: cannot assign twice to immutable variable
    let mut y = 5;      // 가변
    y += 1;             // ✅

    // shadowing: 같은 이름으로 "새 변수"를 만든다 (옛 변수는 가려질 뿐)
    let z = 5;
    let z = z * 2;              // i32 10
    let z = format!("{z} 개");   // String "10 개"  <- 타입이 바뀌어도 OK
    println!("{z}");
}

const MAX_RETRIES: u32 = 3;     // 컴파일 타임 상수. 타입 필수, 어디서나(함수 밖에서도) 선언 가능
```

- **기본이 불변**인 이유: "이 값은 안 바뀐다"는 정보가 코드에 박혀 있으면 읽는 사람(과 AI, 컴파일러)이 추론하기 쉽습니다. 바뀌는 값만 `mut` 로 눈에 띄게 만드는 것이죠.
- `mut` 와 shadowing 의 차이: `mut` 는 같은 변수의 값을 바꾸고 **타입은 못 바꿉니다**. shadowing 은 **새 변수**라서 타입을 바꿀 수 있어요.
  흔한 용도는 `let input = input.trim();` 처럼 "같은 의미의 값을 단계별로 가공"하는 것.

> 💡 **쉬운 설명 — shadowing(그림자 가리기)**
> 같은 이름표를 새 상자에 옮겨 붙이는 것과 같아요. 옛 상자는 사라지지 않지만 이름표가 없어서 더는 부를 수 없게 됩니다.
> 반면 `mut` 는 같은 상자 안의 내용물을 바꾸는 것이라, 상자 모양(타입)은 그대로여야 합니다.

> ⚠️ **C++ 개발자가 헷갈리는 점**
> C++ 의 기본은 "가변, `const` 를 붙여 잠금"이고 Rust 는 정반대입니다. 그래서 Rust 에서는 `mut` 가 없는 것이 정상이고, `mut` 가 많은 코드가 이상 신호예요.
> (Rust 의 `let` 은 `const auto` 와 비슷하고, `let mut` 이 `auto` 입니다.)

### 2. 스칼라 타입

| 분류 | 타입 | 메모 |
|---|---|---|
| 부호 있는 정수 | `i8 i16 i32 i64 i128 isize` | **기본 정수 리터럴은 `i32`** |
| 부호 없는 정수 | `u8 u16 u32 u64 u128 usize` | 배열 인덱스·길이(`len()`)는 항상 **`usize`** |
| 실수 | `f32`, `f64` | **기본 실수 리터럴은 `f64`** |
| 논리 | `bool` | `true` / `false` (정수와 섞이지 않음) |
| 문자 | `char` | **4바이트 유니코드 문자** 하나 (`'가'`, `'A'`). C++ `char`(1바이트)와 다름 |
| 단위 | `()` | "값 없음". C++ `void` 와 비슷하지만 **실제 값**이라 변수에 담을 수 있음 |

리터럴 표기: `1_000_000` (밑줄은 구분용), `0xff`, `0o17`, `0b1010`, `b'A'`(= `u8` 65), `2.5e3`, `10u8` / `10_u8` (접미사로 타입 지정).

- `isize` / `usize` 는 **포인터 폭**(64비트 OS 에서 64비트)이라 C++ 의 `ptrdiff_t` / `size_t` 와 비슷합니다.
- `i32` 로 인덱스하면 안 됩니다: `v[i]` 의 `i` 는 `usize` 여야 해요 (실제 에러는 아래 "자주 만나는 컴파일 에러" 참고).
- `u32` 에 `-1` 을 대입하면 컴파일 에러 (`E0600`). C++ 의 `unsigned x = -1;` 이 조용히 4294967295 가 되는 것과 대조적이죠.

### 3. 정수 오버플로

```rust
fn main() {
    let a: u8 = 250;
    println!("{:?}", a.checked_add(10));     // None            넘치면 None (가장 안전)
    println!("{}",   a.wrapping_add(10));    // 4               한 바퀴 돌아감 (해시/난수 등 의도적 사용)
    println!("{}",   a.saturating_add(10));  // 255             끝에서 멈춤 (밝기, 볼륨)
    println!("{:?}", a.overflowing_add(10)); // (4, true)       값과 "넘쳤는지" 를 같이
    // println!("{}", a + 10);               // debug: panic "attempt to add with overflow"
}
```

| | debug (`cargo build`) | release (`--release`) |
|---|---|---|
| `a + b` 가 넘치면 | **패닉** (프로그램 중단, 스택 트레이스) | 조용히 감김 |

- 컴파일러가 **미리 알 수 있는** 경우(`let x: u8 = 255; x + 1`)는 아예 컴파일 에러 (`this arithmetic operation will overflow`).
- C++ 와 비교: 부호 없는 정수는 C++ 도 감기는 것이 *표준*이지만, **부호 있는 정수 오버플로는 UB** 라 컴파일러가 최적화 때 "안 넘친다"고 가정해 버려요. Rust 는 UB 가 아니라 *정의된* 동작(패닉 or 감김)입니다.
- Python 의 `int` 는 크기 제한이 없어서 오버플로가 없어요. 대신 느리고 메모리가 늘어납니다. 이 챕터의 Python 코드는 `max_value` 인자로 "u8 / u32 라면 넘쳤을 지점"을 흉내 냅니다.

> 💡 **쉬운 설명 — 오버플로**
> 자동차 주행거리계가 99999 에서 00000 으로 돌아가는 것처럼, 정해진 칸 수(8비트 = 0~255)를 넘으면 값이 처음으로 돌아가는 현상이에요.
> Rust 는 개발 중(debug)에는 "넘쳤어요!" 하고 바로 멈춰서 버그를 알려 주고, 최종 배포(release)에서는 속도를 위해 검사를 끕니다.
> 그래서 *중요한 계산*(돈, 길이, 인덱스)은 `checked_*` 로 의도를 코드에 적어 두는 것이 좋습니다.

### 4. 형 변환: `as` vs `From` vs `TryFrom`

```rust
fn main() {
    let big: i32 = 300;
    let a = big as u8;               // 44   (하위 8비트만 남김. 컴파일러는 불평 안 함!)
    let b = -1_i32 as u8;            // 255
    let c = 3.99_f64 as i32;         // 3    (소수부 버림, 0 쪽으로)
    let d = 1e20_f64 as i32;         // 2147483647  (범위를 넘으면 포화! C++ 은 UB)
    let e = f64::NAN as i32;         // 0
    let f = i64::from(7_i32);        // 7    손실 없는 변환만 From 이 구현됨 (실패 불가)
    let g = u8::try_from(300_i32);   // Err(TryFromIntError(PosOverflow))  실패할 수 있으면 Result
    let h = u8::try_from(200_i32);   // Ok(200)
    println!("{a} {b} {c} {d} {e} {f} {g:?} {h:?}");
}
```

| 방법 | 언제 | 안전성 |
|---|---|---|
| `x as T` | 의도적으로 자르거나 부호를 바꿀 때, 정수 ↔ 실수 | 조용히 값을 잃을 수 있음 |
| `T::from(x)` / `x.into()` | 손실 없는 확대 (`u8 → u32`, `i32 → f64`) | 컴파일 타임 보장 |
| `T::try_from(x)` | 줄어들 수 있는 변환 | `Result` 로 실패를 알려 줌 |

> ⚠️ **C++ 개발자가 헷갈리는 점**
> Rust 의 `as` 는 C++ 의 C 스타일 캐스트처럼 생겼지만 **정의된 동작만** 합니다 (UB 없음). 그래도 값을 조용히 자르니,
> AI 가 쓴 코드에서 `as u8`, `as usize` 가 보이면 "음수나 큰 값이 들어올 수 있나?"를 한 번 의심하세요. 리뷰 포인트 1순위입니다.

### 5. 튜플과 배열

```rust
fn main() {
    let t: (i32, f64, &str) = (1, 2.5, "hi");
    println!("{} {} {}", t.0, t.1, t.2);   // 인덱스는 .0 .1 .2 (점 표기)
    let (a, b, c) = t;                     // 구조 분해(destructuring): 한 번에 꺼내기
    println!("{a} {b} {c}");

    let arr = [10, 20, 30];                // [i32; 3]  타입에 "길이"가 들어 있음
    let zeros = [0; 5];                    // [0, 0, 0, 0, 0]
    println!("{} {:?} {:?}", arr.len(), arr, zeros);   // 3 [10, 20, 30] [0, 0, 0, 0, 0]
    let i = 1;                             // usize 로 추론 (인덱스로 쓰이니까)
    println!("{}", arr[i]);                // 20
    // println!("{}", arr[5]);             // ❌ 컴파일 에러: index out of bounds (상수 인덱스는 컴파일러가 잡음)
}
```

- 배열 인덱스는 **항상 경계 검사**를 합니다. 컴파일 때 알 수 없는 인덱스가 범위를 벗어나면 **패닉**(`index out of bounds: the len is 3 but the index is 5`)이에요.
  C++ 의 `a[5]` 가 UB 인 것과 달리, 틀린 메모리를 조용히 읽는 일은 없습니다.
- 길이를 모르는 가변 배열은 `Vec<T>` (7장). 튜플은 서로 다른 타입을 묶고, 배열은 같은 타입을 묶습니다.
- `()` 가 "빈 튜플"이라서, 값을 안 돌려주는 함수는 사실 `()` 를 돌려주는 거예요 (1장에서 본 `;` 이야기와 연결됩니다).

### 6. 식(expression)과 문(statement), 그리고 `;`

Rust 는 **거의 모든 것이 식**입니다. 식은 값을 만들고, `;` 는 그 값을 버립니다.

```rust
fn main() {
    // 1) 블록은 마지막 식이 값
    let block = {
        let a = 3;
        a * a          // 세미콜론 없음 -> 블록의 값 9
    };

    // 2) if 도 식 (두 갈래의 타입이 같아야 함)
    let n = 27;
    let label = if n > 20 { "big" } else { "small" };

    // 3) loop 도 식: break 에 값을 실어 보낼 수 있음 (while / for 는 불가)
    let mut counter = 0;
    let found = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2;     // loop 의 값이 10
        }
    };
    println!("{block} {label} {found}");   // 9 big 10
}
```

- `x * 2` 는 값이 `i32` 인 식, `x * 2;` 는 그 값을 버리는 **문** → 값이 `()`.
  함수 끝의 `;` 하나가 `E0308: expected i32, found ()` 를 만드는 이유예요 (1장).
- if/else 두 갈래의 타입이 다르면 에러 (`E0308: if and else have incompatible types`).
  Python 은 허용하지만, Rust 는 "이 변수의 타입이 뭐냐"에 항상 한 가지 답이 있기를 요구합니다.
- 삼항 연산자가 없는 이유: `if` 가 이미 식이기 때문에 필요 없습니다.

### 7. `match` — 빠짐없이 검사하는 switch

```rust
fn describe(n: u32) -> &'static str {
    match n {
        0 => "zero",
        1..=9 => "digit",                 // 범위 패턴 (끝 포함)
        10 | 20 | 30 => "round",          // 여러 값 (or 패턴)
        x if x % 2 == 0 => "even",        // 가드(조건): 패턴 + if
        _ => "odd",                       // 나머지 전부. 빠뜨리면 컴파일 에러!
    }
}
```

- `match` 는 값에 대해 위에서부터 패턴을 시험하고, **첫 번째로 맞는 갈래**만 실행합니다. fallthrough 도 `break` 도 없어요.
- **모든 가능성을 다뤄야 컴파일됩니다.** 나중에 `enum` 에 새 값이 추가되면 모든 `match` 가 컴파일 에러로 알려 주므로 "처리 안 한 케이스"가 숨지 못합니다 (5장).
- `match` 도 식이므로 `let s = match n { ... };` 가 가능합니다.
- 변수에 바인딩하며 범위 매칭: `n @ 1..=9 => ...` (`n` 에 값이 담김).

> ⚠️ **C++ / Python 개발자가 헷갈리는 점**
> C++ `switch` 는 `break` 를 빼먹으면 다음 케이스로 떨어지고(fallthrough), 경우를 빠뜨려도 조용합니다. Python `match` 도 안 맞으면 그냥 지나가요.
> Rust 의 `match` 는 반대로 "빠뜨리면 컴파일이 안 됨"이라서, AI 가 `_ => {}` 로 대충 덮어 둔 곳은 **의도된 무시인지** 리뷰하세요.

### 8. 반복: `loop`, `while`, `for`, 범위, 라벨

```rust
fn main() {
    for i in 0..3 { print!("{i} "); }          // 0 1 2        (끝 제외)
    for i in 0..=3 { print!("{i} "); }         // 0 1 2 3      (끝 포함)
    for i in (0..10).step_by(5) { print!("{i} "); }   // 0 5
    for i in (1..=3).rev() { print!("{i} "); } // 3 2 1
    println!();

    let mut n = 3;
    while n > 0 { n -= 1; }

    // 라벨: 안쪽 루프에서 바깥 루프를 한 번에 탈출
    let mut found = None;
    'outer: for a in 1..10 {
        for b in 1..10 {
            if a * b == 42 {
                found = Some((a, b));
                break 'outer;      // 바깥 for 까지 종료
            }
        }
    }
    println!("{found:?}");         // Some((6, 7))
}
```

- `for` 는 **이터레이터**를 돌립니다 (`0..3` 도 이터레이터). C 스타일 `for (i=0; i<n; i++)` 는 없어요. 인덱스 루프보다 이터레이터가 관용구입니다 (7장, 12장).
- `i++` / `++i` 도 없습니다. `i += 1`.
- C++ 에 `goto` 나 플래그가 필요했던 "중첩 루프 탈출"을 라벨 하나로 해결합니다.

### 9. 나눗셈과 나머지 — Python 과 다른 점

| 식 | C++ | Rust | Python |
|---|---|---|---|
| `7 / 2` | 3 | 3 | `7 // 2` = 3 (`7 / 2` 는 3.5 실수!) |
| `-7 / 2` | -3 | -3 | `-7 // 2` = **-4** |
| `-7 % 3` | -1 | -1 | **2** |
| 유클리드 나머지 | 직접 구현 | `(-7_i32).rem_euclid(3)` = 2 | `-7 % 3` |

Python 에서 Rust 로 옮길 때 가장 자주 틀리는 부분입니다. 이 챕터의 `basics_demo` 가 이 값들을 세 언어로 출력해서 확인해 줘요.

---

## 🧯 자주 만나는 컴파일 에러

실제 rustc 1.97 의 출력입니다 (🔬 실험실에서 직접 재현할 수 있어요).

### 1) 불변 변수에 재대입 — `E0384`

```text
error[E0384]: cannot assign twice to immutable variable `x`
 --> p6.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```
**고치기**: 정말 바뀌어야 하면 `let mut`. 아니면 shadowing(`let x = 6;`)이 더 Rust 다울 수 있어요.

### 2) 서로 다른 숫자 타입 연산 — `E0308` / `E0277`

```text
error[E0308]: mismatched types
 --> p7.rs:4:24
  |
4 |     println!("{}", a + b);
  |                        ^ expected `i32`, found `i64`

error[E0277]: cannot add `i64` to `i32`
```
**고치기**: `i64::from(a) + b` (손실 없는 확대) 또는 `a as i64 + b`. 어느 쪽으로 맞출지 *명시*하게 만드는 것이 Rust 의 의도입니다.

### 3) 인덱스에 `i32` 사용 — `E0277`

```text
error[E0277]: the type `[{integer}]` cannot be indexed by `i32`
 --> q5.rs:4:22
  |
4 |     println!("{}", v[i]);
  |                      ^ slice indices are of type `usize` or ranges of `usize`
```
**고치기**: `v[i as usize]` 보다는 처음부터 `let i: usize = 1;` (또는 `for i in 0..v.len()`).

### 4) `match` 가 모든 경우를 덮지 못함 — `E0004`

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=0_i32` and `3_i32..=i32::MAX` not covered
 --> p10.rs:3:19
  |
3 |     let s = match n {
  |                   ^ patterns `i32::MIN..=0_i32` and `3_i32..=i32::MAX` not covered
  |
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern, ...
```
**고치기**: 빠진 범위의 갈래 추가, 또는 `_ => ...`. 단, `_` 는 "나중에 추가될 경우도 전부 삼켜 버리는" 갈래라는 점에 유의하세요.

### 5) `if` / `else` 타입 불일치 — `E0308`

```text
error[E0308]: `if` and `else` have incompatible types
 --> p8.rs:3:43
  |
3 |     let label = if n > 2 { "big" } else { 0 };
  |                            -----          ^ expected `&str`, found integer
  |                            |
  |                            expected because of this
```

### 6) 컴파일러가 아는 오버플로 vs 런타임 오버플로

```text
error: this arithmetic operation will overflow
 --> p11.rs:3:13
  |
3 |     let y = x + 1;
  |             ^^^^^ attempt to compute `u8::MAX + 1_u8`, which would overflow
  |
  = note: `#[deny(arithmetic_overflow)]` on by default
```
값이 컴파일 때 정해져 있으면 위처럼 에러, 실행 중에야 정해지면 **debug 에서 패닉**:
```text
thread 'main' panicked at p9.rs:3:13:
attempt to add with overflow
```

### 7) 타입을 정할 수 없음 — `E0284`

```text
error[E0284]: type annotations needed
 --> q2.rs:2:9
  |
2 |     let guess = "42".parse().unwrap();
  |         ^^^^^        ----- type must be known at this point
help: consider giving `guess` an explicit type
```
`parse` 는 "무엇으로 읽을지"를 알아야 합니다. `let guess: u32 = ...` 또는 `"42".parse::<u32>()`.

---

## 🛠️ 실습: 콜라츠 통계

### 무엇을 만드나요

**콜라츠 추측**: 자연수 n 에서 시작해 "짝수면 2로 나누고, 홀수면 3배 + 1" 을 반복하면 언젠가 1 에 도달한다는 (아직 증명되지 않은) 추측입니다.
n = 6 이면 `6 → 3 → 10 → 5 → 16 → 8 → 4 → 2 → 1` (8 단계, 최댓값 16).

프로그램은 아래를 출력합니다 (일부):

```text
== collatz 1..=30 ==
  n steps   peak kind
  1     0      1 start
  2     1      2 short
  ...
 27   111   9232 long
longest chain: n=27 steps=111
highest peak: n=27 peak=9232
== overflow: u8 add ==
250 + 5: checked=Some(255) wrapping=255 saturating=255
250 + 10: checked=None wrapping=4 saturating=255
== overflow: collatz ==
u8 starts 1..=255 that overflow: 146 (first: 27 at step 11)
u32 start=159487: overflow at step 59
```

콜라츠 사슬은 중간에 값이 엄청 커지기도 해서(27 은 9232 까지), **정수 타입이 작으면 넘칩니다.** u8 에서는 n = 27 이 11 단계 만에, u32 에서는 159487 이 59 단계 만에 넘쳐요.
이를 `checked_mul` 로 안전하게 탐지하는 것이 실습의 핵심입니다.

### 파일 구성

| 파일 | 역할 |
|---|---|
| `python/main.py` | Python 참조 구현 (정수가 무한 정밀도라 `max_value` 인자로 흉내) |
| `cpp/main.cpp` | C++20 참조 구현 (`std::optional`, `std::format`) |
| `rust/src/main.rs` | ★ 여러분이 채울 파일 (TODO 10개 + 🔬 실험 10개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법

```bash
cd chapters/ch02_syntax_basics
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch02_syntax_basics                      # TODO 가 남아 있으면 패닉(정상)
cargo test -p ch02_syntax_basics
cargo run -q -p ch02_syntax_basics --example solution   # 정답 실행
../../scripts/compare.sh ch02                           # 내 코드 비교
../../scripts/compare.sh ch02 --solution                # 정답 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| TODO(1) | `collatz_next` | `if` 식, 세미콜론 없는 반환 |
| TODO(2) | `collatz_steps` | `mut`, `while`, `+=` |
| TODO(3) | `collatz_peak` | `.max()` |
| TODO(4) | `classify_steps` | `match`, 범위 패턴, 빠짐없는 검사 |
| TODO(5) | `longest_chain` | 튜플 반환, `for` + `..=` |
| TODO(6) | `max_peak` | 튜플 반환 |
| TODO(7) | `checked_next_u8/u32` | `checked_mul`, `Option`, `?` 맛보기 |
| TODO(8) | `overflow_step_u8/u32` | `match` on `Option`, 조기 `return` |
| TODO(9) | `find_first_exceeding` | **labeled break** |
| TODO(10) | `to_u8`, `trunc_to_i32` | `as` 변환 |

### 완료 기준

- `cargo test -p ch02_syntax_basics` 8개 모두 `ok`
- `../../scripts/compare.sh ch02` 에서 `✅ python vs rust : 출력이 똑같아요`

### 🔬 실험 목록 (`rust/src/main.rs` 의 `experiments()`)

| 실험 | 해볼 것 | 기대 결과 |
|---|---|---|
| 1 | 불변 변수에 재대입 | `E0384` |
| 2 | `i32 + i64` | `E0308`, `E0277` |
| 3 | `if` / `else` 타입 다르게 | `E0308` |
| 4 | `match` 에서 `_` 제거 | `E0004` |
| 5 | `let x: u8 = 255; x + 1` | 컴파일 에러 (lint). **`cargo check` 가 아니라 `cargo build`** 로 확인 |
| 6 | 런타임 오버플로: **debug vs `--release` 비교** | debug 패닉 / release 는 `y = 0` |
| 7 | `let x: u32 = -1;` | `E0600` |
| 8 | `let x: u8 = 300;` | 컴파일 에러 (lint) |
| 9 | `while` 에서 `break 값` | `E0571` |
| 10 | 타입 추론 실패 `parse()` | `E0284` |

실험 6 은 이 챕터의 핵심입니다. 꼭 `cargo run -p ch02_syntax_basics` 와 `cargo run --release -p ch02_syntax_basics` 를 **둘 다** 돌려 보세요.

---

## 🤖 바이브 코딩 포인트

- **`as` 캐스트를 전수 검사하세요.** AI 는 `len() as i32`, `x as u8`, `idx as usize` 를 습관처럼 씁니다. 값이 음수·초과일 수 있다면 `try_from` / `checked_*` 로 바꿔 달라고 요청하세요.
- **`+`, `-`, `*` 가 금액·길이·인덱스에 쓰였는지** 보세요. debug 에선 패닉, release 에선 조용히 틀린 값이 나옵니다. 필요한 곳엔 `checked_*` / `saturating_*` / `wrapping_*` 중 *의도*가 드러나는 것을 쓰게 하세요.
- **`_ => {}` / `_ => unreachable!()` 를 의심하세요.** 빠짐없는 `match` 가 Rust 의 안전장치인데, 와일드카드가 그것을 꺼 버립니다. enum 을 매칭할 때는 가능하면 모든 변형을 나열하게 요청하세요.
- **`let mut` 가 많으면 신호입니다.** 루프 안에서 변수를 계속 바꾸는 C++/Python 스타일일 수 있어요. "가변 변수를 줄이고 표현식/이터레이터 중심으로 다시 써 줘"가 좋은 프롬프트입니다 (12장).
- **불필요한 shadowing 과 `return`** : `return x;` 를 함수 끝에 쓰거나 같은 이름을 남발하면 읽기 어려워집니다. "마지막 식으로 반환하고, 의미가 다른 값은 다른 이름을 써 줘"라고 요청해 보세요.
- 프롬프트 팁: *"정수 오버플로가 가능한 곳을 표시하고, 필요한 곳은 checked 연산으로 바꿔 줘. 변환은 `as` 대신 `From`/`TryFrom` 을 우선해 줘."*

---

## ✅ 셀프 체크

**Q1.** `let x = 5; let x = x * 2;` 와 `let mut x = 5; x = x * 2;` 의 차이는? 언제 shadowing 이 더 낫나요?

<details><summary>답</summary>

앞의 것은 *새 변수*를 만들어 옛 `x` 를 가린 것이고(타입도 바꿀 수 있음), 뒤의 것은 같은 변수의 값을 바꾼 것입니다(타입 고정, `mut` 필요).
이후에 변하지 않을 가공된 값이면 shadowing 이 좋아요 (`let line = line.trim();`). 루프에서 누적하는 값이면 `mut`.
</details>

**Q2.** 다음 코드는 debug 와 release 에서 각각 어떻게 동작하나요?

```rust
let x: u8 = std::env::args().count() as u8 + 254;
let y = x + 1;
```

<details><summary>답</summary>

debug: `attempt to add with overflow` 패닉. release: 패닉 없이 `y = 0` (255 + 1 이 한 바퀴 돌아 0).
의도한 감김이면 `x.wrapping_add(1)`, 넘치면 안 되면 `x.checked_add(1)`.
</details>

**Q3.** 아래 두 함수 중 컴파일되는 것은? 왜 그럴까요?

```rust
fn a(x: i32) -> i32 { x + 1 }
fn b(x: i32) -> i32 { x + 1; }
```

<details><summary>답</summary>

`a` 만 컴파일됩니다. `b` 는 `;` 때문에 본문의 값이 `()` 가 되어 `E0308: expected i32, found ()`.
</details>

**Q4.** `-7 / 2`, `-7 % 3` 의 값은 Rust / C++ / Python 에서 각각? 결과를 Python 식으로 맞추려면 Rust 에서 무엇을 쓰나요?

<details><summary>답</summary>

Rust / C++ : `-3`, `-1`. Python : `-7 // 2 = -4`, `-7 % 3 = 2`. Python 의 나머지와 같은 값은 `(-7_i32).rem_euclid(3)` (= 2), 내림 나눗셈은 `(-7_i32).div_euclid(2)` 가 가장 가깝습니다 (= -4).
</details>

**Q5.** `match` 가 `if / else if` 사슬보다 안전한 이유는? `_` 를 쓸 때 주의할 점은?

<details><summary>답</summary>

모든 경우를 다뤘는지 컴파일러가 검사해서 빠뜨린 경우가 컴파일 에러로 드러납니다. `_` 는 지금 없는 미래의 경우까지 전부 삼키므로, 나중에 값이 추가되어도 컴파일러가 알려 주지 않아요. 중요한 enum 은 변형을 모두 나열하는 편이 안전합니다.
</details>

---

## 📚 더 알아보기

- The Rust Book 3장 *Common Programming Concepts*: <https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html>
  (변수 3.1, 데이터 타입 3.2 — 정수 오버플로 설명 포함, 제어 흐름 3.5)
- The Rust Book 6.2 *The `match` Control Flow Construct*: <https://doc.rust-lang.org/book/ch06-02-match.html>
- Rust by Example — Primitives: <https://doc.rust-lang.org/rust-by-example/primitives.html>
- Rust by Example — Casting: <https://doc.rust-lang.org/rust-by-example/types/cast.html>
- std 문서: [`u8::checked_add`](https://doc.rust-lang.org/std/primitive.u8.html#method.checked_add), [`i32::rem_euclid`](https://doc.rust-lang.org/std/primitive.i32.html#method.rem_euclid)
- The Rust Reference — Integer overflow: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow>
