# Ch16. HTTP 클라이언트 & 실전 에러 처리

> `reqwest` 로 REST API 를 호출하고, 실패(404 / 5xx / 타임아웃)를 **타입으로 표현**하며, 재시도까지 붙여 봅니다. "라이브러리는 `thiserror`, 애플리케이션은 `anyhow`" 라는 AI 생성 Rust 코드의 단골 조합을 이해하는 챕터예요.  ·  예상 소요: 이론 40분 / 실습 70분  ·  선행: Ch06(에러 처리), Ch14(async/tokio), Ch15(serde)

## 🎯 학습 목표

- `reqwest` 의 async 클라이언트로 GET/POST JSON 요청을 보내고 응답을 serde 로 해석할 수 있다.
- **"응답을 받았다(Ok)" 와 "성공 상태 코드다(2xx)" 는 다르다**는 것을 알고 상태 코드를 직접 처리할 수 있다.
- `thiserror` 로 만든 에러 enum(라이브러리 쪽)과 `anyhow` + `.context()`(애플리케이션 쪽)의 역할 차이를 설명할 수 있다.
- 타임아웃과 지수 백오프 재시도를 구현하고, "재시도할 가치가 있는 에러"를 타입으로 구분할 수 있다.
- 접속 주소 같은 설정을 환경 변수(`API_BASE_URL`)에서 읽어 오는 구조를 테스트하기 쉽게 설계할 수 있다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 (libcurl) | Python (urllib) | Rust (reqwest) |
|---|---|---|---|
| HTTP 호출 | `curl_easy_perform` (핸들 + 콜백) | `urllib.request.urlopen` | `client.get(url).send().await` |
| 4xx/5xx 응답 | 직접 `CURLINFO_RESPONSE_CODE` 확인 | **예외**(`HTTPError`) 로 옵니다 | **`Ok(response)`** — 직접 `status()` 확인 |
| JSON 처리 | nlohmann::json | `json.loads` | `serde` + `serde_json` |
| 에러 표현 | 예외 클래스 계층 | 예외 클래스 계층 | **enum** + `Result` (`thiserror`) |
| 맥락 덧붙이기 | 예외 메시지 문자열 조합 | `raise X from e` | `.context("...")` (`anyhow`) |
| 타임아웃 | `CURLOPT_TIMEOUT_MS` | `urlopen(timeout=...)` | `Client::builder().timeout(..)` |
| 재시도 | 직접 루프 + `sleep_for` | 직접 루프 + `time.sleep` | 직접 루프 + `tokio::time::sleep().await` |
| 설정 | `std::getenv` | `os.environ.get` | `std::env::var(..).ok()` |

## 📘 이론

> 💡 이 챕터의 코드 조각은 `reqwest`·`tokio` 같은 외부 크레이트를 쓰기 때문에 Rust Playground 에서는 그대로 돌아가지 않을 수 있어요.
> 실습 폴더에서 `cargo` 로 돌려 보세요. (Ch06 이전 챕터의 `std` 만 쓰는 조각은 Playground 에서도 됩니다.)

### 1. reqwest 기본: 요청 보내기

```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(2))   // 요청 전체에 대한 제한 시간
    .build()?;

let resp = client.get("http://127.0.0.1:8787/todos").send().await?;  // ← 네트워크 단계의 실패는 여기서 Err
println!("status = {}", resp.status());
let text = resp.text().await?;                                       // ← 본문 읽기 (여기서도 Err 가능)
```

- `Client` 는 연결을 재사용하는 **풀(pool)** 을 들고 있어요. 요청마다 새로 만들지 말고 한 번 만들어 계속 쓰세요. (Python 의 `requests.Session` 과 비슷)
- `.send().await` 는 **Future 를 만들어 실행**합니다. `.await` 를 빼먹으면 요청이 아예 나가지 않아요 (Ch14: Future 는 게으릅니다).
- JSON 은 `.json(&값)` 으로 보내고(`Serialize`), 받을 때는 `resp.json::<T>().await` 또는 `resp.text()` 후 `serde_json::from_str` 로 해석합니다.
  이 챕터는 후자를 써서 **Decode 에러를 우리 에러 타입으로 직접 분류**해요.

> 💡 **쉬운 설명 — 왜 `.await` 가 두 번 나오나요?**
> `send().await` 는 "응답의 머리(상태 코드, 헤더)가 도착할 때까지", `text().await` 는 "본문이 다 도착할 때까지"예요.
> 택배로 치면 "택배 도착 알림"과 "상자를 열어 물건 꺼내기"처럼 별개의 대기 지점입니다. 둘 다 실패할 수 있어요.

### 2. ⚠️ 가장 흔한 함정: 404 도 `Ok` 입니다

```rust
let resp = client.get(".../todos/999").send().await?;   // 404 여도 Ok(resp)!
// resp.status() == 404, 본문은 {"error": "todo 999 not found"}
let todo: Todo = resp.json().await?;                    // ← 에러 JSON 을 Todo 로 읽으려다 엉뚱한 "decode error"
```

`reqwest` 의 `Err` 는 **통신 자체가 실패**(연결 불가, 타임아웃, TLS 오류 …)했을 때만 나옵니다.
서버가 "404" 나 "500" 이라고 **대답해 준 것**은 통신에 성공한 것이라 `Ok` 예요.

고치는 방법은 두 가지입니다.

```rust
// (a) 직접 확인 — 이 챕터에서 하는 방식
let status = resp.status();
if !status.is_success() {
    return Err(ApiError::Http { status: status.as_u16() });
}

// (b) 짧게: error_for_status() — 4xx/5xx 면 reqwest::Error 로 바꿔 줍니다 (대신 우리 enum 의 variant 를 못 고름)
let resp = client.get(url).send().await?.error_for_status()?;
```

| | Python `urllib` | Rust `reqwest` |
|---|---|---|
| 404 가 오면 | `HTTPError` 예외 (자동) | `Ok(resp)` (직접 확인) |

> ⚠️ **Python 사용자가 헷갈리는 점**: `urllib`/`requests`(`raise_for_status` 를 부르는 경우)에서는 4xx 가 예외로 튀어나오기 때문에 "상태 코드 확인"을 안 해도 되는 줄 알기 쉬워요.
> Rust 에서는 **내가 확인하지 않으면 아무도 확인해 주지 않습니다.** AI 가 쓴 코드에서 `.send().await?` 뒤에 곧바로 `.json().await?` 가 이어지면 "상태 코드는 어디서 확인하지?" 하고 의심해 보세요.

### 3. 에러를 "종류"로 나누기: `thiserror` (라이브러리의 방식)

우리 클라이언트에서 일어날 수 있는 실패는 종류가 분명합니다.

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("todo {id} not found")]
    NotFound { id: u32 },                 // 404 를 "도메인 언어"로 번역한 것

    #[error("HTTP {status}")]
    Http { status: u16 },                 // 그 밖의 2xx 가 아닌 응답

    #[error("request timed out")]
    Timeout,

    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),    // 연결 실패 등

    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),    // 응답 JSON 이 약속과 다름
}
```

- `#[derive(Error)]` 가 `impl std::error::Error` 를, `#[error("...")]` 이 `impl Display` 를 만들어 줍니다. (Ch06 에서 손으로 쓰던 보일러플레이트예요)
- `#[from]` 은 `impl From<serde_json::Error> for ApiError` 를 만들어 줍니다. 그 덕분에 함수 안에서 `?` 한 글자로 변환돼요.
- 호출하는 쪽은 `match` 로 **종류별 대응**을 할 수 있습니다. 예: `Timeout` / 5xx 는 재시도, `NotFound` 는 사용자에게 "없어요" 안내.

```rust
impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Timeout => true,
            ApiError::Http { status } => *status >= 500,   // 서버 쪽 일시 장애
            _ => false,                                     // 404/400 은 백 번 해도 똑같습니다
        }
    }
}
```

> 💡 **쉬운 설명 — 에러도 "층"마다 번역해요.**
> 서버가 준 "HTTP 404" 는 전송 계층의 말이에요. 우리 라이브러리를 쓰는 사람은 "그런 todo 가 없다(`NotFound { id }`)" 를 알고 싶지요.
> 그래서 `get_todo` 안에서 404 만 `NotFound` 로 번역합니다 (`map_not_found`). 병원 접수처가 "코드 404" 대신 "해당 환자분은 없습니다" 라고 말해 주는 것과 같아요.

**`reqwest::Error` 에 `#[from]` 을 쓰지 않은 이유**: `reqwest::Error` 는 타임아웃과 그 밖의 네트워크 오류를 한 타입에 담고 있어요.
둘을 구분하고 싶어서 `From<reqwest::Error>` 를 직접 구현하고 `e.is_timeout()` 으로 갈라 줍니다.

### 4. 애플리케이션의 방식: `anyhow` + `.context()`

`main` 같은 "응용 프로그램" 코드는 에러의 **종류로 분기할 일이 적고**, 사용자에게 **"무엇을 하다가 왜 실패했는지"** 를 보여 주는 게 중요해요.

```rust
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {          // anyhow::Result<T> = Result<T, anyhow::Error>
    let t = client.get_todo(2).await.context("fetching todo 2")?;
    Ok(())
}
// 실패하면:   Error: fetching todo 2
//             Caused by:  todo 2 not found
```

- `anyhow::Error` 는 **"아무 에러나 담는 상자"** 입니다. `std::error::Error + Send + Sync + 'static` 이면 뭐든 `?` 로 들어가요.
- `.context("...")` / `.with_context(|| format!(...))` 는 "원인" 앞에 **맥락**을 한 겹 덧씌웁니다. `println!("{err:#}")` 은 `맥락: 원인` 으로 한 줄에 보여 줘요.
- 상자 안의 원래 타입이 궁금하면 `err.downcast_ref::<ApiError>()` 로 꺼내 볼 수 있습니다.

| | `thiserror` | `anyhow` |
|---|---|---|
| 누가 쓰나 | **라이브러리**, 재사용 모듈 | **애플리케이션** (`main`, 스크립트, 도구) |
| 에러 타입 | 내가 정의한 enum | `anyhow::Error` (타입 지워짐) |
| 호출자가 `match` 가능? | ✅ 가능 | ❌ (`downcast_ref` 로 억지로) |
| 코드 양 | 정의 필요 | 거의 없음 |
| 대표 질문 | "어떤 종류의 실패인가?" | "무슨 일을 하다 왜 실패했나?" |

> 📌 **기억할 한 줄**: 라이브러리는 `thiserror`, 애플리케이션은 `anyhow`. AI 가 만든 Rust 프로젝트의 거의 모든 `Cargo.toml` 에 이 두 줄이 같이 들어 있습니다.
> 반대로 라이브러리 함수가 `anyhow::Result` 를 돌려주면(호출자가 종류를 알 수 없음), 애플리케이션이 `unwrap()` 을 남발하면 "리뷰에서 지적할 포인트"예요.

> ⚠️ **C++ 개발자가 헷갈리는 점**: C++ 예외는 `catch (const NotFound&)` 로 "종류별 처리"를 하고, 처리하지 않으면 **자동으로 위로 올라갑니다.**
> Rust 의 `Result` 는 위로 올리려면 `?` 를 **눈에 보이게** 써야 하고, 반환 타입에 에러 종류가 **적혀 있습니다.** 눈에 보이는 대신 빠뜨릴 수 없어요.

### 5. 타임아웃

```rust
let client = reqwest::Client::builder().timeout(Duration::from_millis(200)).build()?;
// 200ms 안에 응답이 다 오지 않으면 .send().await 또는 .text().await 가 Err 를 돌려주고 e.is_timeout() == true
```

타임아웃이 없는 HTTP 호출은 서버가 멈추면 **영원히 기다립니다.** AI 에이전트처럼 외부 API 를 부르는 프로그램에서는 반드시 설정하세요.
(Ch14 의 `tokio::time::timeout(Duration, future)` 로 "Future 하나"에 제한 시간을 거는 방법도 있어요.)

### 6. 재시도와 지수 백오프

일시적인 오류(5xx, 타임아웃)는 잠깐 뒤에 다시 하면 성공하는 일이 많습니다. 다만 **너무 빨리, 너무 자주** 다시 하면 아픈 서버를 더 괴롭히니까 **기다리는 시간을 2배씩** 늘려요.

```text
시도1 실패 → 50ms 쉼 → 시도2 실패 → 100ms 쉼 → 시도3 성공
```

```rust
async fn retry_with_backoff<T, F, Fut>(max_attempts: u32, base: Duration, mut op: F) -> Result<T, ApiError>
where
    F: FnMut(u32) -> Fut,                       // 호출할 때마다 "새 Future" 를 만들어 주는 클로저
    Fut: Future<Output = Result<T, ApiError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retryable() && attempt < max_attempts => {
                tokio::time::sleep(backoff_delay(attempt, base)).await;   // std::thread::sleep 금지!
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

> 💡 **쉬운 설명 — 왜 Future 가 아니라 "Future 를 만드는 클로저"를 받나요?**
> Future 는 한 번 `.await` 하면 **소비**됩니다 (Ch03 의 move). 재시도하려면 요청을 **다시 만들어야** 하니까 "만드는 방법"(클로저)을 받는 거예요.

> ⚠️ **async 안에서 `std::thread::sleep` 은 금지**: 스레드 자체를 재워서 tokio 가 그 스레드로 돌리던 **다른 작업까지 멈춥니다.** 항상 `tokio::time::sleep(..).await`.

### 7. 환경 변수로 설정 받기 — "읽는 일"과 "처리하는 일"을 나누기

```rust
fn base_url_from(env_value: Option<String>) -> String {            // ← 순수 함수: 테스트 쉬움
    env_value.unwrap_or_else(|| "http://127.0.0.1:8787".to_string())
             .trim_end_matches('/').to_string()
}
// main 에서만 환경을 읽습니다:
let base_url = base_url_from(std::env::var("API_BASE_URL").ok());
```

`std::env::var` 는 `Result<String, VarError>` 를 돌려줘요. `.ok()` 로 `Option` 으로 바꾸면 "없으면 기본값" 패턴이 깔끔해집니다.
환경 변수를 함수 안에서 직접 읽으면 테스트가 전역 상태(환경)를 건드려야 해서 불편해요. **값을 인자로 받는 순수 함수**로 만드세요.
(Ch17 에서 API 키를 다룰 때도 같은 패턴을 씁니다. **API 키를 코드에 적거나 커밋하지 마세요.**)

## 🧯 자주 만나는 컴파일 에러

**1. `?` 가 에러 타입을 못 바꿔요 (E0277)**

```text
error[E0277]: `?` couldn't convert the error to `ApiError`
 --> src/main.rs:4:43
  |
4 |     let r = reqwest::get("http://x").await?;
  |             ------------------------------^ the trait `From<reqwest::Error>` is not implemented for `ApiError`
  |             |
  |             this can't be annotated with `?` because it has type `Result<_, reqwest::Error>`
note: `ApiError` needs to implement `From<reqwest::Error>`
```
의미: `?` 는 에러를 함수의 반환 에러 타입으로 `From` 변환해요. 변환 방법을 모르면 이 에러가 납니다.
고치기: `impl From<reqwest::Error> for ApiError` 를 쓰거나 variant 에 `#[from]` 을 붙입니다. (TODO 2)

**2. `await` 를 일반 함수에서 썼어요 (E0728)**

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> src/main.rs:7:22
  |
7 | fn g() { let x = f().await; }
  | ------               ^^^^^ only allowed inside `async` functions and blocks
  | |
  | this is not `async`
```
의미: `.await` 는 "여기서 잠시 양보할게요" 라는 뜻이라, 양보할 수 있는 `async` 함수 안에서만 쓸 수 있어요.
고치기: 함수를 `async fn` 으로 바꾸고, 가장 바깥은 `#[tokio::main] async fn main()` 으로 시작합니다.

**3. `.context()` 가 없다고 해요 (E0599)**

```text
error[E0599]: the method `context` exists for enum `Result<(), E>`, but its trait bounds were not satisfied
  = note: the following trait bounds were not satisfied:
          `E: anyhow::context::ext::StdError`
```
의미: `anyhow` 의 `.context()` 는 에러 타입이 `std::error::Error` 를 구현해야 쓸 수 있어요. 내가 만든 에러 타입이 이를 구현하지 않았다는 뜻입니다.
고치기: `#[derive(thiserror::Error)]` 를 붙이거나 `impl std::error::Error for MyError {}` 를 추가하세요.

**4. `use` 를 빼먹었는데 메서드가 안 보여요 (E0599)**

`.context(...)` 를 썼는데 "no method named `context` found" 가 나오면 `use anyhow::Context;` (트레이트를 스코프로 가져오기) 가 빠진 거예요.
트레이트가 제공하는 메서드는 그 트레이트가 스코프에 있어야 호출할 수 있습니다 (Ch08).

## 🛠️ 실습: Todo REST 클라이언트

### 무엇을 만드나요

로컬 목(mock) 서버 `tools/mock_server.py` 에 말을 거는 클라이언트를 만듭니다. 서버는 `compare.sh` 가 자동으로 켜 주고,
직접 돌릴 때는 다른 터미널에서 `python3 tools/mock_server.py --port 8787` 로 켜세요. (주소는 환경 변수 `API_BASE_URL` 로 바꿀 수 있어요.)

| 요청 | 설명 |
|---|---|
| `POST /reset` | 서버 상태 초기화 |
| `GET /todos` / `POST /todos` | 목록 / 생성 (`{"title": "..."}`, 빈 제목이면 400) |
| `GET /todos/<id>` | 조회 (없으면 404 `{"error": "todo 999 not found"}`) |
| `GET /flaky` | **처음 2번은 500**, 3번째부터 200 |
| `GET /slow?ms=1000` | 1초 뒤에 응답 (타임아웃 실험용) |

예상 출력:

```text
== reset ==
reset ok
== create ==
created #1 "Learn Rust" done=false
created #2 "Write tests" done=false
created #3 "Ship it" done=false
== list ==
#1 [ ] Learn Rust
#2 [ ] Write tests
#3 [ ] Ship it
== get ==
#2 "Write tests"
== not found ==
error: failed to fetch todo 999: todo 999 not found
  is NotFound? yes
== invalid body ==
error: HTTP 400
== flaky ==
attempt 1: HTTP 500
attempt 2: HTTP 500
attempt 3: ok (server saw 3 calls)
== timeout ==
error: request timed out
```

### 파일 구성

| 파일 | 설명 |
|---|---|
| `python/main.py` | 표준 라이브러리 `urllib` + `json` 구현. 예외 클래스 계층으로 같은 에러 분류 |
| `cpp/main.cpp` | libcurl + nlohmann::json (`cpp/BUILD_FLAGS` = `-lcurl`). `ApiError` 예외 + `ErrorKind` |
| `rust/src/main.rs` | **실습 파일** (TODO 11개) |
| `rust/examples/solution.rs` | 정답 |
| `USES_MOCK_SERVER` | 이 파일이 있으면 `compare.sh` 가 목 서버를 자동으로 켜고 끕니다 |

### 실행 방법 (모두 챕터 폴더에서)

```bash
python3 python/main.py                                  # (목 서버가 켜져 있어야 해요)
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x $(cat cpp/BUILD_FLAGS) && /tmp/x
cargo run -q -p ch16_http_errors                        # 실습 (TODO 를 풀기 전에는 todo!() 로 멈춰요)
cargo test -p ch16_http_errors                          # 네트워크 없이 도는 단위 테스트 10개
cargo run -q -p ch16_http_errors --example solution     # 정답
../../scripts/compare.sh ch16                           # 목 서버를 자동으로 켜고 세 언어를 비교
../../scripts/compare.sh ch16 --solution
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `ApiError` 의 `Display` 메시지 (도전: `thiserror` 로 교체) | `impl Display`, `thiserror` |
| 2 | `From<reqwest::Error>` — 타임아웃이면 `Timeout`, 아니면 `Network` | `From` + `?` 변환 |
| 3 | `is_retryable()` — 타임아웃과 5xx 만 `true` | `match`, `matches!` |
| 4 | `base_url_from()` — 기본값 적용과 끝 `/` 제거 | `Option`, 순수 함수 설계 |
| 5 | `backoff_delay()` — 50, 100, 200ms … | `Duration` 연산 |
| 6 | `map_not_found()` — 404 → `NotFound{id}` 번역 | 에러 계층 번역, 패턴 매칭 |
| 7 | `send()` — **상태 코드 확인** 후 본문 텍스트 반환 | reqwest `Ok(404)` 함정 |
| 8 | `list_todos()` / `create_todo()` | `serde` (`Serialize`/`Deserialize`), `.json(&body)` |
| 9 | `get_todo()` — `map_err` 로 404 번역 | `Result::map_err` |
| 10 | `retry_with_backoff()` | async 클로저(`FnMut -> Future`), `tokio::time::sleep` |
| 11 | `demo_not_found()` — `anyhow` 의 `.context()` 와 `downcast_ref` | `anyhow`, 에러 체인 `{:#}` |

TODO 1–6, 10 은 `cargo test` 로 바로 확인돼요. 7–9, 11 은 서버가 필요해서 `compare.sh ch16` 으로 확인합니다.

### 완료 기준

`cargo test -p ch16_http_errors` 가 모두 통과하고, `../../scripts/compare.sh ch16` 이 "python vs cpp", "python vs rust" 모두 **출력이 똑같아요** 를 보여 주면 끝!

### 🔬 실험 목록

1. **상태 코드 검사 빼기** — TODO(7) 에서 `is_success()` 검사를 지우고 `compare.sh ch16` 을 돌려 보세요. `not found` 단계의 출력이 어떻게 바뀌나요? (`decode error: missing field `id`` 처럼 **엉뚱한 곳에서** 터집니다. 왜일까요?)
2. **`std::thread::sleep` 써 보기** — `retry_with_backoff` 에서 `tokio::time::sleep` 대신 `std::thread::sleep` 을 써 보고, `#[tokio::main(flavor = "current_thread")]` 로 바꿔 다른 작업이 멈추는지 관찰하세요 (Ch14 복습).
3. **재시도 대상 바꾸기** — `is_retryable` 이 `Http { status: 404 }` 도 `true` 로 돌려주게 하면 어떤 일이 생길까요? (불필요한 재시도 — 그래서 에러를 "종류"로 나누는 거예요.)
4. **서버 끄고 실행하기** — 목 서버 없이 `cargo run -q -p ch16_http_errors --example solution` 을 실행해 보세요. `Network` 에러가 `anyhow` 체인(`Caused by:`)으로 어떻게 보이나요?

## 🤖 바이브 코딩 포인트

- **"상태 코드는 어디서 확인했지?"** — AI 가 쓴 `reqwest` 코드에서 `.send().await?` 다음에 바로 `.json().await?` 가 오면 4xx/5xx 를 놓치는 코드일 수 있어요. `error_for_status()` 나 `status().is_success()` 가 있는지 확인하세요.
- **`unwrap()` / `expect()` 가 네트워크 코드에 있으면 빨간불**: 네트워크는 반드시 실패합니다. `?` + 적절한 에러 타입이 맞는 모양이에요.
- **`thiserror` vs `anyhow` 구분 확인**: 라이브러리 모듈이 `anyhow::Result` 를 돌려주거나, `main` 이 장황한 커스텀 enum 을 가지고 있다면 AI 에게 "라이브러리 부분은 thiserror enum, main 은 anyhow + context 로 정리해 줘" 라고 요청해 보세요.
- **타임아웃과 재시도가 있는지 보기**: "타임아웃이 없는 `Client::new()`" 는 프로덕션 에이전트에서는 위험해요. 프롬프트 예: *"모든 외부 HTTP 호출에 타임아웃을 걸고, 5xx 와 타임아웃에만 지수 백오프로 최대 3번 재시도해 줘. 4xx 는 재시도하지 마."*
- **에러를 삼키지 않는지**: `if let Ok(x) = ...` 로 `Err` 를 조용히 버리거나 `.unwrap_or_default()` 로 에러를 숨기는 코드는 디버깅을 지옥으로 만들어요.
- **시크릿**: AI 가 예제 코드에 `"sk-..."` 같은 키를 직접 박아 두면 환경 변수(`std::env::var`)로 바꾸게 하세요.

## ✅ 셀프 체크

1. `reqwest` 로 `GET /todos/999` 를 보냈더니 서버가 404 를 줬습니다. `.send().await` 의 결과는 `Ok` 일까요 `Err` 일까요? 왜 이게 문제가 될 수 있나요?
<details><summary>정답</summary>

`Ok(response)` 입니다. reqwest 의 `Err` 는 통신 자체의 실패(연결, 타임아웃 등)이고, 404 는 서버가 "대답해 준 것"이라서요. 상태 코드를 직접 확인하지 않으면 에러 JSON 을 정상 데이터로 파싱하려다 엉뚱한 decode 에러가 나거나, 더 나쁘게는 에러를 정상 값으로 착각할 수 있어요.
</details>

2. 라이브러리 함수는 왜 `anyhow::Result` 대신 `thiserror` 로 만든 enum 을 돌려주는 게 좋을까요?
<details><summary>정답</summary>

호출하는 쪽이 에러의 **종류**를 보고 대응(재시도, 사용자 안내 등)을 정할 수 있어야 하기 때문이에요. `anyhow::Error` 는 타입이 지워져 있어서 `match` 로 분기할 수 없습니다. 반대로 최종 애플리케이션(`main`)은 종류별 분기보다 "맥락 + 원인"을 보여 주는 것이 중요해서 `anyhow` 가 편해요.
</details>

3. 어떤 에러를 재시도하고 어떤 에러는 재시도하지 않아야 하나요? 이유는?
<details><summary>정답</summary>

타임아웃, 5xx 같은 **일시적** 오류는 재시도할 가치가 있어요. 400/404 처럼 **요청 자체가 틀렸거나 대상이 없는** 오류는 몇 번을 해도 같은 결과라서 재시도하면 낭비(그리고 서버 부담)입니다. 그래서 `is_retryable()` 을 에러 타입에 둡니다.
</details>

4. `retry_with_backoff` 가 `Future` 가 아니라 `FnMut(u32) -> Fut` 클로저를 받는 이유는?
<details><summary>정답</summary>

Future 는 한 번 `.await` 하면 소비(move)되어 다시 쓸 수 없어요. 재시도하려면 매번 새 요청 Future 를 만들어야 하므로 "Future 를 만드는 방법"을 클로저로 받습니다.
</details>

5. `async fn` 안에서 `std::thread::sleep(Duration::from_secs(1))` 대신 `tokio::time::sleep(..).await` 를 써야 하는 이유는?
<details><summary>정답</summary>

`std::thread::sleep` 은 OS 스레드 자체를 재워 버려서, 그 스레드에서 돌아야 할 다른 async 작업들도 함께 멈춥니다. `tokio::time::sleep(..).await` 는 해당 작업만 잠시 양보하고 스레드는 다른 일을 계속하게 해 줘요.
</details>

## 📚 더 알아보기

- The Rust Book: [Ch9 Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html), [Ch17 Async](https://doc.rust-lang.org/book/ch17-00-async-await.html)
- Rust by Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html)
- 크레이트 문서: [reqwest](https://docs.rs/reqwest), [thiserror](https://docs.rs/thiserror), [anyhow](https://docs.rs/anyhow), [serde_json](https://docs.rs/serde_json)
- 다음 챕터 **Ch17. 캡스톤: 미니 AI 에이전트** 에서 이 챕터의 HTTP/에러 처리 + Ch15 의 serde + Ch08 의 trait 객체를 합쳐 에이전트 루프를 만듭니다.
