# Ch09. 라이프타임

> "이 참조는 언제까지 안전한가?"를 컴파일러에게 알려 주는 이름표 — C++ 의 `string_view` 댕글링 버그를 컴파일 때 잡는 장치  ·  예상 소요: 이론 50분 / 실습 60분  ·  선행: Ch04, Ch05, Ch08

## 🎯 학습 목표

- 라이프타임이 **왜** 필요한지(댕글링 참조 방지)를 C++ `std::string_view` 경험에 빗대어 설명할 수 있습니다.
- `fn longer<'a>(a: &'a str, b: &'a str) -> &'a str` 의 `'a` 를 읽고 쓸 수 있습니다.
- **생략 규칙(elision)** 3가지를 알고, 언제 직접 적어야 하는지 구분합니다.
- 참조를 필드로 가진 구조체 `LogEntry<'a>` 를 설계하고 `impl` 할 수 있습니다.
- `'static` 이 무엇이고 무엇이 아닌지 알며, "라이프타임은 **설명**이지 **제어**가 아니다"를 이해합니다.
- 흔한 에러 `E0106`, `E0597`, `E0515` 를 보고 원인을 짚을 수 있습니다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 복사 없는 문자열 조각 | `std::string_view` | (없음, 슬라이스는 복사) | `&str` |
| 원본보다 오래 사는 참조 | **허용됨 → UB (댕글링)** | GC 가 살려 둠 (문제 없음) | **컴파일 에러** |
| "누구 데이터를 가리키는가" 표기 | 주석/관례 (`// s 가 살아있어야 함`) | 필요 없음 | `'a` 라이프타임 |
| 지역 변수 참조 반환 | 경고 가능, 컴파일 됨 | 항상 안전 | `E0515` |
| 자료구조에 참조 저장 | `string_view` 멤버 (조심!) | 객체 참조 | `struct S<'a> { x: &'a str }` |
| 수명을 늘려 주는 도구 | `shared_ptr` / 복사 | 참조 카운팅 | 소유 타입(`String`, `Rc`) 으로 바꾸기 |
| "프로그램 끝까지 산다" | 정적/전역 변수 | 모듈 전역 | `'static` |

## 📘 이론

> 이 챕터의 코드 조각은 std 만 쓰므로 <https://play.rust-lang.org> 에서 그대로 실행해 볼 수 있습니다. (`fn main` 으로 감싸 주세요.)

### 1. 왜 필요할까요? — C++ 에서 이미 겪은 그 버그

C++ 에서 이런 코드를 써 보신 적이 있을 거예요.

```cpp
std::string_view first_word() {
    std::string s = "hello world";
    return std::string_view(s).substr(0, 5);   // 컴파일 OK, 경고도 (보통) 없음
}   // s 는 여기서 파괴 → 반환된 string_view 는 해제된 메모리를 가리킴 (UB)
```

`string_view` 는 `(포인터, 길이)` 두 칸짜리 "빌린 창문"입니다. **창문이 가리키는 집이 철거돼도** 창문 객체는 멀쩡해 보이죠.
이 버그는 테스트에서 안 걸리고 운영에서 가끔 터지는 대표적인 UB 입니다.

Rust 의 `&str` 도 똑같은 "(포인터, 길이)" 입니다. 하지만 **컴파일러가 모든 참조에 대해 "이 참조는 어떤 데이터를 얼마 동안 빌린 것인가"를 추적**합니다. 그 추적 단위가 라이프타임입니다.

```rust
fn first_word_upper(text: &str) -> &str {
    let word = text.split_whitespace().next().unwrap_or("");
    let upper = word.to_uppercase();   // 이 함수가 소유하는 새 String
    &upper                             // ❌ E0515: 곧 사라질 지역 변수를 빌려 줌
}
```

> 💡 **쉬운 설명**: 도서관 책(원본)과 대출증(참조)을 생각하세요. 대출증을 가진 사람이 있는 동안에는 도서관이 책을 폐기하면 안 됩니다.
> 라이프타임은 "이 대출증은 저 책에서 나왔다"는 **관계 표시**예요. 컴파일러는 대출증이 책보다 오래 남지 않는지 검사합니다.

### 2. 라이프타임 표기 `'a`

```rust
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    if b.len() > a.len() { b } else { a }
}
```

읽는 법: "어떤 라이프타임 `'a` 가 있다. `a` 와 `b` 는 적어도 `'a` 동안 유효하고, 반환값도 `'a` 동안 유효하다."

- 반환값이 `a` 인지 `b` 인지는 **실행해 봐야** 알기 때문에, 컴파일러는 "둘 중 **더 짧게 사는 쪽**만큼만 쓸 수 있다"고 보수적으로 가정합니다.
- 라이프타임 이름(`'a`, `'b`, `'text`)은 그냥 이름입니다. 의미는 **어느 참조들이 서로 묶여 있는가**에 있어요.

호출하는 쪽에서 보면 이렇습니다.

```rust
let first = String::from("long long long");
let result;
{
    let second = String::from("short");
    result = longer(&first, &second);   // result 는 first 와 second 중 짧은 쪽에 묶임
}                                       // second 가 죽음 → result 도 더 못 씀
// println!("{result}");                // ❌ E0597: `second` does not live long enough
```

⚠️ **C++ 개발자가 헷갈리는 점 ① — 라이프타임은 수명을 *제어*하지 않습니다.**
`'a` 를 적는다고 값이 오래 살아 주지 않아요. 컴파일러에게 **이미 정해진 수명 관계를 "설명"**하는 것뿐입니다.
수명이 모자라면 `'a` 를 바꾸는 게 아니라 **데이터의 소유 구조**(누가 `String` 을 들고 있나)를 바꿔야 합니다.
런타임에는 라이프타임이 흔적도 없이 사라지므로 **비용 0** 입니다 (`string_view` 와 똑같은 속도).

### 3. 생략 규칙 (Lifetime Elision)

대부분의 함수에서는 `'a` 를 안 적어도 됩니다. 컴파일러가 3가지 규칙으로 채워 줘요.

1. 입력 참조 **각각**에 서로 다른 라이프타임이 붙는다.
2. 입력 참조가 **딱 하나**면, 출력 참조는 그것과 같은 라이프타임이다.
3. 메서드에서 `&self` / `&mut self` 가 있으면, 출력 참조는 `self` 의 라이프타임이다.

```rust
fn parse_line(line: &str) -> Option<LogEntry<'_>>      // 규칙 2: 입력이 하나 → 출력도 같은 것
fn parse_line<'a>(line: &'a str) -> Option<LogEntry<'a>>   // 위와 똑같은 뜻 (풀어 쓴 것)

fn longer(a: &str, b: &str) -> &str                    // ❌ 규칙으로 못 채움 (입력이 둘!) → E0106
```

> 💡 규칙으로 채울 수 없을 때만 컴파일러가 `E0106: missing lifetime specifier` 로 "직접 적어 주세요"라고 요청합니다.
> 이건 에러 중에서도 **친절한 편**이에요. 제안(`help:`)을 그대로 따르면 대개 해결됩니다.

`'_` 는 "이름은 필요 없지만 라이프타임이 있다"는 표시입니다: `LogEntry<'_>`, `impl Display for LogEntry<'_>`.

### 4. 참조를 가진 구조체

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct LogEntry<'a> {
    level: &'a str,
    module: &'a str,
    message: &'a str,
}

impl<'a> LogEntry<'a> {
    fn message(&self) -> &'a str { self.message }   // 반환은 self 가 아니라 'a (원본 텍스트)에 묶임
}
```

- 구조체가 참조를 필드로 가지면 **반드시** `<'a>` 를 구조체 이름 옆에 적어야 합니다. 안 적으면 `E0106`.
- 의미: "`LogEntry<'a>` 값은 `'a` 보다 오래 살 수 없다". 이 실습에서는 `'a` = **로그 파일 전체 텍스트(`String`)** 의 수명이에요.
- 그 결과 `Vec<LogEntry<'a>>` 안의 수천 개 `&str` 은 **한 번도 복사되지 않고** 모두 원본 텍스트의 부분을 가리킵니다. (이것이 "제로카피 파서")

```text
text: String ─────────────────────────────────────────────────────────────┐
 "2024-05-01 10:00:01 INFO auth: user alice logged in\n2024-05-01 ..."     │ 힙 (한 덩어리)
                               ▲    ▲     ▲                                │
 LogEntry<'a> { level ─────────┘    │     │ message ──────────────────────┘
                 module ────────────┘     │   (ptr, len) 만 저장 — 복사 없음
```

실습 함수 `longest_message` 는 라이프타임이 **어디에 묶이는가**를 보여 줍니다.

```rust
fn longest_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str>
//                      ^ 슬라이스 자체의 수명(생략됨)   ^ 결과는 "원본 텍스트"에 묶임
```

결과가 `entries` 슬라이스가 아니라 **원본 텍스트('a)** 에 묶여 있기 때문에, `entries` 라는 `Vec` 이 사라져도 결과 `&str` 은 계속 쓸 수 있습니다.

⚠️ **C++ 개발자가 헷갈리는 점 ② — "Python 처럼 그냥 `String` 을 쓰면 안 되나요?"**
됩니다! 그리고 **초보 단계에서는 그게 정답인 경우가 많아요.** 라이프타임이 복잡해지면 필드를 `String` 으로 바꿔 소유하게 하는 것이 가장 흔한 탈출구입니다.
참조 + 라이프타임은 "복사 비용이 실제로 중요한" 파서·핫패스에서 쓰는 최적화 도구예요. (AI 가 `'a` 를 잔뜩 붙인 코드를 줬다면 "정말 필요한가?"를 물어보세요 → 🤖 참고)

### 5. `'static`

`'static` 은 "프로그램이 끝날 때까지 유효"라는 라이프타임입니다.

```rust
let s: &'static str = "hello";   // 문자열 리터럴은 바이너리에 박혀 있어서 영원히 유효
```

- **문자열 리터럴**, `const`/`static` 항목이 `'static` 입니다. (⚙️ C++ 의 `.rodata` 의 `const char*` 와 같아요.)
- ⚠️ 에러가 났을 때 `&'static str` 로 고치려는 유혹은 **대부분 틀린 해법**입니다. 지역 변수를 `'static` 이라고 우긴다고 영원히 살지 않아요. 컴파일러가 거부합니다(🔬 실험 4).
- `T: 'static` 이라는 **바운드**는 "이 타입은 짧게 사는 참조를 품고 있지 않다"는 뜻입니다. `tokio::spawn` 등에서 자주 보게 됩니다 (Ch13–14). 값이 영원히 산다는 뜻이 아니라, 참조를 안 빌려도 된다는 뜻이에요.

### 6. 라이프타임이 에러를 낼 때 — 사고 순서

1. **"누가 소유하나?"** 데이터를 실제로 소유한 변수(`String`, `Vec`)가 어디인지 찾습니다.
2. **"참조가 그것보다 오래 쓰이나?"** 에러 화살표가 "빌린 곳"과 "나중에 또 쓰는 곳"을 가리킵니다.
3. 해법 순서: ① 소유자를 더 바깥 범위로 옮긴다 → ② 참조 대신 소유 타입(`String`, `.to_string()`, `.clone()`)을 쓴다 → ③ 정말 필요하면 `'a` 를 적는다.
   `'static` 이나 `Box::leak` 같은 힘기술은 마지막의 마지막 수단입니다.

## 🧯 자주 만나는 컴파일 에러

(아래는 모두 이 환경의 `rustc` 로 직접 얻은 실제 출력입니다.)

### 1) E0106 — 라이프타임 지정자가 없어요 (함수)

```text
error[E0106]: missing lifetime specifier
 --> l1.rs:1:32
  |
1 | fn longer(a: &str, b: &str) -> &str {
  |              ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
  |          ++++     ++          ++          ++
```

- **의미**: 입력 참조가 둘이라 반환값이 누구에게 묶이는지 컴파일러가 몰라요.
- **고치기**: `help:` 가 알려 준 대로 `<'a>` 를 붙입니다. (반환이 항상 `a` 에서만 나온다면 `a: &'a str, b: &str` 로 더 정확하게 적을 수도 있어요.)

### 2) E0106 — 구조체 필드에 참조를 쓰면서 `'a` 를 안 적었어요

```text
error[E0106]: missing lifetime specifier
 --> l2.rs:2:12
  |
2 |     level: &str,
  |            ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ struct LogEntry<'a> {
2 ~     level: &'a str,
  |
```

- **고치기**: 구조체에 `<'a>` 를 달고 모든 참조 필드에 `&'a` 를 적습니다. 또는 필드를 `String` 으로 바꿔 소유합니다.

### 3) E0597 — 빌린 값이 충분히 오래 살지 못해요

```text
error[E0597]: `second` does not live long enough
  --> l3.rs:9:33
   |
 8 |         let second = String::from("short");
   |             ------ binding `second` declared here
 9 |         result = longer(&first, &second);
   |                                 ^^^^^^^ borrowed value does not live long enough
10 |     }
   |     - `second` dropped here while still borrowed
11 |     println!("{result}");
   |                ------ borrow later used here
```

- **의미**: C++ 의 "string_view 를 들고 있는데 원본이 먼저 파괴됨"을 컴파일러가 잡은 것. 화살표 3개(선언 / 파괴 / 나중 사용)를 순서대로 읽으세요.
- **고치기**: `second` 를 `result` 를 쓰는 곳까지 살게 바깥으로 옮기거나, `result` 를 안쪽 블록에서만 쓰거나, `longer(..).to_string()` 으로 소유 문자열로 만듭니다.
- 구조체 버전(`LogEntry { message: &text[5..] }`)도 똑같이 `E0597` 이 납니다.

### 4) E0515 — 지역 변수의 참조를 반환하려 했어요

```text
error[E0515]: cannot return reference to local variable `upper`
 --> l4.rs:4:5
  |
4 |     &upper
  |     ^^^^^^ returns a reference to data owned by the current function
```

- **의미**: 함수가 끝나면 `upper` 가 해제되는데 그 참조를 밖으로 내보내려 함 (C++ 의 `return s.c_str();` 같은 버그).
- **고치기**: 반환 타입을 `String` 으로 바꿔서 소유권을 넘겨 줍니다. "새로 만든 값은 `String` 으로, 기존 데이터의 일부는 `&str` 로" 가 기본 원칙이에요.

### 5) `lifetime may not live long enough` — `'static` 으로 우겼어요

```text
error: lifetime may not live long enough
 --> l7.rs:2:28
  |
1 | fn longer(a: &str, b: &str) -> &'static str {
  |                       - let's call the lifetime of this reference `'2`
2 |     if b.len() > a.len() { b } else { a }
  |                            ^ returning this value requires that `'2` must outlive `'static`
```

- **의미**: 입력 `b` 는 `'static` 만큼 오래 산다는 보장이 없어요. 라이프타임 표기는 **약속을 적는 것**이지 수명을 늘리는 마법이 아닙니다.

## 🛠️ 실습: 제로카피 로그 파서

### 무엇을 만드나요

`data/app.log` 의 각 줄 `날짜 시간 LEVEL module: message` 를 `LogEntry<'a>` 로 파싱합니다. 문자열은 **한 글자도 복사하지 않습니다.**
형식이 틀린 줄은 건너뛰고 몇 줄인지 셉니다. 그리고 레벨별 필터, 가장 긴 메시지, 모듈별 개수를 출력합니다.

예상 출력(일부):

```text
parsed 11 entries, skipped 3 malformed lines
== ERROR entries ==
[ERROR] db: query timeout after 30s while loading user profile
[ERROR] api: upstream service unavailable
[ERROR] auth: token signature mismatch
...
== per module ==
api: 3
auth: 4
cache: 2
db: 2
== longer ==
longer("abc", "de") = abc
```

### 파일 구성

| 언어 | 파일 | 포인트 |
|---|---|---|
| 데이터 | `data/app.log` | 유효한 줄 11개 + 잘못된 줄 3개 + 빈 줄 1개 |
| Python | `python/main.py` | `str.split`, `partition` (슬라이스마다 복사) |
| C++ | `cpp/main.cpp` | `std::string_view` + 주석 처리된 **댕글링 버그 예시** |
| Rust | `rust/src/main.rs` (실습), `rust/examples/solution.rs` (정답) | `LogEntry<'a>` |

### 실행 방법

```bash
cd chapters/ch09_lifetimes
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch09_lifetimes                       # 내 풀이
cargo test -p ch09_lifetimes                         # 내 풀이 테스트
cargo run -q -p ch09_lifetimes --example solution    # 정답
../../scripts/compare.sh ch09                        # 내 풀이 비교
../../scripts/compare.sh ch09 --solution             # 정답 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `impl Display for LogEntry<'_>` | `'_`, 구조체 라이프타임과 impl |
| 2 | `parse_line` | 생략 규칙(규칙 2), `splitn`, `split_once`, `?` on Option |
| 3 | `parse_log<'a>` | 명시적 `'a`, 반환 Vec 안의 참조 |
| 4 | `filter_by_level<'a>` | 입력 참조가 여럿일 때 어느 것에 묶이나, `Copy` 구조체 |
| 5 | `longest_message<'a>` | 결과가 슬라이스가 아니라 **원본**에 묶임 |
| 6 | `count_by_module<'a>` | `BTreeMap<&'a str, _>`, entry API |
| 7 | `longer<'a>` | 고전적인 두 입력 → 한 출력 |

### 완료 기준

- `cargo test -p ch09_lifetimes` 전부 통과 (zero-copy 를 포인터 범위로 검증하는 테스트 포함)
- `../../scripts/compare.sh ch09` 에서 세 언어 출력이 동일

### 🔬 실험 목록

| 번호 | 에러 | 해 볼 것 |
|---|---|---|
| 1 | `E0106` | `longer` 에서 `<'a>` 를 빼기 / 구조체에서 `<'a>` 를 빼기 |
| 2 | `E0597` | `text` 를 안쪽 블록에서 만들고 `entries` 를 블록 밖에서 쓰기 |
| 3 | `E0515` | 지역 `String` 의 참조를 반환 (C++ 의 `return s.c_str()` 와 비교) |
| 4 | `lifetime may not live long enough` | `-> &'static str` 로 우겨 보기 |

각 실험은 `rust/src/main.rs` 안에 주석 처리된 코드와 질문으로 들어 있습니다. 풀어서 에러를 읽고, 다시 주석으로 되돌려 놓으세요.

## 🤖 바이브 코딩 포인트

- **AI 가 `'a` 를 잔뜩 붙였다면** → "이 함수 입출력을 `String`/소유 타입으로 바꾸면 라이프타임 없이 되나요?" 라고 물어보세요. 성능이 핵심이 아니면 소유 타입이 더 읽기 쉽습니다.
- **`.clone()` / `.to_string()` 이 파서 안쪽에 많다면** → 이 챕터처럼 `&str` 슬라이스로 바꿀 수 있는지 검토하세요 (복사 줄이기). 단, 정확성이 먼저입니다.
- **`&'static str` 이 에러 해결용으로 등장하면** → 거의 틀린 방향입니다. 소유자를 올바른 범위로 옮기는 해법을 요청하세요.
- **`Box::leak`, `unsafe`, `transmute` 로 수명을 속이는 코드** → 즉시 의심. "안전한 방법으로 다시 짜 줘"라고 요청하세요.
- **구조체에 `&'a T` 필드가 있는 API** → 그 구조체는 원본보다 오래 못 삽니다. 스레드/async 에 넘기거나 `Vec` 에 오래 보관한다면 소유 타입이 필요한지 확인하세요.
- 프롬프트 팁: "라이프타임 에러(E0597)가 나는데, `'a` 를 더 붙이지 말고 소유권 구조를 바꾸는 방향으로 고쳐 줘. 왜 그런 에러가 났는지도 한 문단으로 설명해 줘."

## ✅ 셀프 체크

**Q1.** C++ 의 `string_view` 댕글링 버그와 Rust 의 `E0597` 은 어떤 관계인가요?

<details><summary>답</summary>

같은 종류의 버그(원본이 먼저 사라진 뒤 빌린 조각을 사용)입니다. C++ 는 컴파일되고 UB 가 되지만, Rust 는 라이프타임 검사로 컴파일 시점에 막습니다. 런타임 비용은 둘 다 0 이에요.
</details>

**Q2.** `fn first(a: &str, b: &str) -> &str` 은 왜 안 되고, `fn first(a: &str) -> &str` 은 왜 되나요?

<details><summary>답</summary>

생략 규칙 2: 입력 참조가 **하나**면 출력이 그것과 같다고 추론합니다. 입력이 둘이면 어느 쪽에 묶일지 모르므로 `E0106` — 직접 `<'a>` 를 적어 알려 줘야 합니다.
</details>

**Q3.** `longest_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str>` 의 결과는 `entries` 벡터가 사라진 뒤에도 쓸 수 있습니다. 왜일까요?

<details><summary>답</summary>

반환 `&'a str` 의 `'a` 는 `entries` 슬라이스(생략된 별도 라이프타임)가 아니라 원본 텍스트에 묶여 있기 때문입니다. `LogEntry<'a>` 안의 `&'a str` 을 그대로 복사해(참조는 `Copy`) 돌려주는 것이라 `Vec` 의 수명과 무관해요.
</details>

**Q4.** `'static` 은 "영원히 사는 값"인가요?

<details><summary>답</summary>

참조로서의 `&'static T` 는 "프로그램 끝까지 유효한 데이터를 가리킨다"는 뜻(예: 문자열 리터럴)이지만, 라이프타임을 적는다고 값이 오래 사는 것은 아닙니다. `T: 'static` 바운드는 "짧은 참조를 품고 있지 않다"는 뜻이라서, `String` 같은 소유 타입도 만족해요.
</details>

**Q5.** 라이프타임 에러가 계속 나서 막힐 때 가장 먼저 시도할 일은?

<details><summary>답</summary>

참조 대신 소유 타입(`String`, `.to_string()`, `.clone()`)으로 바꿔 보기입니다. 구조 자체를 단순화하면 대부분 해결되고, 그다음에 성능 때문에 필요할 때만 참조 + 라이프타임으로 최적화합니다.
</details>

## 📚 더 알아보기

- The Rust Book Ch10.3 (라이프타임으로 참조 유효성 검증): <https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html>
- Rust by Example — Lifetimes: <https://doc.rust-lang.org/rust-by-example/scope/lifetime.html>
- Rust Reference — Lifetime elision: <https://doc.rust-lang.org/reference/lifetime-elision.html>
- 에러 코드 해설: `rustc --explain E0106`, `E0597`, `E0515`
- (심화) Nomicon — Lifetimes: <https://doc.rust-lang.org/nomicon/lifetimes.html>
