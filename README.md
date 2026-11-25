# 🦀 rust_learn — C++20 · Python 개발자를 위한 Rust 커리큘럼

> **목표:** Rust로 **AI 에이전트를 바이브 코딩**하려면 최소한 알아야 하는 것을 익힙니다.
> 핵심 언어 특성, Rust다운 코딩 스타일, 프로젝트 구조를 직접 손으로 써 보면서 배웁니다.

AI가 Rust 코드를 대신 써 주더라도, 사람은 그 코드를 **읽고 판단하고 방향을 잡아야** 합니다.
이 커리큘럼은 그 판단력을 기르기 위해 만들었습니다. 각 챕터는 **이론 → 실습**으로 이루어져 있습니다.
실습은 **이미 완성된 Python·C++ 프로그램과 똑같은 결과를 내는 Rust 프로그램을 직접 만드는 것**입니다.

---

## 📌 이런 분을 위한 과정입니다

| 이미 아는 것 | 이 과정에서 얻는 것 |
|---|---|
| Modern C++20 (RAII, move, 템플릿, 스마트 포인터) | C++ 지식과 Rust 개념의 **정확한 대응과 차이** (예: Rust의 move는 왜 "원본을 못 쓰게" 만드나) |
| Python, MATLAB | 익숙한 코드와 나란히 놓고 보며 Rust 문법과 스타일을 익힘 |
| 바이브 코딩 경험 | AI가 생성한 Rust 코드를 **리뷰하고 고치는 눈**, 컴파일러·clippy·테스트를 활용한 피드백 루프 |

> 💡 **쉬운 설명 — 바이브 코딩(vibe coding)이란?**
> 사람이 "무엇을 만들지"를 말로 설명하면 AI가 코드를 생성하고, 사람은 결과를 보면서 방향을 잡아 주는 개발 방식입니다.
> Rust는 컴파일러가 아주 깐깐해서, AI가 만든 실수를 **실행 전에** 많이 잡아 줍니다. 그래서 바이브 코딩과 궁합이 좋은 언어입니다.
> 단, 컴파일러가 하는 말을 사람이 이해해야 그 장점을 살릴 수 있습니다.

---

## 🚀 빠른 시작

### 1) 환경 열기 — 추천: GitHub Codespaces (브라우저만 있으면 됨)
1. 이 저장소의 GitHub 페이지에서 **Code → Codespaces → Create codespace on main**을 누릅니다.
2. 몇 분 기다리면 브라우저 안에 VS Code가 열리고, Rust·C++·Python 도구가 자동으로 설치됩니다.
3. 터미널에서 아래 명령을 입력합니다.

```bash
cargo --version
```

```bash
scripts/compare.sh ch01 --solution
```

로컬 PC(Windows/WSL2, macOS, Linux)에서 하는 방법과 Rust Playground 활용법은 → [docs/00_environment_setup.md](docs/00_environment_setup.md)를 보세요.

> ❓ **Claude Code 앱에서 바로 실습하면 안 되나요?**
> Rust 학습의 핵심은 **직접 타이핑하면서 컴파일러 에러를 그 자리에서 보는 경험**입니다.
> Claude Code 앱(클라우드 세션)에서는 직접 코드를 편집하고 rust-analyzer의 실시간 힌트를 보기가 어렵습니다.
> 그래서 **코드는 Codespaces(브라우저)나 로컬에서 직접 쓰고**, Claude Code 앱은 아래처럼 쓰는 것을 권장합니다.
> - 질문하기
> - 컴파일 에러의 *이유* 물어보기
> - 다 푼 코드 리뷰 받기

### 2) 한 챕터를 공부하는 순서

```
① chapters/chNN_xxx/README.md      이론을 읽고, 코드 조각은 Playground에서 돌려 봅니다
② python/main.py, cpp/main.cpp     완성된 레퍼런스 프로그램을 읽고 실행해 봅니다 (무엇을 만들지 파악)
③ rust/src/main.rs                 TODO(1)부터 순서대로 채웁니다 (🔬 실험 주석도 꼭 풀어 보기)
④ cargo test -p chNN_xxx           테스트가 모두 통과하는지 확인합니다
⑤ scripts/compare.sh chNN          Python·C++·Rust 출력이 한 글자도 다르지 않은지 확인합니다
⑥ rust/examples/solution.rs        다 풀고 나서 정답과 비교합니다 (먼저 보면 효과가 반감돼요!)
```

```bash
cd chapters/ch03_ownership
```

```bash
cargo run -p ch03_ownership
```

```bash
cargo test -p ch03_ownership
```

```bash
../../scripts/compare.sh ch03
```

> 💡 **쉬운 설명 — `todo!()`가 뭔가요?**
> 실습 파일의 빈칸에는 `todo!()`가 들어 있습니다. "아직 구현 안 됨"이라는 표시입니다.
> 컴파일은 되지만, 실행이 그 줄에 도달하면 멈추면서(panic) 알려 줍니다.
> 그래서 처음에 테스트가 실패하는 것은 **정상**입니다. 하나씩 채우면서 초록색으로 바꿔 가면 됩니다.

---

## 🗺️ 커리큘럼 (17챕터)

### Part 1. 기초 — Rust의 뼈대

| # | 챕터 | 핵심 개념 | 실습 프로젝트 |
|---|---|---|---|
| 01 | [Cargo와 첫 프로그램](chapters/ch01_cargo_basics/) | rustup·cargo·crate, 빌드 프로필, rustfmt·clippy, 매크로(`!`) | 온도 변환표 |
| 02 | [기본 문법](chapters/ch02_syntax_basics/) | `let`/`mut`, 섀도잉, 정수 오버플로, **표현식 중심 문법**(`if`/`match`/`loop`가 값을 반환) | 콜라츠 수열 통계 |
| 03 | [소유권과 이동](chapters/ch03_ownership/) ⭐ | 소유권 3원칙, **기본이 move**, Copy vs Clone, Drop/RAII | 작업 파이프라인 |
| 04 | [빌림과 슬라이스](chapters/ch04_borrowing/) ⭐ | `&T`/`&mut T`, "공유 XOR 수정" 규칙, `String` vs `&str`, UTF-8 | 텍스트 분석기 |
| 05 | [구조체·열거형·패턴 매칭](chapters/ch05_structs_enums/) | `impl`, 데이터를 품은 enum, `Option`, 빠짐없는 `match` | 자판기 상태 머신 |
| 06 | [에러 처리](chapters/ch06_error_handling/) | `Result`, `?` 연산자, 커스텀 에러, 예외가 없는 세계 | 설정 파일 파서 |
| 07 | [컬렉션·클로저·이터레이터](chapters/ch07_collections_iterators/) | `Vec`/`HashMap`, entry API, 클로저, 이터레이터 체인 | 성적 데이터 분석 |

### Part 2. Rust다운 설계 — 구조와 스타일

| # | 챕터 | 핵심 개념 | 실습 프로젝트 |
|---|---|---|---|
| 08 | [트레이트와 제네릭](chapters/ch08_traits_generics/) | trait, 제네릭 바운드, `impl Trait` vs `dyn Trait`, 정적/동적 디스패치 | 에이전트 도구 레지스트리 (동기) |
| 09 | [라이프타임](chapters/ch09_lifetimes/) ⭐ | `'a`, 생략 규칙, 참조를 품은 구조체, `'static` | 제로카피 로그 파서 |
| 10 | [모듈·크레이트·테스트](chapters/ch10_modules_testing/) | 모듈 트리, `pub`, lib/bin 분리, 워크스페이스, 단위/통합/문서 테스트 | 가계부 라이브러리 |
| 11 | [스마트 포인터](chapters/ch11_smart_pointers/) | `Box`/`Rc`/`Weak`/`RefCell`, 내부 가변성, 그리고 **쓰지 말아야 할 때** | 수식 트리 + 대화 분기 트리 |
| 12 | [Rust다운 코드 (관용구)](chapters/ch12_idiomatic_rust/) ⭐ | `From`/`Into`, builder, newtype, 콤비네이터, clippy | "C++ 스타일 Rust" 리팩터링 |

### Part 3. AI 에이전트를 향해 — 실전 생태계

| # | 챕터 | 핵심 개념 | 실습 프로젝트 |
|---|---|---|---|
| 13 | [동시성](chapters/ch13_concurrency/) | 스레드, `Send`/`Sync`, `Arc<Mutex<T>>`, 채널, 데이터 레이스가 컴파일 에러가 되는 원리 | 워커 풀 |
| 14 | [async/await와 Tokio](chapters/ch14_async_tokio/) | Future(게으른 실행), 런타임, `spawn`/`JoinSet`/`timeout`, 재시도 | 가짜 LLM API 동시 호출 |
| 15 | [Serde로 JSON 다루기](chapters/ch15_serde_json/) | derive, `#[serde(tag = "type")]`, 타입 있는 JSON vs `Value` | LLM 대화 로그 변환기 |
| 16 | [HTTP 클라이언트와 실전 에러 처리](chapters/ch16_http_errors/) | `reqwest`, 상태 코드, 타임아웃·재시도, `thiserror` vs `anyhow` | Todo REST 클라이언트 (로컬 목 서버) |
| 17 | [캡스톤: 미니 AI 에이전트](chapters/ch17_capstone_agent/) 🏁 | 에이전트 루프, Claude Messages API, tool use, 비동기 Tool trait | 도구를 쓰는 에이전트 (목 서버 / 실제 Claude API) |

⭐ = Rust의 핵심이자 C++ 개발자가 가장 많이 헷갈리는 챕터입니다. 시간을 넉넉히 쓰세요.

> 💡 **쉬운 설명 — 목(mock) 서버란?**
> 진짜 서버인 척하는 가짜 서버입니다. 16~17장은 인터넷이나 API 키 없이도 실습할 수 있도록,
> 내 컴퓨터 안에서 실행되는 가짜 API 서버(`tools/mock_server.py`)를 상대로 통신합니다.
> `compare.sh`가 이 서버를 자동으로 켜고 끕니다. 17장은 원하면 진짜 Claude API로도 실행할 수 있습니다.

### 📚 부록

| 문서 | 내용 |
|---|---|
| [00. 실습 환경 준비](docs/00_environment_setup.md) | Codespaces / 로컬(WSL2·macOS·Linux) / Playground, Claude Code 앱과 함께 쓰는 법 |
| [01. C++ · Python · Rust 대응표](docs/01_cpp_python_rust_cheatsheet.md) | 주제별로 세 언어를 나란히 비교한 치트시트 |
| [02. Rust 바이브 코딩 가이드](docs/02_vibe_coding_with_rust.md) | AI가 쓴 Rust 코드 읽는 법, 리뷰 체크리스트, 프롬프트 템플릿 |
| [03. 컴파일러 에러 읽는 법](docs/03_reading_compiler_errors.md) | 초보자가 자주 만나는 rustc 에러 사전 |

---

## 🗂️ 저장소 구조

```
rust_learn/
├── README.md                  ← 지금 보고 있는 파일 (커리큘럼 개요)
├── Cargo.toml                 ← 워크스페이스: 모든 챕터의 Rust 실습을 하나로 묶음
├── chapters/
│   └── chNN_주제/
│       ├── README.md          ← 이론 + 실습 안내
│       ├── python/main.py     ← 완성된 Python 레퍼런스
│       ├── cpp/main.cpp       ← 완성된 C++20 레퍼런스
│       ├── data/              ← (필요한 챕터만) 입력 데이터
│       └── rust/
│           ├── src/main.rs        ← ✍️ 여러분이 채울 실습 파일 (TODO + 주석 가이드)
│           └── examples/solution.rs ← 정답 (다 풀고 나서 보세요)
├── docs/                      ← 부록
├── scripts/
│   ├── compare.sh             ← Python·C++·Rust 출력 비교
│   ├── setup.sh               ← 로컬 환경 설치 도우미
│   └── verify_all.sh          ← 모든 챕터 정답 검증 (관리용)
└── tools/mock_server.py       ← 16~17장용 로컬 목 서버
```

> 💡 **쉬운 설명 — 워크스페이스(workspace)란?**
> 여러 Rust 프로젝트(크레이트)를 한 지붕 아래 묶어 관리하는 기능입니다.
> 빌드 결과물 폴더(`target/`)와 라이브러리 버전 목록(`Cargo.lock`)을 함께 씁니다.
> 그래서 어느 폴더에 있든 `cargo run -p 챕터이름`으로 원하는 챕터를 실행할 수 있습니다.
> 10장에서 자세히 다룹니다.

---

## 🧭 학습 팁

1. **정답보다 에러 메시지를 먼저 보세요.** Rust 컴파일러의 에러 메시지는 "선생님"에 가깝습니다. `help:`와 `note:` 줄을 끝까지 읽는 습관을 들이세요.
2. **🔬 실험 주석은 꼭 풀어 보세요.** 일부러 에러를 내 보는 것이 "왜 Rust는 이렇게 하지?"를 이해하는 가장 빠른 길입니다.
3. **AI에게는 "고쳐 줘" 대신 "왜?"를 물어보세요.** 학습하는 동안에는 답을 받는 것보다 이유를 듣는 쪽이 훨씬 남습니다.
4. **clippy를 친구로.** `cargo clippy -p 챕터이름`은 "Rust다운 코드"를 알려 주는 무료 코드 리뷰어입니다.
5. **C++ 습관을 의심하세요.** 특히 "일단 복사", "포인터로 공유", "null 체크", "예외 던지기"는 Rust에서 다른 방식으로 풉니다.

## ✅ 진도 체크

- [ ] Ch01 Cargo와 첫 프로그램
- [ ] Ch02 기본 문법
- [ ] Ch03 소유권과 이동
- [ ] Ch04 빌림과 슬라이스
- [ ] Ch05 구조체·열거형·패턴 매칭
- [ ] Ch06 에러 처리
- [ ] Ch07 컬렉션·클로저·이터레이터
- [ ] Ch08 트레이트와 제네릭
- [ ] Ch09 라이프타임
- [ ] Ch10 모듈·크레이트·테스트
- [ ] Ch11 스마트 포인터
- [ ] Ch12 Rust다운 코드
- [ ] Ch13 동시성
- [ ] Ch14 async/await와 Tokio
- [ ] Ch15 Serde로 JSON 다루기
- [ ] Ch16 HTTP 클라이언트와 실전 에러 처리
- [ ] Ch17 캡스톤: 미니 AI 에이전트

---

참고 자료: [The Rust Programming Language](https://doc.rust-lang.org/book/) · [Rust by Example](https://doc.rust-lang.org/rust-by-example/) · [Rust Playground](https://play.rust-lang.org/) · [Tokio 튜토리얼](https://tokio.rs/tokio/tutorial) · [Serde](https://serde.rs/)
