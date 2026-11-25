# Ch01. Cargo와 첫 프로그램

> `cargo` 하나로 만들고·돌리고·검사하고·테스트한다. 함수의 마지막 식이 반환값이다.  ·  예상 소요: 이론 40분 / 실습 40분  ·  선행: 없음 (첫 장)

## 🎯 학습 목표

- `rustup` / `rustc` / `cargo` 가 각각 무슨 일을 하는지 구분하고, C++ (컴파일러 + CMake) · Python (인터프리터) 과 비교해 설명할 수 있다.
- `cargo new / build / run / check / test`, `--release`, `cargo fmt`, `cargo clippy` 를 목적에 맞게 골라 쓴다.
- `println!` / `format!` 이 왜 `!` 가 붙은 "매크로"인지, 형식 문자열(`{:>7.1}`)을 어떻게 쓰는지 안다.
- 함수를 정의하고, "마지막 식 = 반환값" 규칙과 세미콜론 한 글자의 의미 차이를 이해한다.
- 이 저장소를 쓰는 법 (`compare.sh`, `cargo -p`, 정답 위치, "먼저 시도 → 그다음 엿보기")을 익힌다.

---

## 🗺️ 먼저: 이 저장소 사용법

이 코스는 **같은 프로그램을 Python / C++ / Rust 세 언어로** 만들어 놓고, 출력이 **한 글자도 다르지 않은지** 비교하며 배웁니다.
이미 아는 C++20 / Python 코드를 "지도"로 삼아서 Rust 를 읽는 방식이에요.

```
chapters/ch01_cargo_basics/
  README.md              <- 지금 읽는 글 (이론 + 실습 안내)
  python/main.py         <- 참조 구현 (Python)
  cpp/main.cpp           <- 참조 구현 (C++20)
  rust/src/main.rs       <- ★ 여러분이 채울 파일 (TODO 가 들어 있어요)
  rust/examples/solution.rs  <- 정답 (막힐 때만!)
```

### 자주 쓰는 명령 (모두 챕터 폴더에서 실행)

```bash
cd chapters/ch01_cargo_basics

python3 python/main.py                       # Python 실행
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x   # C++ 실행

cargo run -q -p ch01_cargo_basics            # 내가 푸는 Rust 실행
cargo test -p ch01_cargo_basics              # 내가 푸는 Rust 테스트
cargo run -q -p ch01_cargo_basics --example solution   # 정답 실행

../../scripts/compare.sh ch01                # Python / C++ / Rust(내 코드) 출력 비교
../../scripts/compare.sh ch01 --solution     # Python / C++ / Rust(정답) 출력 비교
```

- **`-p ch01_cargo_basics`** : 이 저장소는 여러 챕터가 한 "워크스페이스"에 들어 있어서, 어느 패키지(`-p` = package)를
  돌릴지 알려 줘야 합니다. 챕터 폴더 *안의* `rust/` 에서는 `-p` 를 생략해도 돼요.
- **정답은 `rust/examples/solution.rs`** 에 있습니다. 처음부터 열어 보지 마세요.
  **"먼저 시도 → 막히면 Python/C++ 같은 이름의 함수 보기 → 그래도 막히면 정답 엿보기"** 순서를 권해요.
  Rust 는 "컴파일러와 대화하며 배우는" 언어라서, 에러 메시지를 직접 읽어 본 경험이 실력이 됩니다.
- `todo!()` 가 남은 상태로 실행하면 `not yet implemented: TODO(2): ...` 라는 패닉(panic, 프로그램 비정상 종료)이 납니다.
  "에러"가 아니라 "여기부터 풀어 주세요"라는 안내예요.

### 에디터 설정 (강력 추천): rust-analyzer 와 인레이 힌트

VS Code 라면 **rust-analyzer** 확장을 설치하세요. 가장 큰 도움은 **인레이 힌트(inlay hints)** 입니다.
`let x = 5;` 옆에 회색 글씨로 `: i32` 처럼 **컴파일러가 추론한 타입**을 그려 줍니다.
Rust 는 타입을 거의 생략하고 쓰기 때문에, 이 힌트가 있어야 "이 변수가 `String` 인지 `&str` 인지"를 눈으로 확인할 수 있어요.
(AI가 만든 코드를 리뷰할 때도 큰 도움이 됩니다.) 에러는 저장하면 바로 빨간 줄로 나타나고, 💡 전구 메뉴로 자동 수정도 됩니다.

### 코드를 바로 돌려 보고 싶을 때: Rust Playground

이 README 의 코드 조각(표준 라이브러리만 쓰는 것들)은 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있어요.
왼쪽 위 `Run` 버튼, `Tools → Rustfmt / Clippy` 도 됩니다. 설치 없이 실험하기 좋습니다. 이후 챕터에서는 "이 조각은 Playground 에서 돌아간다"고 매번 말하지 않을게요.

---

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 도구 설치/버전 관리 | 시스템 패키지, 직접 | pyenv / 시스템 | `rustup` (툴체인 설치·전환·업데이트) |
| 컴파일러 / 실행기 | `g++` / `clang++` | `python3` (인터프리터) | `rustc` (보통 직접 호출 안 함) |
| 빌드 + 패키지 관리 | CMake + (vcpkg / Conan) | pip + venv | **`cargo` 하나로 전부** |
| 설정 파일 | `CMakeLists.txt` | `pyproject.toml` | `Cargo.toml` |
| 잠금 파일 | (없음 / conan.lock) | `requirements.txt`, `uv.lock` | `Cargo.lock` |
| 빌드 단위 | 번역 단위(.cpp), 정적/동적 라이브러리 | 모듈(.py) | **crate** (컴파일 단위 = 라이브러리 또는 실행 파일) |
| 코드 포맷터 | clang-format | black / ruff format | `cargo fmt` (rustfmt, 공식) |
| 린터 | clang-tidy | ruff / pylint | `cargo clippy` (공식) |
| 단위 테스트 | GoogleTest 등 별도 설치 | `unittest` / pytest | **내장** `#[test]` + `cargo test` |
| 출력 | `std::cout`, `std::format`(C++20) | `print`, f-string | `println!`, `format!` (매크로) |
| 함수 반환 | `return x;` | `return x` | 마지막 **식**(세미콜론 없음) 또는 `return x;` |
| 정수/실수 섞기 | 암묵적 변환 허용 | 자동 승격 | **불가** (명시적 변환) |
| 디버그 vs 최적화 | `-O0` / `-O2` 플래그 | 없음 | `cargo build` / `cargo build --release` |

---

## 📘 이론

### 1. 도구 삼총사: rustup, rustc, cargo

```bash
$ rustc --version
rustc 1.97.0 (2d8144b78 2026-07-07)
$ cargo --version
cargo 1.97.0 (c980f4866 2026-06-30)
```

- **`rustup`** — 툴체인 매니저. Rust 컴파일러, cargo, rustfmt, clippy 를 설치하고 `rustup update` 로 최신화합니다.
  (Python 의 pyenv, Node 의 nvm 과 비슷한 역할)
- **`rustc`** — 진짜 컴파일러. `.rs` 파일 → 실행 파일. g++ 에 해당하지만, **직접 부를 일은 거의 없습니다.**
- **`cargo`** — 빌드 도구 + 패키지 매니저 + 테스트 러너 + 문서 생성기. CMake + vcpkg + GoogleTest + Doxygen 을 한 도구로 합쳐 놓았다고 생각하세요.
  `cargo build` 는 내부에서 `rustc` 를 알맞은 옵션으로 불러 줍니다.

> 💡 **쉬운 설명 — "툴체인"이란?**
> 코드를 기계가 읽는 파일로 바꿔 주는 도구 묶음(컴파일러 + 보조 도구)을 툴체인이라고 불러요. 요리로 치면 칼·도마·냄비 세트입니다.
> `rustup` 은 이 세트를 사 오고 최신 버전으로 바꿔 주는 "도구 가게"예요.

### 2. package, crate, workspace — 용어 정리

| 용어 | 뜻 | C++ / Python 에서 비슷한 것 |
|---|---|---|
| **crate** | 컴파일러가 한 번에 컴파일하는 단위. 실행 파일(binary crate) 또는 라이브러리(library crate) | CMake 의 target (`add_executable`, `add_library`) |
| **package** | `Cargo.toml` 이 있는 폴더. crate 를 1개 이상 담음 | CMake project / Python 배포 패키지 |
| **workspace** | 여러 package 를 한 덩어리로 묶어 한 번에 빌드·공유 | CMake `add_subdirectory` 로 묶은 최상위 프로젝트 |
| **module** | crate 안의 이름 공간 (10장) | namespace / Python 모듈 |

이 저장소의 루트 `Cargo.toml` 이 **workspace** 이고, 챕터마다 `rust/` 가 하나의 **package**(= 실행 crate)입니다.
그래서 `cargo run -p ch01_cargo_basics` 처럼 "어느 package 를 돌릴지" 지정해요.

### 3. 첫 프로젝트: `cargo new`

```bash
$ cargo new hello_cargo
    Creating binary (application) `hello_cargo` package
$ cd hello_cargo && ls -a
.  ..  .git  .gitignore  Cargo.toml  src
```

`Cargo.toml` (CMakeLists.txt + pyproject.toml 역할):

```toml
[package]
name = "hello_cargo"
version = "0.1.0"
edition = "2024"      # 언어 "에디션" (아래 설명)

[dependencies]        # 외부 라이브러리(crate)는 여기에. 1~12장에서는 비워 둡니다.
```

`src/main.rs` (`cargo new` 가 만들어 주는 전부):

```rust
fn main() {
    println!("Hello, world!");
}
```

> 💡 **쉬운 설명 — edition(에디션)이란?**
> `-std=c++20` 같은 "언어 버전 스위치"와 비슷한데, **과거 코드를 깨뜨리지 않는 약속**이 핵심이에요.
> 2015 / 2018 / 2021 / 2024 에디션이 있고, 에디션이 달라도 같은 프로그램 안에서 서로의 crate 를 섞어 쓸 수 있습니다.
> 새 프로젝트는 `cargo new` 가 알아서 최신 에디션(`2024`)을 적어 줍니다.

### 4. 자주 쓰는 cargo 명령 5개

```bash
$ cargo build        # 컴파일만. 결과: target/debug/hello_cargo
   Compiling hello_cargo v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s

$ cargo run          # 빌드 + 실행
     Running `target/debug/hello_cargo`
Hello, world!

$ cargo run -q       # -q(quiet): 컴파일 로그 숨기고 프로그램 출력만
Hello, world!

$ cargo check        # 코드 생성 없이 "에러만" 빠르게 검사
    Checking hello_cargo v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s

$ cargo test         # #[test] 함수 실행
running 0 tests
test result: ok. 0 passed; 0 failed; ...
```

| 명령 | 언제 쓰나요? | C++ / Python 감각 |
|---|---|---|
| `cargo check` | **가장 자주.** 코드를 고치고 에러만 보고 싶을 때. 실행 파일은 안 만들어서 빠름 | `g++ -fsyntax-only` |
| `cargo run` | 돌려 보고 싶을 때 | `cmake --build . && ./app` / `python main.py` |
| `cargo build` | 실행 파일만 만들 때 | `cmake --build .` |
| `cargo test` | 테스트 | `ctest` / `pytest` |
| `cargo build --release` | 최적화된 최종 빌드 | `-O2` / `-O3` |

> 💡 **쉬운 설명 — 왜 `cargo check` 가 빠를까?**
> 컴파일은 "검사 → 기계어 생성 → 링크" 순서인데, 오래 걸리는 건 뒤쪽(기계어 생성, 링크)이에요.
> `check` 는 "검사"까지만 하고 멈춥니다. Rust 는 검사(타입, 소유권 규칙 등)가 엄격해서 **에러의 대부분이 이 단계에서** 잡혀요.
> 그래서 편집 → `cargo check` → 수정 사이클이 Rust 개발의 기본 리듬입니다. (rust-analyzer 도 내부에서 같은 일을 합니다.)

### 5. debug vs release

```bash
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 0.09s     # target/release/ 에 생성
```

| | `cargo build` (dev/debug) | `cargo build --release` |
|---|---|---|
| 최적화 | 없음 (`-O0` 비슷) | 있음 (`-O3` 급) |
| 컴파일 속도 | 빠름 | 느림 |
| 실행 속도 | 느림 (종종 몇 배~수십 배 차이) | 빠름 |
| **정수 오버플로** | **패닉(프로그램 중단)** | 조용히 감김(wrap) — 2장에서 자세히 |
| 디버깅 정보 | 있음 | 기본 없음 |

> ⚠️ **C++ 개발자가 헷갈리는 점**
> 1. "성능이 이상하게 느리다" → 십중팔구 `--release` 를 빼먹은 겁니다. 벤치마크는 **반드시** release 로 하세요.
> 2. C++ 에서는 `-O0` 로 빌드해도 부호 있는 정수 오버플로가 *UB(정의되지 않은 동작)* 일 뿐 검사해 주지 않아요.
>    Rust debug 빌드는 오버플로를 **잡아서 패닉**시킵니다. 같은 코드가 debug 와 release 에서 다르게 동작할 수 있다는 뜻이니 기억해 두세요.
> 3. 빌드 결과는 `target/` 폴더에 쌓입니다 (Git 에 올리지 않습니다 — `cargo new` 가 `.gitignore` 에 이미 넣어 줘요).

### 6. 포맷터와 린터: `cargo fmt`, `cargo clippy`

```bash
cargo fmt                 # 코드를 공식 스타일로 자동 정리 (논쟁 끝!)
cargo fmt -- --check      # 정리가 필요한지 검사만 (CI 용)
cargo clippy              # "이렇게 쓰는 게 더 Rust 답다"는 조언 + 흔한 버그 탐지
cargo clippy -- -D warnings   # 조언을 에러로 취급 (이 저장소의 정답 코드는 이 기준을 통과합니다)
```

clippy 는 **무료로 따라붙는 코드 리뷰어**입니다. AI 가 만든 코드에 돌려 보면 "C++ 스타일로 쓴 Rust"를 많이 잡아 줘요 (12장에서 본격적으로).

### 7. `println!` 과 `format!` — 왜 `!` 가 붙을까?

```rust
fn main() {
    let name = "Cargo";
    let temp = 21.456;
    println!("Hello, {}!", name);          // 위치 인자
    println!("Hello, {name}!");            // 변수 이름을 바로 (Rust 2021+)
    println!("{:.1}도", temp);             // 소수점 1자리 -> 21.5도
    println!("[{:>8.2}]", temp);           // 폭 8, 오른쪽 정렬, 소수 2자리 -> [   21.46]
    println!("[{:<6}] [{:^6}]", "ab", "cd");   // 왼쪽 / 가운데 정렬
    println!("{:?}", vec![1, 2, 3]);       // 디버그 출력 -> [1, 2, 3]
    println!("{{중괄호}}");                  // 중괄호 자체는 두 번 -> {중괄호}

    let line = format!("{name}:{temp:.0}"); // 출력하지 않고 String 으로 돌려줌
    println!("{line}");                    // Cargo:21
}
```

- `{}` = 사람이 읽는 출력(`Display`), `{:?}` = 개발자용 디버그 출력(`Debug`). Vec 같은 건 `{:?}` 만 됩니다.
- 형식 지정자: `[[채움]정렬][폭][.정밀도]`  — `>` 오른쪽, `<` 왼쪽, `^` 가운데. C++ `std::format` 과 거의 같고 (`{:>7.1f}` 에서 `f` 만 빠짐), Python f-string 과도 매우 비슷합니다.
- **`!` 는 "이건 함수가 아니라 매크로"라는 표시**입니다. C 의 `#define` 매크로와 달리 Rust 매크로는 컴파일러가 문법을 이해하는 코드 생성기라서 안전하고,
  가변 인자(`println!("{} {}", a, b)`)처럼 일반 함수로는 못 하는 일(Rust 함수는 가변 인자 불가)을 가능하게 해 줍니다.
  형식 문자열과 인자 개수가 안 맞으면 **컴파일 에러**입니다 (C 의 `printf` 와 큰 차이).

> 💡 **쉬운 설명 — 매크로**
> "코드를 써 주는 코드"입니다. `println!("{}", x)` 를 쓰면 컴파일러가 이걸 실제 출력 코드로 펼쳐 줘요.
> 일단은 "이름 끝에 `!` 가 있으면 매크로, 문법은 함수처럼 쓰되 특별한 능력이 있다" 정도로 알면 충분합니다.

> ⚠️ **C++ 개발자가 헷갈리는 점**
> `println!("{}", x)` 는 `x` 를 **빌려서(borrow)** 읽기만 합니다. 값이 이동(move)하거나 복사되지 않아요. (3~4장에서 이유를 배웁니다.)

### 8. 함수와 "식(expression)"

```rust
fn c_to_f(c: f64) -> f64 {   // 매개변수 타입과 반환 타입은 반드시 적습니다
    c * 9.0 / 5.0 + 32.0     // 세미콜론 없음 = 이 값이 반환값
}

fn log_value(x: f64) {       // 반환 타입 생략 = () (아무것도 안 돌려줌, C++ void)
    println!("{x}");         // 세미콜론 있음 = "문(statement)": 값을 버림
}

fn early(x: f64) -> f64 {
    if x < 0.0 {
        return 0.0;          // 중간에 빠져나갈 때만 return 을 씁니다
    }
    x
}
```

Rust 에서는 **"식(expression)"이 값을 만들고, "문(statement)"은 값을 만들지 않아요.**
`;` 는 "이 식의 값을 버린다"는 뜻의 연산자 같은 것입니다. 그래서 함수 마지막 줄의 `;` 하나가 반환값을 `()` 로 바꿔 버립니다.

> ⚠️ **C++ / Python 개발자가 헷갈리는 점**
> - C++/Python 은 `return` 이 기본이지만 Rust 의 관용구는 **마지막 식으로 반환**, `return` 은 "일찍 탈출"할 때만입니다.
> - 인자/반환 타입은 **반드시 명시**합니다 (함수 안의 변수는 추론). 함수 시그니처는 "AI 가 쓴 코드를 읽을 때 가장 먼저 보는 곳"입니다.
> - 함수 오버로딩이 없고, 기본 인자도 없습니다 (이름을 다르게 짓거나 12장의 builder 를 씁니다).
> - 숫자 리터럴 `100` 은 정수, `100.0` 은 실수입니다. `c_to_f(100)` 은 컴파일 에러 — C++ 의 `f(100)` 과 다른 점이에요.

### 9. 컴파일 모델 비교 — 에러가 "언제" 드러나는가

| | C++ | Python | Rust |
|---|---|---|---|
| 실행 전 검사 | 문법·타입 (템플릿은 사용 시점) | 문법만 | 문법·타입·**소유권·빌림·스레드 안전성** |
| 실행 방식 | 컴파일 → 링크 → 네이티브 실행 파일 | 인터프리터가 줄 단위 실행 | 컴파일 → 링크 → 네이티브 실행 파일 |
| 오타/타입 오류 발견 | 컴파일 때 | **그 줄이 실행될 때** | 컴파일 때 |
| 헤더/전방선언 | 필요 | 불필요 | **불필요** (파일 순서, 선언 순서 무관) |

Rust 컴파일은 C++ 처럼 네이티브 코드를 만들지만, "에러를 컴파일 때 최대한 잡는다"는 철학이 훨씬 강합니다.
그래서 처음에는 컴파일 에러를 많이 만나요. **그건 Rust 가 일하고 있다는 신호**입니다. 에러 메시지에는 대부분 고치는 방법까지 적혀 있어요.

---

## 🧯 자주 만나는 컴파일 에러

아래는 이 챕터의 코드로 **실제로 rustc 1.97 이 출력한** 메시지입니다. (`rust/src/main.rs` 의 `experiments()` 에서 직접 재현해 볼 수 있어요.)

### 1) 반환값 뒤에 세미콜론 — `E0308`

```rust
fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0;
}
```
```text
error[E0308]: mismatched types
 --> p1.rs:1:22
  |
1 | fn c_to_f(c: f64) -> f64 {
  |    ------            ^^^ expected `f64`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     c * 9.0 / 5.0 + 32.0;
  |                         - help: remove this semicolon to return this value
```
**의미**: "`f64` 를 돌려준다고 해 놓고 아무것도(`()`) 안 돌려줬다." **고치기**: 마지막 `;` 를 지웁니다. 컴파일러가 정확히 그 위치를 짚어 줘요.

### 2) 정수를 실수 자리에 — `E0308`

```rust
println!("{}", c_to_f(100));
```
```text
error[E0308]: mismatched types
 --> p2.rs:5:27
  |
5 |     println!("{}", c_to_f(100));
  |                    ------ ^^^ expected `f64`, found integer
  |                    |
  |                    arguments to this function are incorrect
  ...
help: use a float literal
  |
5 |     println!("{}", c_to_f(100.0));
  |                              ++
```
**의미**: Rust 는 `i32 → f64` 같은 암묵 변환을 하지 않습니다. **고치기**: `100.0` 이라고 쓰거나, 변수라면 `f64::from(n)` / `n as f64`.

### 3) 매크로의 `!` 누락 — `E0423`

```text
error[E0423]: expected function, found macro `println`
 --> p5.rs:2:5
  |
2 |     println("hello");
  |     ^^^^^^^ not a function
help: use `!` to invoke the macro
```
**고치기**: `println!("hello");`

### 4) 오타 — `E0425`

```text
error[E0425]: cannot find value `tmep` in this scope
 --> p3.rs:3:20
  |
3 |     println!("{}", tmep);
  |                    ^^^^
help: a local variable with a similar name exists
```
Python 이었다면 그 줄이 *실행되는 순간* `NameError` 가 났을 거예요. Rust 는 **실행 전에** 잡습니다.

### 5) `{}` 로 출력할 수 없는 타입 — `E0277`

```rust
let v = vec![1, 2, 3];
println!("{}", v);
```
```text
error[E0277]: `Vec<{integer}>` doesn't implement `std::fmt::Display`
  = help: the trait `std::fmt::Display` is not implemented for `Vec<{integer}>`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```
**고치기**: `{:?}` (디버그 출력). `Display` 는 "사람에게 보여줄 형식"이 정해진 타입만 구현하는 트레이트(8장)입니다.

---

## 🛠️ 실습: 온도 변환표

### 무엇을 만드나요

섭씨 ↔ 화씨 변환 함수를 만들고, 표 · 왕복 변환 · 간단한 통계를 출력합니다. 기대 출력 (일부):

```text
Hello, Cargo!
=== Celsius to Fahrenheit ===
      C       F
  -10.0    14.0
    0.0    32.0
   10.0    50.0
   ...
   40.0   104.0
=== round trip (F -> C) ===
  14.0 F =  -10.0 C
  ...
=== sample readings ===
readings: 22.5, 18.0, 30.1, 15.7, 25.3
min: 15.7  max: 30.1  avg: 22.32
```

### 파일 구성

| 파일 | 역할 |
|---|---|
| `python/main.py` | Python 참조 구현 |
| `cpp/main.cpp` | C++20 참조 구현 (`std::format` 사용) |
| `rust/src/main.rs` | ★ 여러분이 채울 파일 (TODO 6개 + 🔬 실험실) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법

```bash
cd chapters/ch01_cargo_basics
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch01_cargo_basics                       # 처음엔 TODO(1) 패닉이 나는 게 정상!
cargo test -p ch01_cargo_basics
../../scripts/compare.sh ch01                           # 세 언어 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| TODO(1) | `greeting` — `format!` 으로 문자열 만들기 | 매크로, `String` 반환 |
| TODO(2) | `c_to_f` | 마지막 식 반환, 실수 리터럴 |
| TODO(3) | `f_to_c` | 연산 순서 |
| TODO(4) | `table_line` — `{:>7.1}` 정렬 | 형식 지정자 |
| TODO(5) | `print_table` — `(-10..=40).step_by(10)` | 범위, `for`, `f64::from` |
| TODO(6) | `min_max_avg` — 튜플 반환 | `fold`, 슬라이스(맛보기) |

### 완료 기준

- `cargo test -p ch01_cargo_basics` → 6개 모두 `ok`
- `../../scripts/compare.sh ch01` → `✅ python vs rust : 출력이 똑같아요`

### 🔬 실험 목록 (`rust/src/main.rs` 의 `experiments()` 함수)

| 실험 | 해볼 것 | 기대 에러 |
|---|---|---|
| 1 | 마지막 식에 `;` 붙이기 | `E0308` |
| 2 | `c_to_f(100)` 호출 | `E0308` |
| 3 | `println("hello")` | `E0423` |
| 4 | 변수 이름 오타 | `E0425` |
| 5 | `println!("{}", vec)` | `E0277` |

추가 실험: 정답 `solution.rs` 로 `cargo run --release -q -p ch01_cargo_basics --example solution` 을 해 보고,
`target/debug/examples/` 와 `target/release/examples/` 에 실행 파일이 따로 생기는 걸 확인해 보세요.

---

## 🤖 바이브 코딩 포인트

AI 가 Rust 코드를 만들어 줄 때 **이 챕터 수준에서** 확인할 것들:

- **`cargo check` → `cargo clippy` → `cargo test` 순서로 먼저 돌려 보세요.** 에러 없이 통과하면 "컴파일되는 코드"까지는 보장됩니다. 특히 `clippy` 경고가 많으면 C++/Python 식으로 쓴 코드일 가능성이 높아요.
- **`Cargo.toml` 의 `[dependencies]` 를 읽으세요.** AI 가 필요 이상으로 crate 를 추가하지 않았는지, 버전이 존재하는지 확인합니다 (없는 crate 이름이나 오래된 API 를 지어내는 경우가 있어요).
- **함수 시그니처(`fn name(...) -> ...`)부터 읽으세요.** 인자와 반환 타입만 봐도 "무엇을 받아 무엇을 돌려주는지"가 보입니다.
- 프롬프트 팁: *"edition 2024, std 만 사용, `cargo clippy -- -D warnings` 를 통과하게, 테스트(`#[cfg(test)]`)도 같이 작성해 줘"* 처럼 **검증 명령을 요구 사항에 포함**하세요.
- 프롬프트 팁: AI 가 `.unwrap()` 을 남발하거나 `println!` 디버그 출력을 남겼다면 "에러 처리는 Result 로, 디버그 출력은 제거해 줘"라고 구체적으로 요청합니다 (6장).
- 컴파일 에러가 나면 **에러 전문(`error[E....]` 부터 `help:` 까지)** 을 그대로 AI 에게 붙여 주세요. rustc 의 메시지는 정보가 풍부해서 AI 도 훨씬 정확히 고칩니다.

---

## ✅ 셀프 체크

**Q1.** `cargo check` 와 `cargo build` 의 차이는? 언제 `check` 를 쓰나요?

<details><summary>답</summary>

`check` 는 타입/소유권 검사까지만 하고 기계어 생성·링크를 건너뛰어 훨씬 빠릅니다. 코드를 고치면서 "에러가 있는지"만 볼 때 씁니다. 실행 파일이 필요할 때만 `build` / `run` 을 합니다.
</details>

**Q2.** 아래 함수는 왜 컴파일되지 않나요? 한 글자만 고친다면?

```rust
fn double(x: i32) -> i32 {
    x * 2;
}
```

<details><summary>답</summary>

마지막 `;` 때문에 함수 본문이 `()` 를 돌려주게 되어 `E0308` 이 납니다. `;` 를 지우면 `x * 2` 식이 반환값이 됩니다.
</details>

**Q3.** `println!` 이 함수가 아니라 매크로인 이유 두 가지를 말해 보세요.

<details><summary>답</summary>

① Rust 함수는 가변 인자를 받을 수 없는데 `println!` 은 인자 개수가 자유롭습니다. ② 형식 문자열(`"{:>7.1}"`)을 컴파일 때 해석해서 인자 개수/타입이 맞지 않으면 컴파일 에러로 알려 줍니다 (C 의 `printf` 는 런타임 UB).
</details>

**Q4.** 같은 프로그램이 debug 빌드에서는 패닉이 나는데 release 빌드에서는 조용히 넘어가는 경우의 예는? (힌트: 숫자)

<details><summary>답</summary>

정수 오버플로. debug 에서는 `attempt to add with overflow` 패닉, release 에서는 값이 한 바퀴 돌아 감깁니다 (2장).
</details>

**Q5.** `Cargo.toml`, `Cargo.lock`, `target/` 중 Git 에 올리지 않는 것은? (실행 파일을 만드는 프로젝트 기준)

<details><summary>답</summary>

`target/` 만 올리지 않습니다. 실행 파일(application)이라면 `Cargo.lock` 도 올려서 모두가 같은 의존성 버전을 쓰게 합니다.
</details>

---

## 📚 더 알아보기

- The Rust Book 1장 *Getting Started*: <https://doc.rust-lang.org/book/ch01-00-getting-started.html>
- The Rust Book 2장 *Guessing Game* (cargo + 입출력 맛보기): <https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html>
- The Rust Book 3.3 *Functions*: <https://doc.rust-lang.org/book/ch03-03-how-functions-work.html>
- The Cargo Book: <https://doc.rust-lang.org/cargo/>
- Rust by Example — Formatted print: <https://doc.rust-lang.org/rust-by-example/hello/print.html>
- `std::fmt` (형식 지정자 전체 문법): <https://doc.rust-lang.org/std/fmt/>
- Rust Playground: <https://play.rust-lang.org>
