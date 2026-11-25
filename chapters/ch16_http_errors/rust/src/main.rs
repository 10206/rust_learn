//! Ch16. HTTP 클라이언트 & 실전 에러 처리 — Todo REST 클라이언트
//!
//! 목표
//!   - `reqwest` (async) 로 JSON API 를 호출하고 상태 코드를 직접 처리합니다.
//!   - 라이브러리 쪽 에러는 `thiserror` enum, 애플리케이션(main) 쪽은 `anyhow` + `.context()`.
//!   - 타임아웃과 재시도(지수 백오프)를 구현합니다.
//!
//! 대응 파일: ../python/main.py, ../cpp/main.cpp  (같은 출력을 내는 참조 구현)
//!
//! 진행 방법
//!   1. TODO(1) ~ TODO(11) 를 번호 순서대로 채웁니다. (README 의 TODO 표 참고)
//!   2. 순수 함수(TODO 1~6, 10)는 `cargo test -p ch16_http_errors` 로 바로 확인할 수 있어요.
//!   3. 네트워크 부분(TODO 7~9, 11)은 목 서버가 필요해서 `../../scripts/compare.sh ch16` 로 확인합니다.
//!      (직접 실행하려면 다른 터미널에서 `python3 tools/mock_server.py --port 8787`)
//!
//! 완료 기준: `cargo test -p ch16_http_errors` 전부 통과 AND
//!            `../../scripts/compare.sh ch16` 에서 Python/C++ 와 출력이 똑같다.

// 다 풀고 나면 이 줄을 지워 보세요 (아직 안 쓴 코드에 대한 경고를 잠시 꺼 둡니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::fmt;
use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8787";

// ─────────────────────────────────────────────
// TODO(1): ApiError 에 사람이 읽을 메시지(Display)를 붙여 주세요.
//   🐍 Python: class NotFound(ApiError): super().__init__(f"todo {id} not found")   (python/main.py)
//   ⚙️ C++  : struct ApiError : std::runtime_error { ... what() ... }                (cpp/main.cpp)
//   🦀 Rust : 지금은 손으로 `impl fmt::Display` 를 쓰는 틀이 있습니다. 필요한 메시지는
//             NotFound{id} -> "todo 7 not found",  Http{status} -> "HTTP 503",
//             Timeout -> "request timed out",  Network(e) -> "network error: {e}",  Decode(e) -> "decode error: {e}"
//   💡 도전: `use thiserror::Error;` 는 이미 가져와 있어요. enum 위에 #[derive(Debug, Error)] 를 쓰고
//            각 variant 에 #[error("...")] 를 붙이면 Display 와 `impl std::error::Error` 를 한 번에 만들어 줍니다.
//            (그러려면 아래 두 `impl` 블록(Display, Error)을 지워야 해요.)
//   ❓ 왜?: "라이브러리"는 에러의 종류가 분명해야 호출하는 쪽이 `match` 로 대응할 수 있어요. thiserror 는 그 보일러플레이트를 줄여 줍니다.
// ─────────────────────────────────────────────
#[derive(Debug)]
pub enum ApiError {
    NotFound { id: u32 },
    Http { status: u16 },
    Timeout,
    Network(reqwest::Error),
    Decode(serde_json::Error),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("TODO(1): variant 별 메시지를 write!(f, ...) 로 출력")
    }
}

impl std::error::Error for ApiError {}

// serde_json::Error -> ApiError::Decode 변환 (thiserror 의 #[from] 이 해 주는 일을 손으로 쓴 것)
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::Decode(e)
    }
}

// ─────────────────────────────────────────────
// TODO(2): reqwest::Error -> ApiError 변환. 타임아웃이면 Timeout, 아니면 Network(e).
//   🐍 Python: except socket.timeout: raise Timeout() / except URLError: raise NetworkError(e)
//   ⚙️ C++  : if (rc == CURLE_OPERATION_TIMEDOUT) throw ApiError(Timeout...)
//   🦀 Rust : `impl From<X> for Y` 가 있으면 `?` 가 자동으로 에러 타입을 바꿔 줍니다.
//   💡 힌트: e.is_timeout()
//   ❓ 왜?: reqwest 는 타임아웃도 그냥 reqwest::Error 로 돌려줘서, 우리 enum 에서 따로 구분해 주어야 "재시도할지" 결정할 수 있어요.
// ─────────────────────────────────────────────
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        todo!("TODO(2): is_timeout() 이면 ApiError::Timeout, 아니면 ApiError::Network(e)")
    }
}

impl ApiError {
    // ─────────────────────────────────────────────
    // TODO(3): 다시 시도해 볼 만한 에러면 true. (Timeout, 그리고 status >= 500 인 Http)
    //   🐍 Python: def retryable(self) -> bool                                 (python/main.py)
    //   ⚙️ C++  : bool retryable() const                                        (cpp/main.cpp)
    //   🦀 Rust : match self { ... }  — 404/400 은 다시 해도 똑같으니 false.
    //   💡 힌트: matches!(self, ...) 매크로도 쓸 수 있어요.
    // ─────────────────────────────────────────────
    pub fn is_retryable(&self) -> bool {
        todo!("TODO(3): Timeout 이거나 5xx 면 true")
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

#[derive(Serialize)]
struct NewTodo<'a> {
    title: &'a str,
}

#[derive(Debug, Deserialize)]
struct FlakyOk {
    attempts: u32,
}

// ─────────────────────────────────────────────
// TODO(4): 환경 변수 값(Option<String>)을 받아 기본 주소를 적용하고 끝의 '/' 를 모두 제거하세요.
//   🐍 Python: os.environ.get("API_BASE_URL", DEFAULT_BASE_URL)
//   ⚙️ C++  : std::getenv("API_BASE_URL")
//   🦀 Rust : 일부러 std::env::var 를 부르지 않고 "값"을 인자로 받아요 → 테스트하기 쉬운 설계!
//   💡 힌트: unwrap_or_else, trim_end_matches('/')
//   ❓ 왜?: 환경 변수를 직접 읽는 함수는 테스트마다 전역 상태를 건드려야 해서 불편해요. 읽는 일(main)과 처리하는 일(순수 함수)을 나누세요.
// ─────────────────────────────────────────────
fn base_url_from(env_value: Option<String>) -> String {
    todo!("TODO(4)")
}

// ─────────────────────────────────────────────
// TODO(5): 지수 백오프. attempt=1 -> base, 2 -> 2*base, 3 -> 4*base ...
//   🐍 Python: base * (2 ** (attempt - 1))
//   ⚙️ C++  : base_ms * (1 << (attempt - 1))
//   🦀 Rust : Duration 에 u32 를 곱할 수 있어요 (`base * n`).
//   💡 힌트: 2u32.pow(attempt - 1)
// ─────────────────────────────────────────────
fn backoff_delay(attempt: u32, base: Duration) -> Duration {
    todo!("TODO(5)")
}

// ─────────────────────────────────────────────
// TODO(6): 404 로 온 Http 에러만 NotFound{id} 로 "번역"하고, 나머지 에러는 그대로 돌려주세요.
//   🐍 Python: if e.status == 404: raise NotFound(id) from e
//   ⚙️ C++  : if (e.kind == Http && e.status == 404) throw ApiError(NotFound, ...)
//   🦀 Rust : match err { ApiError::Http { status: 404 } => ..., other => other }
//   ❓ 왜?: "HTTP 404" 는 전송 계층의 말이고, "그런 todo 는 없다(NotFound)" 는 우리 도메인의 말이에요. 에러도 층마다 번역합니다.
// ─────────────────────────────────────────────
fn map_not_found(err: ApiError, id: u32) -> ApiError {
    todo!("TODO(6)")
}

struct TodoClient {
    http: reqwest::Client,
    base_url: String,
}

impl TodoClient {
    // (제공됨) 타임아웃이 설정된 클라이언트를 만듭니다. build() 의 에러도 ? 로 ApiError 가 돼요 (TODO 2 덕분).
    fn new(base_url: &str, timeout: Duration) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { http, base_url: base_url.to_string() })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // ─────────────────────────────────────────────
    // TODO(7): 요청을 보내고, 2xx 가 아니면 ApiError::Http{status} 로 바꾼 뒤, 성공이면 본문 텍스트를 돌려주세요.
    //   🐍 Python: urlopen(...) — 4xx/5xx 는 HTTPError 예외로 옵니다               (python/main.py: _request)
    //   ⚙️ C++  : curl_easy_getinfo(CURLINFO_RESPONSE_CODE)                         (cpp/main.cpp: request)
    //   🦀 Rust : reqwest 는 404/500 도 `Ok(response)` 로 돌려줍니다! status 를 직접 확인해야 해요.
    //   💡 힌트: req.send().await? → resp.status().is_success() → resp.text().await?
    //   ❓ 왜?: "서버와 말이 통했다(Ok)" 와 "서버가 좋은 소식을 줬다" 는 다른 문제예요. 이 구분을 놓치면 에러 응답을 JSON 으로 파싱하다 엉뚱한 에러가 납니다.
    // ─────────────────────────────────────────────
    async fn send(&self, req: reqwest::RequestBuilder) -> Result<String, ApiError> {
        todo!("TODO(7)")
    }

    // (제공됨) GET + JSON 디코드. 제네릭 T: DeserializeOwned 덕분에 어떤 타입이든 받을 수 있어요.
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let text = self.send(self.http.get(self.url(path))).await?;
        Ok(serde_json::from_str(&text)?)
    }

    // (제공됨) POST /reset
    async fn reset(&self) -> Result<(), ApiError> {
        self.send(self.http.post(self.url("/reset")).json(&serde_json::json!({}))).await?;
        Ok(())
    }

    // ─────────────────────────────────────────────
    // TODO(8): GET /todos 와 POST /todos 를 구현하세요. (get_json 과 send 를 재사용)
    //   🐍 Python: [self._todo(o) for o in self._request("GET", "/todos")]
    //   ⚙️ C++  : request("POST", "/todos", json{{"title", title}})
    //   🦀 Rust : POST 본문은 `.json(&NewTodo { title })` — serde 의 Serialize 가 JSON 으로 바꿔 줍니다.
    //   💡 힌트: 응답 본문은 send() 가 돌려준 String → serde_json::from_str(&text)? (Decode 에러는 From 으로 자동 변환)
    // ─────────────────────────────────────────────
    async fn list_todos(&self) -> Result<Vec<Todo>, ApiError> {
        todo!("TODO(8): list_todos")
    }

    async fn create_todo(&self, title: &str) -> Result<Todo, ApiError> {
        todo!("TODO(8): create_todo")
    }

    // ─────────────────────────────────────────────
    // TODO(9): GET /todos/{id}. 404 는 map_not_found 로 NotFound{id} 로 번역하세요.
    //   💡 힌트: Result::map_err(|e| ...)
    // ─────────────────────────────────────────────
    async fn get_todo(&self, id: u32) -> Result<Todo, ApiError> {
        todo!("TODO(9)")
    }

    // (제공됨)
    async fn get_flaky(&self) -> Result<FlakyOk, ApiError> {
        self.get_json("/flaky").await
    }

    async fn get_slow(&self, ms: u32) -> Result<serde_json::Value, ApiError> {
        self.get_json(&format!("/slow?ms={ms}")).await
    }
}

// ─────────────────────────────────────────────
// TODO(10): 재시도 + 지수 백오프.
//   `op(attempt)` 를 최대 max_attempts 번 시도합니다. 성공하면 바로 반환,
//   `is_retryable()` 이고 아직 시도 횟수가 남았다면 backoff_delay(attempt, base) 만큼 쉰 뒤 다시,
//   그 밖의 에러는 즉시 반환합니다.
//   🐍 Python: retry_with_backoff(max_attempts, base, op)               (python/main.py)
//   ⚙️ C++  : template <typename F> auto retry_with_backoff(...)         (cpp/main.cpp)
//   🦀 Rust : op 는 "호출할 때마다 새 Future 를 돌려주는 클로저"(FnMut(u32) -> Fut) 입니다.
//             Future 는 한 번 .await 하면 소비되므로, 재시도하려면 매번 새로 만들어야 해서예요.
//   💡 힌트: loop + match op(attempt).await { Ok(v) => ..., Err(e) if 조건 => ..., Err(e) => ... }
//            쉬는 것은 tokio::time::sleep(..).await (std::thread::sleep 은 async 안에서 금지!)
//   ❓ 왜?: 일시적 오류(5xx, 타임아웃)는 잠깐 뒤 다시 하면 성공하곤 하지만, 404/400 은 백 번 해도 같아요. 그래서 "재시도 가능" 여부를 에러 타입이 알려 줍니다.
// ─────────────────────────────────────────────
async fn retry_with_backoff<T, F, Fut>(max_attempts: u32, base: Duration, mut op: F) -> Result<T, ApiError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    todo!("TODO(10)")
}

// ─────────────────────────────────────────────
// TODO(11): 애플리케이션 쪽 에러 처리 (anyhow).
//   1) client.get_todo(999) 를 호출하고 `.context("failed to fetch todo 999")` 로 맥락을 덧붙인 뒤
//      (성공할 리 없으니) `.unwrap_err()` 로 anyhow::Error 를 꺼내세요.
//   2) `println!("error: {err:#}")`  ← {:#} 는 "맥락: 원인" 형태로 체인 전체를 한 줄로 보여 줍니다.
//   3) `err.downcast_ref::<ApiError>()` 로 원래의 ApiError 를 꺼내, NotFound 이면
//      `println!("  is NotFound? yes")`, 아니면 "no" 를 출력하세요.
//   🐍 Python: print(f"error: failed to fetch todo 999: {e}") / isinstance(e, NotFound)
//   ⚙️ C++  : e.what() / e.kind == ErrorKind::NotFound
//   🦀 Rust : anyhow::Error 는 "아무 에러나 담는 상자"예요. 상자 안의 원래 타입이 궁금하면 downcast_ref.
//   ❓ 왜?: main 같은 "응용 프로그램" 코드는 에러 종류로 분기할 일이 적고, 사용자에게 "무슨 일을 하다 왜 실패했는지"를 보여 주는 게 중요해서 anyhow 가 편해요.
//           (라이브러리는 thiserror, 응용 프로그램은 anyhow — AI 가 만든 Rust 코드에서 가장 흔한 조합입니다.)
// ─────────────────────────────────────────────
async fn demo_not_found(client: &TodoClient) -> anyhow::Result<()> {
    todo!("TODO(11)")
}

// (제공됨) main 은 이미 완성되어 있어요. 위의 TODO 들이 채워지면 Python/C++ 와 같은 출력을 냅니다.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = base_url_from(std::env::var("API_BASE_URL").ok());
    let client = TodoClient::new(&base_url, Duration::from_secs(2))?;

    println!("== reset ==");
    client.reset().await.context("reset failed")?;
    println!("reset ok");

    println!("== create ==");
    for title in ["Learn Rust", "Write tests", "Ship it"] {
        let t = client.create_todo(title).await.with_context(|| format!("creating {title:?}"))?;
        println!("created #{} \"{}\" done={}", t.id, t.title, t.done);
    }

    println!("== list ==");
    for t in client.list_todos().await.context("listing todos")? {
        println!("#{} [{}] {}", t.id, if t.done { 'x' } else { ' ' }, t.title);
    }

    println!("== get ==");
    let t = client.get_todo(2).await.context("fetching todo 2")?;
    println!("#{} \"{}\"", t.id, t.title);

    println!("== not found ==");
    demo_not_found(&client).await?;

    println!("== invalid body ==");
    if let Err(e) = client.create_todo("").await {
        println!("error: {e}");
    }

    println!("== flaky ==");
    let client_ref = &client;
    retry_with_backoff(5, Duration::from_millis(50), |attempt| async move {
        let result = client_ref.get_flaky().await;
        match &result {
            Ok(ok) => println!("attempt {attempt}: ok (server saw {} calls)", ok.attempts),
            Err(e) => println!("attempt {attempt}: {e}"),
        }
        result
    })
    .await
    .context("flaky endpoint never succeeded")?;

    println!("== timeout ==");
    let impatient = TodoClient::new(&base_url, Duration::from_millis(200))?;
    if let Err(e) = impatient.get_slow(1000).await {
        println!("error: {e}");
    }
    Ok(())
}

// 🔬 실험 1: 상태 코드를 확인하지 않으면? TODO(7) 에서 status 검사를 일부러 빼고 compare.sh ch16 을 돌려 보세요.
//   404 응답 본문 {"error": "..."} 를 Todo 로 파싱하려다 NotFound 대신 "decode error: missing field `id`" 가 나옵니다.
//   → 질문: 왜 에러가 "엉뚱한 곳"에서 터질까요? (힌트: reqwest 의 Ok 는 "응답을 받았다" 는 뜻일 뿐)
//
// 🔬 실험 2: `std::thread::sleep` 을 retry_with_backoff 안에 써 보세요. 컴파일은 되지만 tokio 의 일꾼 스레드를 막습니다.
//   → 단일 스레드 런타임(#[tokio::main(flavor = "current_thread")])에서는 다른 작업이 전부 멈춰요. (ch14 복습)

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn display_messages() {
        assert_eq!(ApiError::NotFound { id: 7 }.to_string(), "todo 7 not found");
        assert_eq!(ApiError::Http { status: 503 }.to_string(), "HTTP 503");
        assert_eq!(ApiError::Timeout.to_string(), "request timed out");
    }

    #[test]
    fn only_timeouts_and_5xx_are_retryable() {
        assert!(ApiError::Timeout.is_retryable());
        assert!(ApiError::Http { status: 500 }.is_retryable());
        assert!(ApiError::Http { status: 503 }.is_retryable());
        assert!(!ApiError::Http { status: 400 }.is_retryable());
        assert!(!ApiError::NotFound { id: 1 }.is_retryable());
    }

    #[test]
    fn backoff_doubles() {
        let base = Duration::from_millis(50);
        assert_eq!(backoff_delay(1, base), Duration::from_millis(50));
        assert_eq!(backoff_delay(2, base), Duration::from_millis(100));
        assert_eq!(backoff_delay(4, base), Duration::from_millis(400));
    }

    #[test]
    fn base_url_default_and_trim() {
        assert_eq!(base_url_from(None), "http://127.0.0.1:8787");
        assert_eq!(base_url_from(Some("http://example.test:9000/".into())), "http://example.test:9000");
    }

    #[test]
    fn map_not_found_only_touches_404() {
        assert!(matches!(map_not_found(ApiError::Http { status: 404 }, 9), ApiError::NotFound { id: 9 }));
        assert!(matches!(map_not_found(ApiError::Http { status: 500 }, 9), ApiError::Http { status: 500 }));
    }

    #[test]
    fn decode_error_comes_from_serde() {
        let result: Result<Todo, ApiError> = serde_json::from_str::<Todo>(r#"{"id":1}"#).map_err(ApiError::from);
        assert!(matches!(result, Err(ApiError::Decode(_))));
        let ok: Todo = serde_json::from_str(r#"{"id":1,"title":"a","done":true}"#).unwrap();
        assert_eq!(ok, Todo { id: 1, title: "a".into(), done: true });
    }

    #[test]
    fn non_timeout_reqwest_error_becomes_network() {
        // 잘못된 URL 은 요청을 만드는 단계에서 builder 에러가 납니다 (타임아웃이 아님).
        let err = reqwest::Client::new().get("http://[").build().unwrap_err();
        assert!(matches!(ApiError::from(err), ApiError::Network(_)));
    }

    #[tokio::test]
    async fn retry_succeeds_after_failures() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result = retry_with_backoff(5, Duration::from_millis(1), move |attempt| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                if attempt < 3 { Err(ApiError::Http { status: 500 }) } else { Ok(attempt) }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_stops_on_non_retryable_error() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<(), ApiError> = retry_with_backoff(5, Duration::from_millis(1), move |_| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(ApiError::NotFound { id: 1 })
            }
        })
        .await;
        assert!(matches!(result, Err(ApiError::NotFound { id: 1 })));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_gives_up_after_max_attempts() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<(), ApiError> = retry_with_backoff(3, Duration::from_millis(1), move |_| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(ApiError::Timeout)
            }
        })
        .await;
        assert!(matches!(result, Err(ApiError::Timeout)));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
}
