# 00. 실습 환경 준비

> 이 저장소의 실습은 챕터마다 **Python → C++ → Rust** 세 가지 구현을 실행해서 출력이 똑같은지 비교합니다. 그래서 Rust만이 아니라 `g++`(C++20), `python3`, 그리고 몇몇 챕터에서는 `libcurl`, `nlohmann-json` 같은 도구도 필요해요. 아래 세 가지 방법 중 하나를 고르세요.

## 목차

1. [어떤 방법을 고를까요](#1-어떤-방법을-고를까요)
2. [방법 A. GitHub Codespaces (브라우저) — 추천](#2-방법-a-github-codespaces-브라우저--추천)
3. [방법 B. 로컬 PC](#3-방법-b-로컬-pc)
4. [방법 C. Rust Playground](#4-방법-c-rust-playground)
5. [Claude Code 앱은 어떻게 같이 쓰나요?](#5-claude-code-앱은-어떻게-같이-쓰나요)
6. [문제 해결 (Troubleshooting)](#6-문제-해결-troubleshooting)
7. [환경이 준비됐는지 확인하기](#7-환경이-준비됐는지-확인하기)

---

## 1. 어떤 방법을 고를까요

| | A. Codespaces | B. 로컬 PC | C. Playground |
|---|---|---|---|
| 설치 | **없음** (브라우저) | 직접 설치 | 없음 |
| 모든 챕터 실습 | 가능 | 가능 | **불가** (이론 조각만) |
| C++/Python 비교 | 가능 (자동 설치) | 가능 | 불가 |
| rust-analyzer(실시간 에러/타입 표시) | 가능 | 가능 | 제한적 |
| 비용 | 월 무료 할당량 내 무료 | 무료 | 무료 |
| 인터넷 필요 | 필요 | 처음 설치 때만 | 필요 |
| 추천 대상 | **처음 시작하는 모든 분** | 오래 쓸 개인 PC가 있는 분 | 문법 한 줄만 확인할 때 |

처음이라면 **A**로 시작하세요. 환경 문제로 막히는 시간이 거의 없습니다.

---

## 2. 방법 A. GitHub Codespaces (브라우저) — 추천

Codespaces는 GitHub가 클라우드에 빌려 주는 개발용 컴퓨터입니다. 브라우저에서 VS Code가 열리고, 이 저장소의 `.devcontainer/devcontainer.json` 설정대로 환경이 **자동으로** 만들어져요.

> 💡 **쉬운 설명**: "내 PC에 아무것도 설치하지 않고, 인터넷 창 안에서 이미 세팅이 끝난 컴퓨터를 하나 빌려 쓰는 것"입니다. 다 쓰고 나면 껐다가 나중에 다시 켤 수 있어요.

### 2.1 만들기 (단계별)

1. GitHub에 로그인합니다. (계정이 없다면 무료로 만들 수 있어요.)
2. 이 저장소 페이지로 갑니다. 본인의 계정에 **Fork**(복제)했다면 내 Fork 페이지에서 하세요. 그래야 풀이를 **내 저장소에 저장(push)** 할 수 있습니다. (원본 저장소에 쓰기 권한이 없다면 Fork가 필요해요.)
3. 초록색 **`<> Code`** 버튼을 누릅니다.
4. **Codespaces** 탭을 고릅니다.
5. **Create codespace on main** 을 누릅니다.
6. 처음에는 환경을 만드느라 **몇 분** 걸립니다. 화면에 "Setting up your codespace" 로그가 보이고, 끝나면 브라우저에 VS Code가 열립니다.

### 2.2 자동으로 설치되는 것

`.devcontainer/devcontainer.json`이 다음을 준비해 줍니다.

| 항목 | 내용 |
|---|---|
| 기본 이미지 | Ubuntu 24.04 (dev container 기본 이미지) |
| Rust | `rustup` 기반 Rust 툴체인 + `rustfmt` + `clippy` |
| 추가 설치 스크립트 | 생성 직후 `bash scripts/setup.sh` 자동 실행 → `g++`(C++ 컴파일러), `nlohmann-json3-dev`, `libcurl4-openssl-dev`, `python3`/`pip`, Python `anthropic` 패키지 |
| VS Code 확장 | `rust-analyzer`(Rust 언어 지원), CodeLLDB(디버거), C/C++, Python, TOML 지원, Error Lens(에러를 줄 옆에 바로 표시) |
| 설정 | `rust-analyzer`가 저장할 때 `clippy`로 검사, 저장 시 자동 포맷 |

첫 생성 때 `postCreateCommand`(설치 스크립트)가 돌고 있으면 터미널에 설치 로그가 흐릅니다. 끝날 때까지 기다리세요. 끝에 `완료! 이제 scripts/compare.sh ch01 처럼 실행해 보세요.`가 보이면 준비 완료입니다.

### 2.3 첫 명령어

화면 아래쪽 **터미널**(없으면 메뉴 `Terminal > New Terminal`, 단축키 ``Ctrl+` ``)에서 실행합니다.

```bash
cargo --version              # 예: cargo 1.97.0 (...)
rustc --version
scripts/compare.sh ch01      # 1장: Python / C++ / Rust 출력 비교
```

처음 `scripts/compare.sh ch01`을 실행하면 Rust 구현이 아직 `todo!()`라서 이런 안내가 나오는 것이 **정상**입니다.

```text
  아직 TODO가 남아 있어요: ...
  rust/src/main.rs 의 TODO 를 순서대로 채워 보세요.
```

정답 구현과 비교해서 환경이 제대로인지 확인하려면:

```bash
scripts/compare.sh ch01 --solution
```

`✅ ... 출력이 똑같아요`가 보이면 Python/C++/Rust가 모두 정상 동작하는 것입니다.

### 2.4 rust-analyzer 사용법

`.rs` 파일을 열면 rust-analyzer가 자동으로 프로젝트를 분석합니다. (처음에는 오른쪽 아래 상태 표시줄에 "loading…"이 한동안 보여요.)

- **인레이 힌트(inlay hints)**: 코드 옆에 회색 글씨로 추론된 타입이 표시됩니다. 예: `let x = 5;` 옆에 `: i32`, 클로저 매개변수의 타입, 메서드 체인 중간 타입. "컴파일러가 이걸 이렇게 이해했구나"를 바로 확인할 수 있어서 Rust 학습에 아주 유용해요. 힌트가 거슬리면 설정(`Ctrl+,`)에서 `rust-analyzer inlay hints`를 검색해 끌 수 있습니다.
- **에러 표시**: 빨간 물결 밑줄 = 에러, 노란 = 경고. 마우스를 올리면 `cargo check`와 같은 에러 메시지가 뜹니다. Error Lens 확장이 있으면 줄 끝에 한 줄로 바로 보여요.
- **Problems 패널**: 메뉴 `View > Problems` (단축키 `Ctrl+Shift+M`). 열린 파일 전체의 에러/경고 목록이 나오고 클릭하면 그 위치로 이동합니다. 에러 코드(`E0382` 등)는 [03 문서](03_reading_compiler_errors.md)에서 찾아보세요.
- **빠른 수정(Quick Fix)**: 에러가 난 곳에서 `Ctrl+.` 를 누르면 컴파일러/rust-analyzer가 제안하는 수정 목록이 나옵니다(예: `use` 추가, `mut` 추가).
- **정의로 이동 / 타입 보기**: 이름 위에서 `F12`(정의로 이동), 마우스 호버(타입과 문서).
- **실행/테스트 버튼**: `fn main` 위의 `▶ Run`, 각 `#[test]` 위의 `▶ Run Test`를 눌러 실행할 수 있어요.

> 처음 몇 번은 `todo!()`가 들어 있는 연습 파일에서 노란 경고(unused variable 등)가 많이 보일 수 있습니다. 각 연습 파일 맨 위의 `#![allow(...)]` 줄이 대부분을 숨겨 줍니다.

### 2.5 내 풀이 저장하기 (commit / push)

Codespaces는 임시 컴퓨터이므로 **중요한 풀이는 Git에 저장**해 두세요.

VS Code UI로 하는 방법:

1. 왼쪽 **Source Control** 아이콘(가지 모양) 클릭
2. 바뀐 파일 옆 `+` 로 스테이지(또는 맨 위 `+`로 전체)
3. 위쪽 메시지 입력칸에 `ch03 풀이` 같은 커밋 메시지 입력 후 **Commit**
4. **Sync Changes**(또는 `Push`) 클릭

터미널로 하는 방법:

```bash
git status
git add chapters/ch03_ownership/rust/src/main.rs
git commit -m "ch03: 소유권 풀이"
git push
```

> ⚠️ **본인 Fork에서 작업**했는지 확인하세요. 원본 저장소에 쓰기 권한이 없으면 `push`가 거부됩니다. 이때는 Fork를 만들고 그 Fork에서 Codespace를 새로 만들면 됩니다. 커밋 전에 `target/` 폴더는 올라가지 않도록 `.gitignore`에 들어 있어야 해요(이미 설정돼 있어야 정상입니다).

### 2.6 무료 사용량과 비용 관리

- 개인 GitHub 계정에는 **매월 무료 Codespaces 사용 할당량**(코어 시간과 저장 용량)이 제공됩니다. 정확한 숫자와 정책은 바뀔 수 있으니 **GitHub의 최신 안내(Billing / Codespaces 문서, 계정의 Settings > Billing)를 직접 확인**하세요.
- 사용량은 **켜져 있는 시간**에 비례합니다. 쓰지 않을 때는 꼭 멈추세요.
  - 왼쪽 아래 초록색 `Codespaces` 표시 > **Stop Current Codespace**
  - 또는 [github.com/codespaces](https://github.com/codespaces) 에서 각 codespace의 `...` 메뉴 > **Stop codespace**
  - 아무 활동이 없으면 일정 시간 뒤에 자동으로 멈추지만(기본 30분), 그 전까지는 시간이 쓰입니다.
- 다 쓴 codespace는 **삭제**하세요. 멈춘 것도 저장 공간을 차지합니다(`github.com/codespaces` > `...` > **Delete**). 삭제 전에 `git push`로 풀이를 저장했는지 확인하세요!
- 코어 수가 작은 머신(2-core)을 고르면 무료 시간이 더 오래 갑니다. 이 저장소의 실습에는 2-core로 충분해요.

---

## 3. 방법 B. 로컬 PC

로컬에서는 다음이 필요합니다: **Rust(rustup)**, **g++ (C++20 지원)**, **python3 (3.12 이상 권장)**, **libcurl 개발 헤더**, **nlohmann-json 헤더**, Python **anthropic** 패키지(ch17의 Python 예제에만 필요).

### 3.1 Windows — WSL2 + Ubuntu 권장

이 저장소의 C++ 예제(`g++`, `libcurl`, `nlohmann-json`)와 `scripts/*.sh`는 **Linux 환경**을 가정합니다. Windows에서는 WSL2(Windows 안의 Linux)를 쓰는 것이 가장 편해요.

1. **WSL2 + Ubuntu 설치** — 관리자 권한 PowerShell에서:

   ```powershell
   wsl --install -d Ubuntu
   ```

   설치 후 재부팅하고, Ubuntu를 처음 열어 사용자 이름/비밀번호를 만드세요.
2. **VS Code 설치** 후 확장 **WSL**(Remote - WSL)을 설치합니다.
3. Ubuntu 터미널에서 저장소를 **WSL의 홈 폴더 안**에 받습니다 (`/mnt/c/...` 아래는 느리니 피하세요).

   ```bash
   cd ~
   git clone <이 저장소 주소> rust_learn
   cd rust_learn
   code .          # VS Code가 WSL 모드로 열립니다
   ```
4. 설치 스크립트를 실행합니다.

   ```bash
   bash scripts/setup.sh
   ```

   `g++`, `nlohmann-json3-dev`, `libcurl4-openssl-dev`, `python3`, `pip`, `anthropic`, `rustup`(Rust), `rustfmt`/`clippy`를 설치합니다. 여러 번 실행해도 안전해요.
5. 새 터미널을 열거나 `source "$HOME/.cargo/env"`를 실행한 뒤 확인합니다.

   ```bash
   cargo --version
   scripts/compare.sh ch01
   ```
6. VS Code에서 확장 **rust-analyzer**를 설치하세요. (WSL에 설치하라는 버튼이 보이면 누릅니다.) 선택: Error Lens, CodeLLDB, Even Better TOML, C/C++, Python.

**네이티브 Windows(MSVC) 경로**도 Rust 자체는 설치할 수 있습니다(`rustup-init.exe` + Visual Studio Build Tools). 하지만 이 저장소의 C++ 참조 구현(`g++`, `libcurl`, `nlohmann-json`)과 bash 스크립트가 Linux를 가정하기 때문에 `compare.sh`가 동작하지 않습니다. Rust 코드만 직접 `cargo run -p ...`로 돌린다면 가능하지만 **추천하지 않아요.**

### 3.2 macOS

터미널에서 순서대로 실행하세요. (`scripts/setup.sh`는 macOS에서는 자동 설치를 하지 않고 같은 안내를 출력만 합니다.)

```bash
xcode-select --install                 # clang/g++ (C++20을 지원하는 최신 Xcode CLT)
brew install nlohmann-json curl python # Homebrew 가 없다면 먼저 https://brew.sh 에서 설치
pip3 install anthropic
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

확인:

```bash
cargo --version
g++ --version        # macOS에서는 clang 이 g++ 라는 이름으로 연결됩니다
scripts/compare.sh ch01
```

> ⚠️ macOS의 `g++`는 실제로는 Apple clang입니다. Homebrew의 `nlohmann-json`/`curl` 헤더 경로를 못 찾는다는 컴파일 에러가 나면 `brew --prefix`로 경로를 확인하고 `CPLUS_INCLUDE_PATH`, `LIBRARY_PATH`에 `$(brew --prefix)/include`, `$(brew --prefix)/lib`를 추가하세요. 챕터 실습의 **C++ 부분은 읽기용 참고 구현**이므로 막히면 Rust와 Python만 먼저 진행해도 됩니다.

### 3.3 Linux (Ubuntu/Debian 계열)

```bash
bash scripts/setup.sh
source "$HOME/.cargo/env"     # 새 터미널을 열어도 됩니다
cargo --version
scripts/compare.sh ch01
```

`apt-get`이 없는 배포판(Fedora, Arch 등)에서는 스크립트가 알려 주는 대로 `g++`, `nlohmann-json`, `libcurl` 개발 패키지, `python3`, `pip`을 직접 설치하세요. (예: Fedora `sudo dnf install gcc-c++ json-devel libcurl-devel python3-pip`)

### 3.4 에디터

어떤 OS든 [VS Code](https://code.visualstudio.com/) + **rust-analyzer** 확장을 추천합니다. 인레이 힌트, Problems 패널, Quick Fix 사용법은 위 [2.4](#24-rust-analyzer-사용법)와 같아요.

---

## 4. 방법 C. Rust Playground

[play.rust-lang.org](https://play.rust-lang.org)는 브라우저에서 Rust 코드를 바로 실행해 보는 공식 놀이터입니다. 설치도 로그인도 필요 없어요.

**잘 맞는 용도**
- 이론 문서(README)의 짧은 코드 조각을 붙여 넣어 실행해 보기
- 에러 메시지가 어떻게 나오는지 확인하기 (🔬 실험 블록)
- "이 문법이 맞나?" 한 줄 확인, 코드 공유(Share 버튼으로 링크 생성)
- 도구: `Run`, `Build`, `Test`, `Format`(rustfmt), `Clippy`, `Share`, 에디션 선택(**2024**로 맞추세요)

**한계**
- **파일 하나**로만 동작합니다. 모듈 분리(ch10)나 workspace는 연습할 수 없어요.
- **로컬 파일 접근 불가**: `data/*.csv`, `data/app.log` 같은 입력 파일을 읽는 챕터는 못 합니다.
- **네트워크 불가**: HTTP 요청(ch16, ch17)은 실행되지 않습니다.
- 사용 가능한 외부 크레이트가 **제한적**입니다 (인기 크레이트 일부만 미리 포함. `tokio`, `serde` 등은 되는 경우가 많지만 목록과 버전은 Playground가 정한 대로입니다).
- 실행 시간/메모리 제한이 있고, Python/C++ 비교 스크립트를 쓸 수 없습니다.

결론: Playground는 **보조 도구**입니다. 실습은 A 또는 B에서 하세요.

---

## 5. Claude Code 앱은 어떻게 같이 쓰나요?

이 저장소는 AI 에이전트(Claude Code 등)와 함께 코딩하는 연습을 목표로 합니다. 그런데 "AI가 코드를 다 써 주면 되지 않나요?"라고 생각하기 쉬워요. 솔직하게 정리하면 다음과 같습니다.

### 5.1 Claude Code 앱(클라우드 세션)이 잘하는 것

- **질문하기**: "왜 여기서 소유권이 이동하죠?", "`&str`와 `String`은 언제 구분하죠?"
- **코드 리뷰 받기**: 내가 쓴 풀이를 보여 주고 관용구, 불필요한 `clone()`, 에러 처리 문제를 지적받기
- **컴파일러 에러 해설**: 에러 전문을 붙여 넣고 "어떤 규칙 때문에 나는 건가요?"
- **참고 자료 만들기/밀어 넣기**: 추가 예제, 정리 문서, 퀴즈를 만들어 저장소에 반영(push)하게 하기
- **비교 설명**: "이 Rust 코드를 C++로 치면 어떤 코드인가요?"

### 5.2 그래도 Rust 코드는 직접 타이핑하세요

클라우드 세션의 AI는 코드를 순식간에 완성해 주지만, **학습자가 컴파일러 에러를 눈으로 보지 못하게** 만드는 부작용이 있습니다. Rust 학습의 핵심은 컴파일러와 대화하는 경험이에요.

- 소유권/빌림/라이프타임 규칙은 **에러를 직접 만나고, 직접 읽고, 직접 고쳐 볼 때** 머릿속에 남습니다.
- rust-analyzer의 실시간 에러 표시와 인레이 힌트는 Codespaces/로컬 에디터에서만 보여요.
- AI가 대신 써 준 코드를 "그럴듯하네" 하고 넘기면, 나중에 AI가 만든 코드를 **리뷰할 능력**이 생기지 않습니다. (이 저장소의 최종 목표는 리뷰 능력입니다. → [02 문서](02_vibe_coding_with_rust.md))

### 5.3 추천 워크플로

```text
1. 챕터 README 이론 읽기
2. Codespaces/로컬에서 rust/src/main.rs 의 TODO(1) 를 내가 직접 작성
3. 터미널에서 실행
     cargo test -p chNN_slug          # 테스트 확인
     scripts/compare.sh chNN          # Python / C++ / Rust 출력 비교
4. 컴파일 에러가 나면 먼저 직접 읽어 보기 (→ docs/03_reading_compiler_errors.md)
5. 30분 이상 막히면, 에러 전문 + 내 코드를 Claude 에게 붙여 넣고 "왜?" 라고 질문
     (X) "이거 고쳐 줘"
     (O) "이 에러가 어떤 규칙 때문에 나는지 설명해 주세요. 정답 코드는 주지 마세요."
6. 다 풀면 Claude 에게 코드 리뷰 요청
     "관용적이지 않은 부분과 불필요한 clone 을 찾아 주세요"
7. 마지막에 --solution 과 내 풀이를 비교해 보기
     scripts/compare.sh chNN --solution
8. git commit / push 로 저장
```

> 💡 **질문 요령**: "고쳐 줘"는 답을 받는 질문이고, "왜 그런지 설명해 줘 / 선택지를 2가지 보여 줘"는 이해를 받는 질문입니다. 학습 중에는 항상 후자로 물어보세요. 프롬프트 템플릿은 [02 문서의 4장](02_vibe_coding_with_rust.md#4-프롬프트-템플릿-한국어--english)에 있습니다.

---

## 6. 문제 해결 (Troubleshooting)

### 6.1 `Blocking waiting for file lock on package cache` / `... on build directory`

다른 `cargo` 명령(또는 rust-analyzer의 백그라운드 검사)이 이미 `target/` 폴더나 패키지 캐시를 쓰고 있어서 **기다리는 중**입니다. 대부분 잠시 후 저절로 풀려요.

- 잠깐 기다려 보세요. 처음 의존성을 내려받고 빌드할 때는 몇 분 걸릴 수 있습니다.
- 오래 걸리면 다른 터미널의 `cargo` 프로세스를 확인하세요: `ps aux | grep cargo`, 필요하면 `Ctrl+C`로 중단.
- rust-analyzer가 동시에 검사 중일 수 있습니다. 같은 `target/`을 공유하기 때문에, 터미널의 큰 빌드와 겹치면 서로 기다려요.

### 6.2 `not yet implemented: TODO(n): ...` — panic이 나요

**정상입니다.** 실습 파일의 빈칸은 `todo!("TODO(n): ...")`로 되어 있고, 풀기 전에 실행하면 이 메시지와 함께 의도적으로 panic합니다.

- `scripts/compare.sh`는 이 상황을 "아직 TODO가 남아 있어요: …"로 안내해 줍니다.
- 번호 순서대로(TODO(1), TODO(2) …) 채우고 `cargo test -p chNN_slug`로 확인하세요.
- 테스트가 처음에 빨간색으로 실패하는 것도 정상입니다. 하나씩 초록색으로 바꿔 가는 것이 실습이에요.

### 6.3 목(mock) 서버 포트가 사용 중이에요 (ch16, ch17)

`scripts/compare.sh`는 `tools/mock_server.py`를 **포트 8787**로 자동 실행합니다. 이런 메시지가 나오면 포트가 이미 쓰이고 있다는 뜻이에요.

```text
목 서버가 시작되지 않았어요 (포트 8787 사용 중일 수 있어요).
```

```bash
# 누가 8787 을 쓰는지 확인
ss -ltnp | grep 8787           # 또는: lsof -i :8787
# 예전에 직접 띄워 둔 목 서버 종료
pkill -f mock_server.py
```

- 다른 터미널에서 `python3 tools/mock_server.py`를 직접 띄워 둔 채라면 종료하세요.
- Codespaces에서는 포트가 자동으로 "Ports" 탭에 전달(forward)되어 보일 수 있는데, 이것 자체는 문제가 아닙니다.
- 직접 실행하는 Rust 프로그램은 환경 변수 `API_BASE_URL`(기본 `http://127.0.0.1:8787`)을 쓰므로, 다른 서버를 가리키고 있지 않은지 확인하세요: `echo $API_BASE_URL`.

### 6.4 C++ 컴파일에서 `nlohmann/json.hpp: No such file or directory`

JSON 챕터(ch15, ch17)의 C++ 구현은 nlohmann-json 헤더가 필요합니다.

```bash
sudo apt-get install -y nlohmann-json3-dev       # Ubuntu/Debian (scripts/setup.sh 가 설치)
brew install nlohmann-json                       # macOS
```

### 6.5 C++ 링크에서 `curl/curl.h: No such file` / `undefined reference to curl_easy_init`

HTTP 챕터(ch16, ch17)의 C++ 구현은 libcurl 개발 패키지와 링크 플래그(`-lcurl`, 챕터의 `cpp/BUILD_FLAGS`에 적혀 있음)가 필요합니다.

```bash
sudo apt-get install -y libcurl4-openssl-dev     # Ubuntu/Debian
brew install curl                                # macOS
```

그 외 도구가 빠져서 막혔다면 `bash scripts/setup.sh`를 다시 실행하세요(여러 번 실행해도 안전합니다).

### 6.6 `cargo: command not found`

방금 rustup으로 설치했다면 현재 터미널이 아직 PATH를 모르는 것입니다.

```bash
source "$HOME/.cargo/env"
cargo --version
```

그래도 안 되면 새 터미널을 여세요.

### 6.7 `rust-analyzer`가 아무것도 안 보여 줘요

- 오른쪽 아래 상태 표시줄이 "loading"이면 기다립니다(처음에는 몇 분).
- 저장소 **루트 폴더**를 열었는지 확인하세요(`chapters/chNN/...`만 따로 열면 workspace를 못 찾을 수 있어요).
- 명령 팔레트(`Ctrl+Shift+P`) > **rust-analyzer: Restart server**.
- Codespaces에서 확장이 비활성이면 확장 탭에서 "Install in Codespace"를 누르세요.

### 6.8 `error: package ... not found in workspace` / `cargo run -p ...`가 실패해요

- 챕터 폴더 이름과 패키지 이름이 같은지 확인하세요: `cargo run -q -p ch03_ownership`.
- 명령은 어느 폴더에서 실행해도 되지만, `scripts/compare.sh`는 **저장소 루트에서** 실행하거나 경로를 `../../scripts/compare.sh ch03`처럼 맞추세요.
- 새 챕터가 추가된 직후라면 `git pull`로 최신을 받으세요.

### 6.9 디스크/시간이 부족해요

- 첫 `cargo build`는 `tokio`, `reqwest` 등을 컴파일해서 몇 분 걸립니다. 이후에는 캐시되어 빨라져요.
- `target/` 폴더는 커질 수 있습니다. 정리하려면 `cargo clean` (다시 빌드해야 하므로 처음 한 번은 느려집니다).

---

## 7. 환경이 준비됐는지 확인하기

아래가 모두 통과하면 준비 완료입니다.

```bash
cargo --version                    # cargo 1.9x 이상
rustc --version
cargo clippy --version
g++ --version                      # C++20 지원 (g++ 11 이상 권장)
python3 --version                  # 3.12 이상 권장
scripts/compare.sh ch01 --solution # 세 구현의 출력이 똑같아요 ✅
```

다음 단계: [챕터 목록](../README.md)에서 ch01부터 시작하세요. 막히면 [01 치트시트](01_cpp_python_rust_cheatsheet.md), [03 에러 읽기](03_reading_compiler_errors.md), [02 바이브 코딩](02_vibe_coding_with_rust.md)을 참고하세요.
