# Ch06. 에러 처리

> 예외(exception) 대신 "에러도 값"으로 돌려주는 `Result`, 그리고 `?` 한 글자로 전파하기.  ·  예상 소요: 이론 40분 / 실습 70분  ·  선행: Ch05 (enum·match·Option)

## 🎯 학습 목표
- **panic**(복구 불가능한 버그)과 **`Result`**(예상 가능한 실패)를 구분해서 쓴다.
- `Option` / `Result` 의 콤비네이터(`map`, `and_then`, `ok_or`, `map_err`, `unwrap_or` ...)를 읽고 쓴다.
- `?` 연산자가 하는 일(조기 반환 + `From` 변환)을 설명한다.
- 커스텀 에러 enum 을 만들고 `Display` / `std::error::Error` / `From` 을 구현한다.
- C++ 예외, Python 예외와 비교해 "에러가 시그니처에 드러나는 것"의 장단점을 말한다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 실패 알리기 | `throw std::runtime_error(..)` | `raise ValueError(..)` | `return Err(MyError::..)` |
| 시그니처에 드러남? | 아니오 (`noexcept` 뿐) | 아니오 | **예**: `-> Result<T, E>` |
| 처리 | `try { } catch (const E&) { }` | `try: ... except E:` | `match r { Ok(v) => .., Err(e) => .. }` |
| 전파 | 자동 (스택을 풀며 올라감) | 자동 | **명시적** `?` |
| 무시하기 | catch 를 안 하면 위로 전파 / `[[nodiscard]]` 로 경고 | 안 잡으면 프로그램 종료 | `Result` 를 안 쓰면 컴파일러 **경고**(`#[must_use]`) |
| 값 없음 | `std::optional` | `None` | `Option<T>` |
| 에러 값 타입 | (C++23) `std::expected<T, E>` | 없음 | `Result<T, E>` (처음부터 표준) |
| 복구 불가능한 버그 | `assert`, `std::terminate` | `AssertionError` | `panic!`, `unwrap()` 실패 |
| 원인 체인 | `std::throw_with_nested` | `raise A from B` | `Error::source()` |

## 📘 이론

> 코드 조각은 <https://play.rust-lang.org> 에서 그대로 실행할 수 있습니다 (표준 라이브러리만 사용).

### 1. panic 과 Result — 언제 무엇을?

```rust
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", v[10]); // ❌ panic: index out of bounds — 프로그램의 "버그"
}
```

`panic!` 은 스레드를 풀어 내려가며(unwind) 종료시킵니다. **"일어나면 안 되는 일(버그)"** 에 씁니다: 인덱스 범위 초과, `unwrap()` 실패, 정수 오버플로(debug) 등.

반대로 **"일어날 수 있는 일"** — 파일이 없다, 사용자 입력이 틀렸다, 네트워크가 끊겼다 — 은 `Result` 로 돌려줍니다.

```rust
enum Result<T, E> {   // 표준 라이브러리 정의 (Option 과 똑같은 모양의 enum)
    Ok(T),
    Err(E),
}
```

```rust
use std::num::ParseIntError;

fn parse_age(s: &str) -> Result<u8, ParseIntError> {
    s.trim().parse::<u8>()
}

fn main() {
    match parse_age("42") {
        Ok(age) => println!("age = {age}"),
        Err(e) => println!("bad age: {e}"),   // (예: "x" 를 넣으면 "bad age: invalid digit found in string")
    }
    println!("{:?}", parse_age("300"));       // Err(ParseIntError { kind: PosOverflow })
}
```

| 상황 | 선택 |
|---|---|
| 호출한 쪽이 복구를 시도할 만한 실패 | `Result` |
| "값이 없음"이 정상적인 결과 (예: 검색 결과 없음) | `Option` |
| 절대 일어나면 안 되는 내부 불변식 위반 | `panic!` / `unwrap` / `expect("이유")` |
| 테스트, 프로토타입, `main` 첫 줄의 설정 | `unwrap` / `expect` 도 OK |

💡 **쉬운 설명** — C++/Python 의 예외는 "어딘가에서 던지면 어딘가에서 받는" 눈에 안 보이는 통로입니다. Rust 는 실패를 **반환값에 꼭 적어야** 하고, 받은 쪽은 `Ok` 와 `Err` 를 모두 다뤄야 값을 쓸 수 있습니다. 택배 상자에 "파손 가능"이라고 라벨이 붙어서 도착하는 것과 같아요.

⚠️ **C++ 개발자가 헷갈리는 점**
- Rust 의 `panic` 은 예외처럼 `catch` 해서 흐름 제어에 쓰는 것이 아닙니다 (스레드 경계에서만 잡을 수 있음). 비즈니스 오류에 panic 을 쓰지 마세요.
- 정수 나눗셈 0, 배열 범위 초과도 panic 입니다. 외부 입력이 들어오는 곳에서는 `get(i)`(→ `Option`)나 `checked_div`(→ `Option`)를 쓰세요.

### 2. `?` — 에러 전파 한 글자

```rust
use std::fs;
use std::io;

fn read_len(path: &str) -> Result<usize, io::Error> {
    let text = fs::read_to_string(path)?;   // Err 이면 여기서 즉시 return Err(e)
    Ok(text.len())
}
```

`expr?` 는 대략 이렇게 풀립니다.

```rust
let text = match fs::read_to_string(path) {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),   // ← 반환 에러 타입으로 From 변환!
};
```

- `?` 는 **`Result` 나 `Option` 을 반환하는 함수 안에서만** 쓸 수 있습니다. `main` 도 `fn main() -> Result<(), Box<dyn Error>>` 로 만들면 쓸 수 있어요.
- `Option` 을 반환하는 함수에서 `opt?` 는 `None` 이면 `None` 을 반환합니다.

### 3. 콤비네이터 — `if` 중첩을 줄이는 도구들

```rust
let port: u16 = std::env::var("PORT")        // Result<String, VarError>
    .ok()                                    // Option<String>        (에러 정보를 버림)
    .and_then(|s| s.parse().ok())            // Option<u16>           (파싱 실패도 None)
    .unwrap_or(8080);                        // u16                   (없으면 기본값)
```

| 메서드 | 하는 일 | 비유 |
|---|---|---|
| `map(f)` | `Ok`/`Some` 안의 값만 변환 | Python `x and f(x)` |
| `map_err(f)` | `Err` 안의 에러만 변환 | 예외 타입 바꿔 던지기 |
| `and_then(f)` | `f` 도 `Result`/`Option` 을 돌려줄 때 (flatMap) | 연쇄 호출 |
| `ok_or(e)` / `ok_or_else(\|\| e)` | `Option` → `Result` | `if x is None: raise` |
| `ok()` | `Result` → `Option` (에러 버림) | |
| `unwrap_or(v)` / `unwrap_or_else(f)` / `unwrap_or_default()` | 실패 시 기본값 | `x or default` |
| `unwrap()` / `expect("msg")` | 실패 시 panic | `assert` |
| `is_ok()` / `is_err()` / `is_some()` / `is_none()` | 검사 | |

> `ok_or(..)` 의 인자는 *항상* 계산됩니다. 만드는 데 비용이 든다면 `ok_or_else(|| ..)` (클로저, 필요할 때만 실행)를 쓰세요.

### 4. 커스텀 에러 타입 — enum + Display + Error + From

라이브러리/앱에서 실패 종류를 한 enum 으로 모으는 것이 Rust 의 정석입니다.

```rust
use std::fmt;
use std::io;

#[derive(Debug)]
enum ConfigError {
    MissingKey(String),
    BadNumber { line: usize },
    Io(io::Error),
}

// 사용자에게 보여줄 문장: Display
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingKey(k) => write!(f, "missing key '{k}'"),
            ConfigError::BadNumber { line } => write!(f, "line {line}: not a number"),
            ConfigError::Io(_) => write!(f, "cannot read config file"),
        }
    }
}

// "이것은 에러 타입이다" 선언 (Debug + Display 가 있어야 함). source() 로 원인 체인을 제공할 수도 있음
impl std::error::Error for ConfigError {}

// ? 가 io::Error 를 ConfigError 로 자동 변환하게 해 주는 규칙
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

fn load(path: &str) -> Result<String, ConfigError> {
    let text = std::fs::read_to_string(path)?;   // io::Error -> ConfigError (From)
    Ok(text)
}
```

- `Debug` 는 개발자용 (`{:?}`), `Display` 는 사용자용 (`{}`) 입니다.
- 보일러플레이트가 길어 보이지만 **ch16 에서 `thiserror` 크레이트가 이 부분을 매크로 한 줄로 줄여 줍니다.** 일단 손으로 써 봐야 AI 가 생성한 코드를 읽을 수 있어요.
- 앱의 최상위(`main`)에서는 `Box<dyn std::error::Error>` 나 `anyhow::Error` 로 "아무 에러나" 받는 것이 흔합니다 (ch16).

💡 **쉬운 설명** — 에러 enum 은 "이 모듈이 실패할 수 있는 모든 이유 목록"입니다. 호출하는 쪽은 `match` 로 이유별로 다르게 대응할 수 있고, 새 이유가 추가되면 컴파일러가 대응 안 한 곳을 알려 줍니다.

⚠️ **C++ 개발자가 헷갈리는 점**
- C++ 은 `catch (const std::exception&)` 하나로 거의 모든 것을 잡지만, Rust 는 에러 타입이 함수마다 다릅니다. 그래서 `From` 변환과 `Box<dyn Error>`/`anyhow` 가 필요해집니다.
- `std::expected` (C++23) 와 가장 비슷합니다. 하지만 Rust 는 `?` 와 `From` 으로 전파가 한 글자라는 점이 큰 차이입니다.
- 소멸자(RAII)는 에러 경로(`?` 로 일찍 return)에서도 똑같이 실행됩니다. 자원 누수 걱정이 없습니다 (Ch03).

### 5. `main` 에서 Result 를 돌려주기

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let n: i32 = "42".parse()?;     // ParseIntError 가 Box<dyn Error> 로 자동 변환
    println!("{n}");
    Err("something went wrong".into())   // 문자열도 Box<dyn Error> 로 변환됨
}
// 실패하면 프로세스 종료 코드 1 과 함께 stderr 에 `Error: "something went wrong"` 출력
```

## 🧯 자주 만나는 컴파일 에러

**1. `?` 를 `Result` 를 반환하지 않는 함수에서 사용**
```
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
 --> src/main.rs:3:34
  |
2 | fn main() {
  | --------- this function should return `Result` or `Option` to accept `?`
```
→ 의미: `?` 는 "에러면 return" 이라서 반환 타입이 에러를 담을 수 있어야 합니다. 고치는 법: 함수 반환 타입을 `Result<..>` 로 바꾸거나 `match` 로 직접 처리.

**2. `?` 가 에러 타입을 변환하지 못함**
```
error[E0277]: `?` couldn't convert the error to `ConfigError`
  = help: the trait `From<std::io::Error>` is not implemented for `ConfigError`
```
→ 의미: 함수의 에러 타입과 `?` 한 표현식의 에러 타입이 다른데 `From` 이 없습니다. 고치는 법: `impl From<io::Error> for ConfigError` 를 추가하거나 `.map_err(ConfigError::Io)?`.

**3. `Display` 를 구현하지 않고 `{}` 로 출력**
```
error[E0277]: `ConfigError` doesn't implement `std::fmt::Display`
```
→ `impl fmt::Display for ConfigError` 를 구현하거나, 디버깅용이면 `{:?}`.

**4. `Result` 를 사용하지 않음 (경고)**
```
warning: unused `Result` that must be used
  = note: this `Result` may be an `Err` variant, which should be handled
```
→ 에러를 조용히 버리는 코드입니다. 정말 버릴 거면 `let _ = f();` 처럼 의도를 드러내세요.

**5. `?` 로 꺼낸 값의 타입이 기대와 다름**
```
error[E0308]: mismatched types
  |
  |     let p: u16 = parse_number(1, "port", "80", 65535)?;
  |            ---   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `u16`, found `u64`
  |            expected due to this
help: you can convert a `u64` to a `u16` and panic if the converted value doesn't fit
```
→ 의미: `?` 는 에러만 처리하고 성공 값의 타입은 그대로입니다. 고치는 법: 범위를 이미 검사했다면 `as u16`, 아니라면 `u16::try_from(n)` (Ch12).

**6. `unwrap()` on Option/Result 의 런타임 panic (컴파일은 통과!)**
```
thread 'main' panicked at src/main.rs:5:30:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
```
→ 컴파일러가 못 잡는 영역입니다. 외부 입력에는 `unwrap` 대신 `?` / `match`.

## 🛠️ 실습: 설정 파일 파서

### 무엇을 만드나요
`data/app.conf` (key = value, `#` 주석)를 타입이 있는 `Config { name, port: u16, debug: bool, max_retries: u8 }` 로 파싱합니다. 그리고 잘못된 입력 여러 개를 파싱해 **줄 번호가 포함된 에러 메시지**를 출력합니다.

```
loaded: name=my-service port=8080 debug=true max_retries=5
error: cannot read config file
-- bad inputs --
no equals        error: line 2: expected 'key = value'
unknown key      error: line 3: unknown key 'colour'
port too big     error: line 2: invalid value for 'port': out of range (0-65535)
...
missing name     error: missing required key 'name'
...
-- defaults --
minimal: name=tiny port=9000 debug=false max_retries=3
port_or_default(good) = 9000
port_or_default(bad)  = 8080
```

규칙: `name`·`port` 필수, `debug` 기본 `false`, `max_retries` 기본 `3`, 같은 키 중복은 에러, 모르는 키는 에러.

### 파일 구성
| 경로 | 설명 |
|---|---|
| `data/app.conf` | 입력 파일 |
| `python/main.py` | 예외 클래스 계층 버전 |
| `cpp/main.cpp` | 예외 + `std::optional` 버전 |
| `rust/src/main.rs` | **실습 파일** (TODO 9개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (챕터 디렉터리 `chapters/ch06_error_handling` 에서)
```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch06_error_handling
cargo test -p ch06_error_handling
cargo run -q -p ch06_error_handling --example solution
../../scripts/compare.sh ch06
```

### TODO 목록
| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `Display for ConfigError` | variant 별 메시지, Debug vs Display |
| 2 | `Error::source` | 에러 체인 |
| 3 | `From<io::Error>` | `?` 가 쓰는 변환 규칙 |
| 4 | `parse_line` | `Result<Option<..>>`, `split_once`, `ok_or` + `?` |
| 5 | `parse_number` | `parse`, `map_err`, 범위 검사 |
| 6 | `parse_bool` | 문자열 `match` |
| 7 | `parse_config` | `?`, `let else`, `ok_or_else`, `unwrap_or` |
| 8 | `load_config` | `?` 로 io::Error 자동 변환 |
| 9 | `port_or_default` | `map` + `unwrap_or` |

### 완료 기준
- `cargo test -p ch06_error_handling` 이 모두 통과
- `../../scripts/compare.sh ch06` 에서 세 언어의 출력이 동일

### 🔬 실험 목록
1. `main` 안의 `experiment_question_mark` 함수 주석을 풀고, TODO(3) 의 `impl From` 을 잠시 주석 처리해 보세요 → `E0277 ?` couldn't convert the error`.
2. `parse_config(minimal);` 처럼 결과를 버려 보세요 → `unused Result` 경고. Python/C++ 에서는 같은 실수가 조용히 지나갑니다.
3. `parse_number` 의 `.map_err(..)?` 를 `.unwrap()` 으로 바꾸고 `port = 99999999999999999999999` 를 넣어 보세요 → panic. 에러 처리 vs 크래시의 차이를 체감합니다.

## 🤖 바이브 코딩 포인트
- AI 코드에서 **`unwrap()` / `expect()` 의 개수**를 세어 보세요. 테스트와 `main` 초기화 밖에서 외부 입력(파일, 네트워크, 사용자 입력)에 쓰였다면 "panic 하지 않고 `?` 로 전파하게 바꿔 줘" 라고 요청하세요.
- 함수 시그니처가 `Result<T, String>` 이면 동작은 하지만 에러 종류를 구분할 수 없습니다. "커스텀 에러 enum + `thiserror` 로 바꿔 줘" 가 좋은 리팩터링 요청입니다 (ch16).
- `Box<dyn Error>` / `anyhow::Result` 는 앱(바이너리) 코드에 적합하고, 라이브러리 코드에서는 구체적인 에러 enum 이 낫습니다. AI 가 어느 쪽을 골랐는지 확인하세요.
- `?` 앞의 `.map_err(|e| ...)` 는 에러 변환 지점입니다. 에러 메시지에 **문맥(어떤 파일, 몇 번째 줄)** 이 들어갔는지 리뷰하세요.
- `let _ = something_fallible();` 는 에러를 의도적으로 버립니다. 정말 버려도 되는지 주석이나 이유가 있는지 확인하세요.
- "에러 메시지에 줄 번호/키 이름을 포함하고, 첫 에러에서 멈추도록" 처럼 **에러 정책**을 프롬프트에 명시하면 결과 품질이 확 올라갑니다.

## ✅ 셀프 체크
1. panic 과 `Result` 는 각각 어떤 상황에 쓰나요?
<details><summary>답</summary>panic 은 복구할 수 없는 버그(불변식 위반, 범위 초과)에, `Result` 는 호출한 쪽이 대응할 수 있는 예상 가능한 실패(파일 없음, 입력 오류)에 씁니다.</details>

2. `let text = fs::read_to_string(path)?;` 의 `?` 는 정확히 무엇을 하나요?
<details><summary>답</summary>`Ok(v)` 이면 `v` 를 꺼내고, `Err(e)` 이면 `e` 를 함수의 반환 에러 타입으로 `From::from` 변환한 뒤 즉시 `return Err(..)` 합니다.</details>

3. `ok_or(e)` 와 `ok_or_else(|| e)` 의 차이는?
<details><summary>답</summary>`ok_or` 는 에러 값을 항상 미리 만들고, `ok_or_else` 는 `None` 일 때만 클로저를 실행해 만듭니다. 만드는 비용이 있으면 `_else` 쪽이 낫습니다.</details>

4. 커스텀 에러 타입이 `Display` 와 `Debug` 를 모두 갖는 이유는?
<details><summary>답</summary>`std::error::Error` 트레이트가 둘을 요구합니다. `Display` 는 사용자용 메시지, `Debug` 는 개발자용 상세 출력(`{:?}`, `unwrap` 실패 메시지 등)입니다.</details>

5. C++ 예외와 비교해 `Result` 의 장점과 단점은?
<details><summary>답</summary>장점: 실패 가능성이 시그니처에 드러나고 처리를 빠뜨리면 경고/컴파일 에러, 비용이 예측 가능(스택 풀기 없음). 단점: 전파 코드(`?`, `From`)와 에러 타입 정의가 필요해 약간 장황합니다.</details>

## 📚 더 알아보기
- The Rust Book: [Ch09 Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- Rust by Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html)
- std docs: [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html), [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html), [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
- 다음 단계: ch16 에서 `thiserror` / `anyhow` 로 이 보일러플레이트를 줄입니다.
