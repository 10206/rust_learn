# 02. AI 에이전트와 함께 Rust 바이브 코딩하기

> 이 커리큘럼의 최종 목표는 "Rust 문법을 외우는 것"이 아니라, **AI가 써 준 Rust 코드를 읽고, 의심하고, 방향을 바꿀 수 있게 되는 것**입니다. 이 문서는 그때 쓰는 실전 요령 모음이에요.
>
> 이 문서의 코드 블록 중 `compile_fail` 표시가 없는 것은 `rustc 1.97`(edition 2024)에서 실제로 컴파일해 확인했습니다. 컴파일이 **안 되는** 예제도 실제로 실패하는 것을 확인했어요.

## 목차

1. [왜 Rust는 바이브 코딩과 궁합이 좋을까요](#1-왜-rust는-바이브-코딩과-궁합이-좋을까요)
2. [AI가 쓴 Rust를 빠르게 읽는 법](#2-ai가-쓴-rust를-빠르게-읽는-법)
3. [리뷰 체크리스트](#3-리뷰-체크리스트)
4. [프롬프트 템플릿 (한국어 + English)](#4-프롬프트-템플릿-한국어--english)
5. [피드백 루프: compile → clippy → test](#5-피드백-루프-compile--clippy--test)
6. [AI가 자주 하는 Rust 실수 (before / after)](#6-ai가-자주-하는-rust-실수-before--after)
7. ["이게 보이면 이 챕터를 다시 보세요" 매핑표](#7-이게-보이면-이-챕터를-다시-보세요-매핑표)

---

## 1. 왜 Rust는 바이브 코딩과 궁합이 좋을까요

AI가 코드를 써 줄 때 가장 무서운 것은 "겉보기에는 그럴듯하지만 조용히 틀린 코드"입니다. C++이라면 컴파일은 되지만 실행 중에 가끔 터지는(use-after-free, 데이터 레이스) 코드이고, Python이라면 실행해 봐야 알 수 있는 타입 오류입니다.

Rust는 이런 종류의 실수를 **컴파일 단계에서 대부분 막아 줍니다.**

| 위험 | C++20 | Python | Rust |
|---|---|---|---|
| 해제된 메모리 사용 | 컴파일 통과, 런타임 UB | 해당 없음 | 컴파일 에러 (E0382, E0597 …) |
| 스레드 데이터 레이스 | 컴파일 통과, 가끔 터짐 | GIL이 일부 가려 줌 | 컴파일 에러 (`Send`/`Sync`) |
| 처리 안 한 에러/null | 예외가 새거나 null 역참조 | 런타임 `None` 에러 | `Result`/`Option`은 **처리하지 않으면 경고/에러** |
| 빠뜨린 경우 처리 | `switch` 누락은 경고 수준 | `match`도 누락 가능 | 모든 `match`는 빠짐없이 (E0004) |
| 타입 불일치 | 일부 암묵 변환 | 실행 시 오류 | 컴파일 에러 (E0308) |

즉 AI가 쓴 코드를 **일단 컴파일해 보는 것**만으로 사람이 하던 검토의 상당 부분을 컴파일러가 대신해 줍니다. 그래서 바이브 코딩에서는 "AI에게 맡기고 → 컴파일러가 거르고 → 사람은 **컴파일러가 못 잡는 부분**(설계, 의도, 성능, 과도한 복잡도)을 봅니다."

> ⚠️ 하지만 컴파일이 된다고 **맞는 코드**인 것은 아닙니다. 컴파일러는 메모리 안전성과 타입은 보장하지만 "요구사항대로 동작하는지"는 모릅니다. 테스트가 그 몫을 합니다.

---

## 2. AI가 쓴 Rust를 빠르게 읽는 법

긴 코드를 처음부터 끝까지 읽지 마세요. **함수 시그니처(서명)를 먼저** 훑으면 구조의 80%가 보입니다.

### 2.1 시그니처에서 소유권 읽기

```rust
struct Config { name: String }

fn a(c: Config) {}          // 소유권을 가져감: 호출한 쪽은 c 를 더 못 씀
fn b(c: &Config) {}         // 읽기만 빌림
fn c(c: &mut Config) {}     // 수정할 수 있게 빌림
fn d(c: &Config) -> &str { &c.name } // 반환값은 c 에서 빌린 것 (c 보다 오래 못 삼)
fn e(name: &str) -> String { name.to_string() } // 새 소유 값을 만들어 돌려줌

fn main() {
    let cfg = Config { name: "x".into() };
    b(&cfg);
    println!("{}", d(&cfg));
    let mut cfg2 = Config { name: e("y") };
    c(&mut cfg2);
    a(cfg2); // 여기서 cfg2 는 이동됨
}
```

| 매개변수 | 의미 | 읽을 때 던질 질문 |
|---|---|---|
| `T` | 소유권을 **가져감** | 정말 가져가야 하나? 호출자가 이후에 필요하진 않나? |
| `&T` | 읽기만 | 보통 가장 좋은 기본값 |
| `&mut T` | 수정 | 왜 수정해야 하지? 반환값으로 줄 순 없나? |
| `&str` / `&[T]` | 문자열/배열 슬라이스 | `&String` / `&Vec<T>`보다 낫다 (더 많은 입력을 받음) |
| `impl Trait` / `T: Trait` | 제네릭(정적 디스패치) | 경계가 너무 넓거나 좁지 않은가? |
| `Box<dyn Trait>` | 동적 디스패치 | 정말 런타임 다형성이 필요한가? `enum`으로 충분하지 않나? |

### 2.2 눈여겨볼 "냄새" 목록

위에서 아래로 훑으면서 아래 표시가 보이면 속도를 늦추고 읽으세요.

| 보이는 것 | 왜 의심하나요 | 확인할 것 |
|---|---|---|
| **반환 타입 `Result<T, E>`** | 실패 가능한 함수 | 에러 타입 `E`가 무엇인지, 호출부가 `?`로 전파하는지, 삼키지는 않는지(`let _ = ...`, `.ok()`) |
| **`pub` 표면** | 외부에 약속되는 API | `pub`이 꼭 필요한지, 필드까지 `pub`인지 |
| **`unsafe`** | 컴파일러 보증이 꺼진 구간 | 이유가 주석에 있는지, 정말 필요한지. 입문 단계의 코드에 `unsafe`가 나오면 **일단 거절**하고 안전한 방법을 요구하세요. |
| **`.unwrap()` / `.expect()`** | 실패 시 panic으로 프로그램 종료 | 테스트/프로토타입/`main` 최상단이 아니라면 `?`나 명시적 처리로 |
| **`.clone()` 폭탄** | 빌림 에러를 피하려고 복제로 덮은 흔적 | 복제할 필요가 있는지, `&`로 해결되는지. 큰 데이터(`Vec`, `String`)를 루프 안에서 복제하면 성능 문제 |
| **`Rc<RefCell<T>>` / `Arc<Mutex<T>>`** | 소유권 설계를 피해 가는 만능 도구 | 정말 공유 가변 상태가 필요한가? 대개 구조를 바꾸면 `&mut`로 충분 |
| **`'static` 경계 / `&'static str` 매개변수** | 너무 강한 요구 | `thread::spawn`/`tokio::spawn`이 아니면 대개 불필요. 호출부에서 에러를 낳음 |
| **`Box<dyn Error>`** | 에러 종류를 지워 버림 | 라이브러리라면 `thiserror` enum, 앱이라면 `anyhow`가 낫지 않은가 |
| **async 안의 `std::thread::sleep`, 동기 I/O, 무거운 계산** | 런타임 전체를 멈춤 | `tokio::time::sleep`, `tokio::fs`, `spawn_blocking` |
| **`MutexGuard`를 쥔 채 `.await`** | 데드락/`Send` 에러 위험 | 락은 `.await` 전에 놓기 |
| **`as` 변환 연발** | 값이 조용히 잘림 | `try_from`, `checked_*` 로 대체 가능한지 |
| **`_ =>` 만능 분기** | 새 variant가 추가돼도 컴파일러가 알려 주지 않음 | 열거형 전체를 나열할 수 있는지 |
| **`#[allow(...)]`, `#[allow(clippy::...)]`** | 경고를 숨김 | 왜 필요한지 설명이 있는지 |
| **직접 만든 `impl` 대신 `derive`로 충분한 곳** | 불필요한 코드 | `#[derive(Debug, Clone, PartialEq)]` |

### 2.3 5분 읽기 순서

1. `Cargo.toml` — 어떤 크레이트(의존성)가 들어왔나? 필요 이상으로 많지 않은가? `edition = "2024"`인가?
2. `main.rs`/`lib.rs`의 `mod`, `use`, `pub` 목록 — 구조가 한눈에 보입니다.
3. `struct`/`enum` 정의 — 데이터 모양이 맞는가?
4. 함수 **시그니처만** 훑기 (위 2.1, 2.2 표 활용)
5. 에러 타입과 `main`의 흐름
6. `#[cfg(test)]` 테스트 — AI가 쓴 테스트가 정말 의미 있는 것을 검사하나? (`assert!(true)`, 결과를 출력만 하는 테스트는 가치가 없어요)
7. 그다음에야 세부 구현

---

## 3. 리뷰 체크리스트

AI가 만든 PR/코드를 받으면 아래를 순서대로 확인하세요. (복사해서 이슈나 PR 설명에 붙여 쓰세요.)

**빌드와 품질 게이트**
- [ ] `cargo fmt --check` 통과
- [ ] `cargo clippy --all-targets -- -D warnings` 경고 0개
- [ ] `cargo test` 전부 통과 (테스트가 실제로 동작을 검사하는지도 확인)
- [ ] 새 의존성이 있다면 이유가 타당하고 유지보수되는 크레이트인지 확인했다 (`Cargo.toml` 변경 확인)

**소유권 / 성능**
- [ ] 함수 매개변수가 `&str`/`&[T]`/`&T`로 충분한데 `String`/`Vec<T>`/`T`를 받고 있지 않다
- [ ] 불필요한 `.clone()`이 없다 (특히 루프 안, 큰 데이터)
- [ ] `Rc<RefCell<…>>`, `Arc<Mutex<…>>`가 정말 필요한 곳에만 쓰였다
- [ ] 인덱스 루프(`for i in 0..v.len()`) 대신 이터레이터를 쓸 수 있는 곳은 쓰였다

**에러 처리**
- [ ] 라이브러리 코드에 `.unwrap()`/`.expect()`/`panic!`이 없다 (있다면 불변 조건이 주석으로 설명돼 있다)
- [ ] 에러 타입이 적절하다 (라이브러리: `thiserror` enum, 앱: `anyhow` + `.context()`)
- [ ] 에러를 조용히 삼키는 코드(`let _ = ...`, `.ok()`, 빈 `Err(_) => {}`)가 없다
- [ ] 외부 입력(파일, 네트워크, 사용자)을 파싱할 때 실패 경로가 테스트돼 있다

**안전성 / 설계**
- [ ] `unsafe`가 없다 (있다면 정당화 주석과 안전한 대안 검토 기록이 있다)
- [ ] `pub`이 필요한 항목에만 붙어 있다 (기본은 비공개, 크레이트 내부는 `pub(crate)`)
- [ ] `match`가 `_ =>`로 모든 것을 뭉개지 않는다
- [ ] 불필요한 `'static`, 불필요한 라이프타임 표기가 없다

**async / 동시성**
- [ ] `async fn` 안에 블로킹 호출(`std::thread::sleep`, `std::fs`, 큰 계산)이 없다
- [ ] `MutexGuard`(`std::sync::Mutex`)를 쥔 채 `.await` 하지 않는다
- [ ] 외부 호출에 **타임아웃**과 (필요하면) **재시도 상한**이 있다
- [ ] 스레드/태스크를 `spawn`한 뒤 `join`/`await`로 결과와 에러를 확인한다

**보안 / 운영**
- [ ] API 키가 코드에 박혀 있지 않고 환경 변수로 읽는다 (`std::env::var`)
- [ ] 로그나 에러 메시지에 비밀 값이 찍히지 않는다
- [ ] 외부 입력으로 만든 경로/명령어를 그대로 실행하지 않는다

---

## 4. 프롬프트 템플릿 (한국어 + English)

AI에게 Rust 작업을 시킬 때는 **조건을 명시**할수록 결과가 좋아집니다. 아래 템플릿에서 `〈 〉` 부분을 채워 쓰세요.

### 4.1 새 코드 작성 요청

**한국어**

```text
Rust 코드를 작성해 주세요.

[목표]
〈무엇을 하는 코드인지 2~3줄〉

[환경]
- Rust edition 2024, rustc 1.9x (stable)
- 사용할 크레이트: 〈tokio, serde, serde_json, reqwest(rustls-tls) 등. 목록 밖 크레이트는 추가하지 말고 먼저 물어봐 주세요〉
- 구조: 〈src/main.rs 한 파일 / lib.rs + main.rs 분리〉

[에러 처리]
- 라이브러리 코드(lib)는 thiserror 로 에러 enum 을 정의해 주세요.
- 앱 코드(main)는 anyhow::Result 와 .context() 를 사용해 주세요.
- 라이브러리 코드에서 unwrap()/expect()/panic! 금지. (테스트 코드에서는 허용)

[품질 기준]
- 함수 매개변수는 가능한 한 &str, &[T] 로 받아 주세요. 불필요한 clone() 금지.
- unsafe 금지. Rc<RefCell<>>, Arc<Mutex<>> 는 정말 필요할 때만 쓰고 이유를 주석으로 적어 주세요.
- 작성 후 cargo fmt, cargo clippy --all-targets -- -D warnings, cargo test 가 모두 통과해야 합니다.
- 핵심 로직에는 #[cfg(test)] 단위 테스트(정상/에러 경로 모두)를 포함해 주세요.

[설명]
- 소유권/빌림/라이프타임 때문에 구조를 정한 부분이 있으면 그 이유를 짧게 설명해 주세요.
```

**English**

```text
Write Rust code for the following.

[Goal]
<2-3 lines>

[Environment]
- Rust edition 2024, stable toolchain (rustc 1.9x)
- Allowed crates: <tokio, serde, serde_json, reqwest (rustls-tls), ...>. Do not add other crates without asking first.
- Layout: <single src/main.rs | lib.rs + main.rs>

[Error handling]
- Library code: define an error enum with thiserror.
- Application code: anyhow::Result with .context().
- No unwrap()/expect()/panic! in library code (tests are fine).

[Quality bar]
- Take &str / &[T] in parameters where possible; no gratuitous clone().
- No unsafe. Use Rc<RefCell<>> / Arc<Mutex<>> only when truly needed, with a comment explaining why.
- Must pass: cargo fmt, cargo clippy --all-targets -- -D warnings, cargo test.
- Include #[cfg(test)] unit tests covering both success and error paths.

[Explain]
- Briefly explain any design decision driven by ownership, borrowing or lifetimes.
```

### 4.2 기존 코드 수정 요청

```text
아래 함수를 수정해 주세요. (기능 변경 없이 / 기능 추가: 〈내용〉)

〈코드〉

조건:
- 공개 함수 시그니처는 바꾸지 마세요. 꼭 바꿔야 한다면 먼저 이유를 설명하고 제 승인을 받으세요.
- 최소한의 diff 로 수정해 주세요. 관련 없는 리팩터링 금지.
- 수정 후 cargo clippy --all-targets -- -D warnings 와 cargo test 를 실행해서 결과를 알려 주세요.
- 빌림 검사기(borrow checker) 에러를 해결할 때는 어떤 규칙 때문에 에러가 났는지,
  왜 이 해결책을 골랐는지(clone 으로 덮은 것인지, 구조를 바꾼 것인지) 설명해 주세요.
```

```text
Modify the function below. (No behavior change / Add feature: <details>)

<code>

Constraints:
- Do not change public signatures. If you must, explain why and ask first.
- Keep the diff minimal; no unrelated refactoring.
- After editing, run cargo clippy --all-targets -- -D warnings and cargo test, and report the results.
- When fixing a borrow-checker error, explain which rule triggered it and why you chose this fix
  (papering over with clone() vs. restructuring).
```

### 4.3 코드 리뷰 요청

```text
아래 Rust 코드를 리뷰해 주세요. 초보 Rust 개발자(C++/Python 경험자)가 이해할 수 있게 설명해 주세요.

보고 싶은 것:
1. 소유권/빌림 관점의 문제 (불필요한 clone, 과도한 소유권 이동, &String/&Vec 매개변수)
2. 에러 처리 (unwrap/expect, 삼켜진 에러, 에러 타입 선택)
3. Rust 관용구에서 벗어난 부분 (인덱스 루프, 중첩 if, 수동 변환 등)과 개선안
4. 잠재적 panic 지점 (인덱싱, 나눗셈, 슬라이싱)
5. async 사용 시 블로킹/락 보유 문제

형식: 심각도(높음/중간/낮음) 순으로 표를 만들어 주세요. 코드는 아직 고치지 말고 설명만 해 주세요.
```

### 4.4 학습용으로 쓸 때 (정답 대신 힌트)

```text
저는 Rust 를 배우는 중이고 C++20/Python 경험이 있습니다.
아래는 제가 쓴 코드와 컴파일러 에러(전체)입니다.

〈코드〉
〈에러 메시지 전체〉

- 정답 코드를 바로 주지 마세요.
- 이 에러가 어떤 규칙(소유권/빌림/라이프타임/트레이트) 때문인지 C++ 또는 Python 과 비교해서 설명해 주세요.
- 고칠 수 있는 방법을 2가지 제시하고 장단점을 알려 주세요. 선택은 제가 합니다.
```

> 💡 **프롬프트 요령**: "에러 고쳐 줘"보다 **"왜 에러가 나는지 설명해 줘"**가 훨씬 많이 남습니다. 그리고 프로젝트 루트에 규칙을 파일(`CLAUDE.md` 같은 AI 에이전트용 지침 파일)로 적어 두면 매번 같은 조건을 반복해서 쓰지 않아도 됩니다. 예: "edition 2024, clippy `-D warnings` 통과, 라이브러리에서 unwrap 금지".

---

## 5. 피드백 루프: compile → clippy → test

AI와 일할 때의 기본 리듬입니다. 각 단계는 앞 단계보다 느리지만 더 깊은 것을 검사해요.

```text
┌────────────────────────────────────────────────────────┐
│ 1. cargo check          (수 초)  문법, 타입, 소유권/빌림   │
│ 2. cargo clippy         (수 초)  관용구, 흔한 실수, 성능   │
│ 3. cargo test           (수 초~) 동작이 요구사항과 맞는지  │
│ 4. cargo fmt            (즉시)   스타일 통일              │
└────────────────────────────────────────────────────────┘
        실패하면 → 에러 전문을 AI에게 (또는 스스로) 읽고 1번으로
```

```bash
cargo check                                # 빠른 컴파일 검사 (실행 파일 안 만듦)
cargo clippy --all-targets -- -D warnings  # 경고를 에러로 취급
cargo test                                 # 단위/통합/문서 테스트
cargo fmt                                  # 포맷 정리
```

### 5.1 AI에게 이 루프를 돌리게 하기

Claude Code 같은 에이전트는 터미널에서 직접 이 명령을 실행할 수 있습니다. 요청에 이렇게 적으세요.

```text
수정할 때마다 cargo check 를 돌려서 에러를 스스로 고쳐 주세요.
끝났다고 말하기 전에 cargo clippy --all-targets -- -D warnings 와 cargo test 를 실행하고 결과를 그대로 보여 주세요.
테스트를 통과시키려고 테스트 코드를 약하게 고치거나 #[allow(...)] 를 추가하지 마세요.
```

마지막 줄이 중요합니다. 에이전트가 **게이트를 통과하기 위해 게이트를 약하게 만드는** 경우가 실제로 있어요(테스트 삭제, `#[allow]` 추가, `unwrap()`으로 덮기).

### 5.2 왜 Rust의 컴파일러가 바이브 코딩을 더 안전하게 만들까요

1. **피드백이 빠르고 구체적**입니다. 에러 메시지에 위치, 원인, 수정 제안이 들어 있어 AI가 스스로 고치기 쉬워요.
2. **"컴파일되면 대부분 동작한다"**는 경험칙이 있습니다. 메모리/스레드 오류가 컴파일 단계에서 제거되기 때문에 리뷰어가 볼 범위가 줄어듭니다.
3. **의도가 타입에 드러납니다.** `Option`, `Result`, `&mut`, `Send` 같은 표시 덕분에 코드를 읽는 사람이 "무엇이 실패할 수 있고, 무엇이 바뀌고, 무엇이 스레드를 넘나드는지" 시그니처만 보고 알 수 있습니다.
4. **리팩터링이 안전**합니다. 열거형에 variant를 추가하면 처리하지 않은 모든 `match`가 에러가 됩니다.

> ⚠️ 한계: 컴파일러는 **논리 오류**(잘못된 공식, 요구사항 오해), **성능 문제**(과한 clone), **설계 문제**(과한 추상화)를 잡지 못합니다. 이 부분이 사람(과 테스트)의 몫입니다.

### 5.3 clippy가 잡아 주는 것 (실제 확인)

`cargo clippy -- -D warnings`는 기본 규칙만으로도 다음을 잡습니다. (아래는 모두 컴파일은 되지만 clippy가 경고하는 코드입니다.)

```rust
fn total(v: &Vec<i32>) -> i32 {
    // ptr_arg: &Vec<i32> 대신 &[i32]
    let mut sum = 0;
    for i in 0..v.len() {
        // needless_range_loop: 인덱스 루프 대신 이터레이터
        sum += v[i];
    }
    sum
}

fn main() {
    let x = 5;
    let y = x.clone(); // clone_on_copy: Copy 타입을 clone
    if y == 5 {
        println!("{}", total(&vec![1, 2, 3]));
    }
}
```

반면 `unwrap()` 사용 금지는 기본 규칙이 **아닙니다.** 엄격히 하고 싶다면 `cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used`처럼 명시하거나, 크레이트 맨 위에 `#![warn(clippy::unwrap_used)]`를 적습니다.

---

## 6. AI가 자주 하는 Rust 실수 (before / after)

### 6.1 라이브러리 코드에서 `unwrap()`

**Before** (컴파일은 되지만 잘못된 입력에 프로그램이 죽습니다)

```rust
fn parse_port(s: &str) -> u16 {
    s.trim().parse().unwrap() // "abc" 가 들어오면 panic
}

fn main() {
    println!("{}", parse_port(" 8080 "));
}
```

**After** (실패를 타입으로 알립니다)

```rust
use std::num::ParseIntError;

fn parse_port(s: &str) -> Result<u16, ParseIntError> {
    s.trim().parse()
}

fn main() {
    println!("{:?}", parse_port(" 8080 "));
    println!("{:?}", parse_port("abc").is_err());
}
```

### 6.2 `clone()`으로 빌림 에러 덮기 / 과도한 소유권 이동

**Before** (호출마다 벡터 전체를 복제)

```rust
fn total(v: Vec<i32>) -> i32 {
    v.iter().sum()
}

fn main() {
    let data = vec![1, 2, 3];
    let a = total(data.clone()); // 복제
    let b = total(data.clone()); // 또 복제
    println!("{a} {b}");
}
```

**After** (빌려서 읽기만)

```rust
fn total(v: &[i32]) -> i32 {
    v.iter().sum()
}

fn main() {
    let data = vec![1, 2, 3];
    let a = total(&data);
    let b = total(&data);
    println!("{a} {b}");
}
```

### 6.3 `&String` / `&Vec<T>` 매개변수

**Before**

```rust
fn shout(s: &String) -> String {
    s.to_uppercase()
}

fn main() {
    let owned = String::from("hi");
    println!("{}", shout(&owned));
    // shout("literal") 은 컴파일 에러: 리터럴(&str)을 &String 으로 못 넘김
}
```

**After**

```rust
fn shout(s: &str) -> String {
    s.to_uppercase()
}

fn main() {
    let owned = String::from("hi");
    println!("{} {}", shout(&owned), shout("literal")); // 둘 다 가능
}
```

### 6.4 `Rc<RefCell<T>>`로 설계 문제를 덮기

**Before** (단일 소유자인데 공유 가변 상태로 포장)

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Counter {
    n: Rc<RefCell<i32>>,
}

impl Counter {
    fn inc(&self) {
        *self.n.borrow_mut() += 1; // 이중으로 빌리면 런타임에 panic
    }
}

fn main() {
    let c = Counter { n: Rc::new(RefCell::new(0)) };
    c.inc();
    c.inc();
    println!("{}", c.n.borrow());
}
```

**After** (그냥 `&mut self`)

```rust
struct Counter {
    n: i32,
}

impl Counter {
    fn inc(&mut self) {
        self.n += 1; // 빌림 규칙을 컴파일 시점에 검사
    }
}

fn main() {
    let mut c = Counter { n: 0 };
    c.inc();
    c.inc();
    println!("{}", c.n);
}
```

### 6.5 불필요한 `'static` 요구

**Before** (지역 `String`을 넘기면 에러)

```rust,compile_fail
fn log(msg: &'static str) {
    println!("{msg}");
}

fn main() {
    let s = String::from("dynamic");
    log(&s); // `s` 가 'static 이 아니라서 에러 (E0597)
}
```

**After**

```rust
fn log(msg: &str) {
    println!("{msg}");
}

fn main() {
    let s = String::from("dynamic");
    log(&s);
    log("literal");
}
```

### 6.6 `async` 안에서 블로킹

**Before** (`std::thread::sleep`은 런타임 스레드를 통째로 멈춥니다. 컴파일은 되어서 더 위험해요)

```rust
use std::time::Duration;

async fn slow() {
    std::thread::sleep(Duration::from_millis(10)); // 다른 태스크도 같이 멈춤
}

#[tokio::main]
async fn main() {
    slow().await;
    println!("done");
}
```

**After**

```rust
use std::time::Duration;

async fn slow() {
    tokio::time::sleep(Duration::from_millis(10)).await; // 양보하며 대기
}

#[tokio::main]
async fn main() {
    slow().await;
    println!("done");
}
```

### 6.7 `MutexGuard`를 쥔 채 `.await`

**Before** (컴파일 에러: `future cannot be sent between threads safely`)

```rust,compile_fail
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(0));
    let d = Arc::clone(&data);
    let h = tokio::spawn(async move {
        let mut g = d.lock().unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await; // 락을 쥔 채 await
        *g += 1;
    });
    h.await.unwrap();
}
```

실제 에러 (핵심 부분):

```text
error: future cannot be sent between threads safely
  = help: ... the trait `Send` is not implemented for `std::sync::MutexGuard<'_, i32>`
note: future is not `Send` as this value is used across an await
    |
  9 |         let mut g = d.lock().unwrap();
    |             ----- has type `std::sync::MutexGuard<'_, i32>` which is not `Send`
 10 |         tokio::time::sleep(Duration::from_millis(10)).await;
    |                                                       ^^^^^ await occurs here, with `mut g` maybe used later
```

**After** (락은 짧게, `.await` 밖에서)

```rust
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(0));
    let d = Arc::clone(&data);
    let h = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        *d.lock().unwrap() += 1; // 락은 이 한 줄에서만
    });
    h.await.unwrap();
    println!("{}", *data.lock().unwrap());
}
```

### 6.8 라이브러리에서 `Box<dyn Error>` / `String` 에러

**Before** (호출자가 에러 종류를 구분할 수 없음)

```rust
use std::error::Error;

fn load(s: &str) -> Result<i32, Box<dyn Error>> {
    if s.is_empty() {
        return Err("empty input".into()); // 그냥 문자열
    }
    Ok(s.parse::<i32>()?)
}

fn main() {
    println!("{:?}", load("12").is_ok());
    println!("{}", load("").unwrap_err());
}
```

**After** (`thiserror`로 종류를 구분)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum LoadError {
    #[error("empty input")]
    Empty,
    #[error("not a number: {0}")]
    BadNumber(#[from] std::num::ParseIntError),
}

fn load(s: &str) -> Result<i32, LoadError> {
    if s.is_empty() {
        return Err(LoadError::Empty);
    }
    Ok(s.parse::<i32>()?)
}

fn main() {
    match load("x") {
        Err(LoadError::BadNumber(e)) => println!("bad: {e}"),
        other => println!("{:?}", other.is_ok()),
    }
    println!("{}", load("").unwrap_err());
}
```

앱의 `main` 근처에서는 `anyhow::Result`와 `.context("...")`로 간편하게 쓰고, **재사용되는 라이브러리 경계**에서는 위처럼 구체적인 enum을 쓰는 것이 일반적인 분업입니다.

### 6.9 에러를 조용히 삼키기

**Before**

```rust
use std::fs;

fn main() {
    let text = fs::read_to_string("no_such_file.txt").unwrap_or_default(); // 에러가 사라짐
    let _ = fs::write("/proc/nope/out.txt", &text); // 실패해도 모름
    println!("len {}", text.len());
}
```

**After**

```rust
use std::fs;

fn main() -> Result<(), std::io::Error> {
    let text = match fs::read_to_string("no_such_file.txt") {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(), // 의도한 경우만 기본값
        Err(e) => return Err(e),
    };
    println!("len {}", text.len());
    Ok(())
}
```

### 6.10 `_ =>` 로 모든 variant를 뭉개기

**Before**

```rust
enum Event {
    Start,
    Stop,
    Pause, // 나중에 추가됨
}

fn name(e: &Event) -> &'static str {
    match e {
        Event::Start => "start",
        Event::Stop => "stop",
        _ => "other", // Pause 도 조용히 "other" 가 됨
    }
}

fn main() {
    println!("{}", name(&Event::Pause));
}
```

**After** (새 variant를 추가하면 컴파일러가 알려 줌)

```rust
enum Event {
    Start,
    Stop,
    Pause,
}

fn name(e: &Event) -> &'static str {
    match e {
        Event::Start => "start",
        Event::Stop => "stop",
        Event::Pause => "pause",
    }
}

fn main() {
    println!("{}", name(&Event::Pause));
}
```

### 6.11 인덱스 루프

**Before**

```rust
fn main() {
    let v = vec![3, 1, 4, 1, 5];
    let mut evens = Vec::new();
    for i in 0..v.len() {
        if v[i] % 2 == 0 {
            evens.push(v[i] * 10);
        }
    }
    println!("{:?}", evens);
}
```

**After**

```rust
fn main() {
    let v = vec![3, 1, 4, 1, 5];
    let evens: Vec<i32> = v.iter().filter(|&&x| x % 2 == 0).map(|x| x * 10).collect();
    println!("{:?}", evens);
}
```

### 6.12 존재하지 않는 API를 만들어 내기 (환각)

AI는 가끔 **없는 메서드/크레이트 기능**을 자신 있게 씁니다. 컴파일러가 바로 알려 줘요.

```rust,compile_fail
fn main() {
    let mut v = vec![1, 2];
    v.push_all(&[3, 4]); // 그런 메서드는 없음 (E0599)
    println!("{:?}", v);
}
```

대응: 에러 전문과 함께 "이 메서드는 `Vec`에 없습니다. 표준 라이브러리 문서에서 맞는 메서드(`extend_from_slice`)를 찾아 주세요"처럼 **문서 확인**을 요청하세요. 크레이트 버전에 따라 API가 바뀌는 경우(예: `reqwest`, `tokio`, `serde`)도 흔하니 `Cargo.toml`의 버전을 함께 알려 주는 것이 좋습니다.

```rust
fn main() {
    let mut v = vec![1, 2];
    v.extend_from_slice(&[3, 4]);
    println!("{:?}", v);
}
```

### 6.13 "컴파일되게" 하려고 `unsafe`나 `unwrap` 추가하기

AI가 빌림 에러를 만나면 가끔 `unsafe { &mut *ptr }`나 `std::mem::transmute`로 우회하려 합니다. **거절하세요.** "unsafe 없이, 구조를 바꾸거나 소유권을 정리해서 해결해 주세요"라고 요청하면 대개 `&mut`의 범위를 줄이거나, 인덱스/ID로 참조를 대체하거나, 데이터를 두 구조체로 쪼개서 해결합니다.

---

## 7. "이게 보이면 이 챕터를 다시 보세요" 매핑표

| 코드/에러에서 이게 보이면 | 다시 볼 챕터 | 핵심 질문 |
|---|---|---|
| `cargo` 명령, `Cargo.toml`, `edition`, `clippy` 설정 | **ch01** Cargo와 첫 프로그램 | 이 명령/설정은 무엇을 하나? |
| `let mut`, shadowing, `as`, 오버플로 (`checked_*`/`wrapping_*`), `match` 식 | **ch02** 문법 기초 | 값이 조용히 잘리거나 panic 나지 않나? |
| E0382 (moved value), `.clone()` 남발, `Drop`, `move` | **ch03** 소유권 | 이 값의 주인은 누구인가? |
| `&`, `&mut`, E0499/E0502/E0505/E0506, `&str` vs `String`, 슬라이스 | **ch04** 참조와 빌림 | 누가 읽고 누가 바꾸나? 동시에는 안 됩니다. |
| `struct`/`impl`, `enum` + `match`, `Option`, `if let`, `let else` | **ch05** 구조체와 열거형 | 상태를 enum으로 표현할 수 있나? |
| `Result`, `?`, `unwrap`/`expect`, 커스텀 에러, `From` | **ch06** 에러 처리 | 실패가 타입에 드러나고 전파되는가? |
| `Vec`/`HashMap`, 클로저, `iter().map().collect()`, `entry` | **ch07** 컬렉션과 이터레이터 | 이터레이터가 값을 소비하는지, 빌리는지? |
| `trait`, `impl Trait`, `dyn Trait`, `Box<dyn ..>`, 제네릭 경계 `where` | **ch08** 트레이트와 제네릭 | 정적/동적 디스패치 중 무엇이 맞나? |
| `'a`, `'static`, E0106/E0597/E0716, 참조를 담은 struct | **ch09** 라이프타임 | 참조가 가리키는 원본은 얼마나 오래 사나? |
| `mod`, `pub`, `pub(crate)`, `lib.rs` + `main.rs`, `tests/`, workspace | **ch10** 모듈과 테스트 | 공개 범위가 적절한가? 테스트가 의미 있나? |
| `Box`, `Rc`, `Weak`, `RefCell`, `Rc<RefCell<..>>` | **ch11** 스마트 포인터 | 정말 공유 소유/내부 가변성이 필요한가? |
| `From`/`Into`/`TryFrom`, builder, newtype, clippy 경고, 인덱스 루프 | **ch12** Rust다운 코드 | 더 관용적인 표현이 있나? |
| `thread::spawn`, `Arc<Mutex<..>>`, `mpsc`, `Send`/`Sync`, E0373, `Rc` 에러(E0277) | **ch13** 동시성 | 무엇이 스레드 사이를 오가나? |
| `async`/`.await`, `tokio::spawn`, `select!`, `timeout`, 블로킹 호출, `MutexGuard` across await | **ch14** async와 Tokio | Future가 실행되고 있나? 런타임을 막고 있지 않나? |
| `#[derive(Serialize, Deserialize)]`, `#[serde(..)]`, `serde_json::Value` | **ch15** Serde와 JSON | 필드명/태그/기본값이 실제 JSON과 일치하나? |
| `reqwest`, 상태 코드, 타임아웃, 재시도, `thiserror` vs `anyhow`, `.context()` | **ch16** HTTP와 에러 처리 | 실패 경로(네트워크/4xx/5xx/파싱)가 전부 처리되나? |
| 에이전트 루프, `stop_reason`, `tool_use`/`tool_result`, 최대 반복 횟수, API 키 | **ch17** 캡스톤 | 루프가 멈추는 조건과 비용 상한이 있나? |

> 에러 코드별로 찾아보려면 [03. 컴파일러 에러 읽는 법](03_reading_compiler_errors.md), 문법 대응은 [01. 치트시트](01_cpp_python_rust_cheatsheet.md), 환경 설정은 [00. 환경 준비](00_environment_setup.md)를 보세요.
