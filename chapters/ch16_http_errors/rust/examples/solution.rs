//! Ch16 정답: Todo REST 클라이언트 (reqwest + thiserror + anyhow)
//!
//! 실행: cargo run -q -p ch16_http_errors --example solution
//! 테스트: cargo test -p ch16_http_errors --example solution
//! (목 서버가 먼저 떠 있어야 해요: python3 tools/mock_server.py --port 8787, 또는 scripts/compare.sh ch16 --solution)

use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8787";

// ─────────────────────────────────────────────
// 라이브러리 쪽 에러: thiserror 로 "종류가 분명한" enum 을 만듭니다.
// #[error("...")] 가 Display 를 만들어 주고, std::error::Error 도 자동 구현됩니다.
// 호출하는 쪽이 `match` 로 종류별 대응(재시도? 사용자에게 안내?)을 할 수 있는 것이 장점이에요.
// ─────────────────────────────────────────────
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("todo {id} not found")]
    NotFound { id: u32 },
    #[error("HTTP {status}")]
    Http { status: u16 },
    #[error("request timed out")]
    Timeout,
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),
    // #[from] : serde_json::Error -> ApiError 자동 변환 (그래서 `?` 가 바로 동작)
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
}

// reqwest::Error 는 타임아웃과 그 밖의 네트워크 오류를 한 타입으로 돌려줍니다.
// 우리는 둘을 구분하고 싶으므로 #[from] 대신 직접 From 을 구현합니다.
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ApiError::Timeout
        } else {
            ApiError::Network(e)
        }
    }
}

impl ApiError {
    /// 다시 시도해 볼 만한 에러인가? (타임아웃, 5xx 만. 404 나 400 은 다시 해도 똑같아요)
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Timeout => true,
            ApiError::Http { status } => *status >= 500,
            _ => false,
        }
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

/// 환경 변수 값(Option)을 받아 기본 주소를 적용하고, 끝의 '/' 를 제거합니다.
/// (std::env::var 를 직접 부르지 않으니 테스트하기 쉬워요)
fn base_url_from(env_value: Option<String>) -> String {
    env_value
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// 지수 백오프: attempt=1 -> base, 2 -> 2*base, 3 -> 4*base ...
fn backoff_delay(attempt: u32, base: Duration) -> Duration {
    base * 2u32.pow(attempt - 1)
}

/// 404 로 온 Http 에러만 NotFound{id} 로 번역하고, 나머지는 그대로 둡니다.
fn map_not_found(err: ApiError, id: u32) -> ApiError {
    match err {
        ApiError::Http { status: 404 } => ApiError::NotFound { id },
        other => other,
    }
}

struct TodoClient {
    http: reqwest::Client,
    base_url: String,
}

impl TodoClient {
    fn new(base_url: &str, timeout: Duration) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { http, base_url: base_url.to_string() })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// 요청을 보내고, 2xx 가 아니면 ApiError::Http 로 바꾼 뒤 본문 텍스트를 돌려줍니다.
    /// reqwest 는 4xx/5xx 도 "성공(Ok)"으로 돌려주기 때문에 직접 확인해야 해요!
    async fn send(&self, req: reqwest::RequestBuilder) -> Result<String, ApiError> {
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ApiError::Http { status: status.as_u16() });
        }
        Ok(resp.text().await?)
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let text = self.send(self.http.get(self.url(path))).await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn reset(&self) -> Result<(), ApiError> {
        self.send(self.http.post(self.url("/reset")).json(&serde_json::json!({}))).await?;
        Ok(())
    }

    async fn list_todos(&self) -> Result<Vec<Todo>, ApiError> {
        self.get_json("/todos").await
    }

    async fn create_todo(&self, title: &str) -> Result<Todo, ApiError> {
        let text = self.send(self.http.post(self.url("/todos")).json(&NewTodo { title })).await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn get_todo(&self, id: u32) -> Result<Todo, ApiError> {
        self.get_json(&format!("/todos/{id}")).await.map_err(|e| map_not_found(e, id))
    }

    async fn get_flaky(&self) -> Result<FlakyOk, ApiError> {
        self.get_json("/flaky").await
    }

    async fn get_slow(&self, ms: u32) -> Result<serde_json::Value, ApiError> {
        self.get_json(&format!("/slow?ms={ms}")).await
    }
}

/// `op(attempt)` 를 최대 max_attempts 번 시도합니다. 재시도할 가치가 있는 에러일 때만 쉬었다가 다시 해요.
/// op 는 매번 "새 Future" 를 만들어 주는 클로저입니다 (Future 는 한 번 await 하면 소비되니까요).
async fn retry_with_backoff<T, F, Fut>(max_attempts: u32, base: Duration, mut op: F) -> Result<T, ApiError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retryable() && attempt < max_attempts => {
                tokio::time::sleep(backoff_delay(attempt, base)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// 애플리케이션 쪽 에러 처리: anyhow 로 "무엇을 하다가" 실패했는지 맥락(context)을 덧붙입니다.
async fn demo_not_found(client: &TodoClient) -> anyhow::Result<()> {
    let err = client.get_todo(999).await.context("failed to fetch todo 999").unwrap_err();
    // {:#} : 에러 체인을 "맥락: 원인" 한 줄로 출력
    println!("error: {err:#}");
    // anyhow::Error 안에서도 원래의 타입을 downcast 로 꺼내 볼 수 있어요.
    let is_not_found = matches!(err.downcast_ref::<ApiError>(), Some(ApiError::NotFound { .. }));
    println!("  is NotFound? {}", if is_not_found { "yes" } else { "no" });
    Ok(())
}

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
