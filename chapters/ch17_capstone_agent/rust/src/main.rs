//! Ch17. 캡스톤: 미니 AI 에이전트
//!
//! 목표
//!   - Claude Messages API 의 "도구 사용 루프"를 Rust 로 직접 구현합니다.
//!     (user 과제 → 모델 → stop_reason == "tool_use" → 도구 실행 → tool_result → 반복 → end_turn)
//!   - ch08 의 Tool / ToolRegistry 를 비동기(async)로, ch15 의 serde 를, ch16 의 reqwest/에러 처리를 한데 모읍니다.
//!
//! 대응 파일: ../python/main.py (공식 anthropic SDK 사용), ../cpp/main.cpp (libcurl + nlohmann)
//!
//! 진행 방법
//!   1. HTTP 연결(AnthropicClient), 계산기 파서(eval_expr), 출력 도우미, main 은 이미 만들어 두었습니다.
//!      여러분은 "에이전트 로직"인 TODO(1) ~ TODO(11) 를 번호 순서대로 채우세요.
//!   2. 대부분은 `cargo test -p ch17_capstone_agent` 로 바로 확인됩니다. (TODO 11 은 대본대로 답하는 가짜 LLM 으로 테스트해요)
//!   3. 끝나면 `../../scripts/compare.sh ch17` 로 목 서버와 실제로 대화해 보세요.
//!
//! 완료 기준: `cargo test -p ch17_capstone_agent` 전부 통과 AND `../../scripts/compare.sh ch17` 에서 Python/C++ 와 출력이 똑같다.

// 다 풀고 나면 이 줄을 지워 보세요 (아직 안 쓴 코드에 대한 경고를 잠시 꺼 둡니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

const MAX_ITERATIONS: u32 = 10;
const MAX_TOKENS: u32 = 16000;
const PREVIEW_LEN: usize = 70;
const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8787";
const DEFAULT_MODEL: &str = "claude-opus-5-5";
const ANTHROPIC_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
const SYSTEM_PROMPT: &str = "You are a concise assistant. Use the provided tools when they help.";
const DEFAULT_TASK: &str = "Read the note 'notes.txt', count its words, compute (12+30)*2, \
try to read '../secret.txt', and check the weather in Seoul. Then summarize.";

// ───────────────────────── 설정 ─────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct Config {
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Error, PartialEq)]
enum ConfigError {
    #[error("ANTHROPIC_API_KEY is required for the real API")]
    MissingApiKey,
}

impl Config {
    fn is_real(&self) -> bool {
        self.base_url.contains("api.anthropic.com")
    }

    // ─────────────────────────────────────────────
    // TODO(2): 환경 변수 "값"(Option)으로 Config 를 만드세요.
    //   - base_url: 없으면 DEFAULT_BASE_URL, 끝의 '/' 는 제거
    //   - model   : 없으면 DEFAULT_MODEL
    //   - api_key : 있고 비어 있지 않으면 그 값.
    //               없으면 → 목 서버(!is_real)일 때만 "mock-key", 진짜 API 면 Err(ConfigError::MissingApiKey)
    //   🐍 Python: config_from_env()                                          (python/main.py)
    //   ⚙️ C++  : config_from_env()                                           (cpp/main.cpp)
    //   🦀 Rust : std::env::var 를 여기서 부르지 않고 main 이 읽어서 "값"으로 넘겨줍니다 → 테스트가 쉬워요 (ch16 의 base_url_from 과 같은 설계).
    //   💡 힌트: Option::filter, 그리고 match 에 가드(`None if 조건 =>`)를 쓸 수 있어요.
    //   ❓ 왜?: 진짜 API 에 "mock-key" 를 보내 버리면 헷갈리는 401 만 받게 됩니다. 가짜 키는 목 서버에서만 허용하는 안전장치예요.
    // ─────────────────────────────────────────────
    fn from_env_values(base_url: Option<String>, api_key: Option<String>, model: Option<String>) -> Result<Config, ConfigError> {
        todo!("TODO(2)")
    }
}

// ───────────────────────── 메시지 타입 (serde) ─────────────────────────
//
// 설계 포인트: assistant 의 content 는 `Vec<Value>` (원본 JSON) 로 "그대로" 보관했다가 되돌려 보냅니다.
// 우리가 모르는 블록(thinking, 앞으로 생길 새 블록)을 타입으로 다시 만들면 필드가 사라질 수 있어요.
// 읽을 때만 `Block` 이라는 "타입 있는 뷰"로 해석합니다.

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<Value>,
}

impl Message {
    fn user_text(text: &str) -> Message {
        Message { role: "user".into(), content: vec![json!({"type": "text", "text": text})] }
    }
}

#[derive(Serialize)]
struct OutputConfig {
    effort: &'static str,
}

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    tools: &'a [Value],
    messages: &'a [Message],
    output_config: OutputConfig,
    /// 정책상 거절(refusal) 시 서버가 대체 모델로 다시 시도하게 하는 옵션 ("default")
    fallbacks: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Refusal,
    PauseTurn,
    StopSequence,
    /// 앞으로 새 값이 생겨도 파싱 자체는 실패하지 않게 합니다.
    #[serde(other)]
    Unknown,
}

impl StopReason {
    fn as_str(self) -> &'static str {
        match self {
            StopReason::EndTurn => "end_turn",
            StopReason::ToolUse => "tool_use",
            StopReason::MaxTokens => "max_tokens",
            StopReason::Refusal => "refusal",
            StopReason::PauseTurn => "pause_turn",
            StopReason::StopSequence => "stop_sequence",
            StopReason::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MessagesResponse {
    content: Vec<Value>,
    stop_reason: StopReason,
}

/// content 블록의 "읽기용" 타입 뷰. tag = "type" 으로 JSON 의 "type" 필드를 보고 variant 를 고릅니다.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    Thinking,
    #[serde(other)]
    Other,
}

impl MessagesResponse {
    // ─────────────────────────────────────────────
    // TODO(1): self.content (원본 JSON 블록들)를 타입 있는 `Block` 들로 해석해 돌려주세요.
    //   해석에 실패하는 블록은 에러로 끝내지 말고 Block::Other 로 취급합니다. (모르는 블록이 와도 에이전트가 죽지 않게!)
    //   🐍 Python: for block in response.content: if block.type == "text": ...     (python/main.py)
    //   ⚙️ C++  : block.value("type", "") == "tool_use"                             (cpp/main.cpp)
    //   🦀 Rust : serde 의 `#[serde(tag = "type")]` enum 이 "type" 필드를 보고 variant 를 골라 줍니다 (ch15).
    //   💡 힌트: serde_json::from_value::<Block>(v.clone()) 는 Result 를 돌려줘요 → unwrap_or(Block::Other)
    //   ❓ 왜?: 원본(Vec<Value>)은 API 로 되돌려 보내려고 그대로 보관하고, Block 은 "읽기 전용 뷰"로만 씁니다.
    //           응답을 타입으로 바꿨다가 다시 JSON 으로 만들면 우리가 모르는 필드(예: thinking 의 signature)가 사라질 수 있어요.
    // ─────────────────────────────────────────────
    fn blocks(&self) -> Vec<Block> {
        todo!("TODO(1)")
    }
}

#[derive(Debug, Error)]
enum LlmError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
}

// ─────────────────────────────────────────────
// TODO(3): Anthropic 의 에러 응답 본문 {"type":"error","error":{"type":"...","message":"..."}} 에서
//          message 를 꺼내 `LlmError::Api { status, message }` 를 만드세요.
//          JSON 이 아니거나 모양이 다르면(프록시 에러 페이지 등) 본문 전체를 message 로 씁니다.
//   🐍 Python: anthropic.APIStatusError(e.status_code, e.message)       (SDK 가 해 주는 일)
//   ⚙️ C++  : parsed["error"]["message"]                                (cpp/main.cpp: LlmClient::create_message)
//   🦀 Rust : 함수 안에 작은 `#[derive(Deserialize)] struct` 를 정의해서 필요한 필드만 타입으로 받으면 깔끔해요.
//   💡 힌트: serde_json::from_str::<Envelope>(body) → match Ok/Err
//   ❓ 왜?: 에러 응답도 "계약"이 있는 데이터예요. 사용자에게 `400` 만 보여 주지 말고 서버가 알려 준 이유(message)를 보여 줘야 디버깅이 됩니다.
// ─────────────────────────────────────────────
fn parse_api_error(status: u16, body: &str) -> LlmError {
    todo!("TODO(3)")
}

// ───────────────────────── 비동기 trait 와 dyn ─────────────────────────
//
// `async fn` 을 trait 에 바로 쓸 수 있게 되었지만(Rust 1.75+), 그런 trait 는 `dyn Tool` (trait object) 로
// 쓸 수 없습니다. 우리는 `Vec<Box<dyn Tool>>` 로 여러 종류의 도구를 한 목록에 담고 싶어요.
// 그래서 "Future 를 Box 에 담아 돌려주는" 일반 fn 으로 씁니다. (async-trait 크레이트가 매크로로 해 주는 일을 손으로 한 것)
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    /// 성공이면 Ok(내용), 실패면 Err(사유). 둘 다 모델에게 "결과"로 돌아갑니다 (Err -> is_error: true).
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>>;

    fn definition(&self) -> Value {
        json!({"name": self.name(), "description": self.description(), "input_schema": self.input_schema()})
    }
}

fn str_field<'v>(input: &'v Value, key: &str) -> Result<&'v str, String> {
    input.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing string field `{key}`"))
}

/// + - * / 와 괄호, 소수를 지원하는 재귀 하강 계산기 (제공 코드)
fn eval_expr(text: &str) -> Result<f64, String> {
    struct P {
        s: Vec<char>,
        pos: usize,
    }
    impl P {
        fn peek(&self) -> Option<char> {
            self.s.get(self.pos).copied()
        }
        fn expr(&mut self) -> Result<f64, String> {
            let mut v = self.term()?;
            while let Some(op @ ('+' | '-')) = self.peek() {
                self.pos += 1;
                let r = self.term()?;
                v = if op == '+' { v + r } else { v - r };
            }
            Ok(v)
        }
        fn term(&mut self) -> Result<f64, String> {
            let mut v = self.factor()?;
            while let Some(op @ ('*' | '/')) = self.peek() {
                self.pos += 1;
                let r = self.factor()?;
                if op == '*' {
                    v *= r;
                } else if r == 0.0 {
                    return Err("division by zero".into());
                } else {
                    v /= r;
                }
            }
            Ok(v)
        }
        fn factor(&mut self) -> Result<f64, String> {
            match self.peek() {
                Some('-') => {
                    self.pos += 1;
                    Ok(-self.factor()?)
                }
                Some('(') => {
                    self.pos += 1;
                    let v = self.expr()?;
                    if self.peek() != Some(')') {
                        return Err("invalid expression: expected ')'".into());
                    }
                    self.pos += 1;
                    Ok(v)
                }
                Some(c) if c.is_ascii_digit() || c == '.' => {
                    let start = self.pos;
                    while matches!(self.peek(), Some(d) if d.is_ascii_digit() || d == '.') {
                        self.pos += 1;
                    }
                    let num: String = self.s[start..self.pos].iter().collect();
                    num.parse().map_err(|_| format!("invalid expression: bad number '{num}'"))
                }
                Some(c) => Err(format!("invalid expression: unexpected '{c}'")),
                None => Err("invalid expression: unexpected end".into()),
            }
        }
    }
    let mut p = P { s: text.chars().filter(|c| *c != ' ').collect(), pos: 0 };
    let v = p.expr()?;
    match p.peek() {
        None => Ok(v),
        Some(c) => Err(format!("invalid expression: unexpected '{c}'")),
    }
}

fn format_number(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 { format!("{}", x as i64) } else { format!("{x:.2}") }
}

struct Calculator;

impl Tool for Calculator {
    fn name(&self) -> &str {
        "calculator"
    }
    fn description(&self) -> &str {
        "Evaluate an arithmetic expression with + - * / and parentheses."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {"expression": {"type": "string", "description": "e.g. (12+30)*2"}},
            "required": ["expression"]
        })
    }
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>> {
        // ─────────────────────────────────────────────
        // TODO(4): input["expression"] 을 꺼내 eval_expr 로 계산하고 format_number 로 문자열로 만드세요.
        //   🐍 Python: format_number(eval_expr(required_str(tool_input, "expression")))   (python/main.py: Calculator.run)
        //   ⚙️ C++  : format_number(ExprParser(...).parse())                             (cpp/main.cpp)
        //   🦀 Rust : str_field 가 Result<&str, String> 이고 eval_expr 도 Result<f64, String> 이라서 `?` 와 map 으로 이어 붙일 수 있어요.
        //   💡 힌트: Box::pin(async move { ... }) 안에서는 `?` 를 쓸 수 있습니다 (async 블록의 반환 타입이 Result).
        //   ❓ 왜 Box::pin(async move {..}) ?: 위쪽 BoxFuture 설명 참고. "async 블록을 Box 에 담아 trait object 로 만든다"는 관용구예요.
        // ─────────────────────────────────────────────
        Box::pin(async move { todo!("TODO(4)") })
    }
}

struct WordCount;

impl Tool for WordCount {
    fn name(&self) -> &str {
        "word_count"
    }
    fn description(&self) -> &str {
        "Count the whitespace-separated words in a text."
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]})
    }
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>> {
        // ─────────────────────────────────────────────
        // TODO(5): input["text"] 의 단어 수를 문자열로 돌려주세요. (공백/줄바꿈 기준)
        //   🐍 Python: str(len(text.split()))     ⚙️ C++: istringstream >> w 반복
        //   💡 힌트: split_whitespace().count()
        // ─────────────────────────────────────────────
        Box::pin(async move { todo!("TODO(5)") })
    }
}

// ─────────────────────────────────────────────
// TODO(6): dir 안의 "파일 이름 하나"만 허용하는 경로 검사.
//   OK : "notes.txt"                         -> Ok(dir.join("notes.txt"))
//   거부: "../secret.txt", "/etc/passwd", "a/b.txt", "..", "", "x\\y" (역슬래시)  -> Err("path not allowed: <name>")
//   🐍 Python: safe_note_path(name)     ⚙️ C++: safe_note_path(name)   (각 main 파일 참고)
//   🦀 Rust : Path::new(name).components() 는 경로를 조각(Component)으로 나눠 줍니다.
//             Component::Normal(_) 하나뿐이면 "그냥 파일 이름"이에요. (`..` 는 Component::ParentDir, `/x` 는 RootDir)
//   💡 힌트: let mut comps = Path::new(name).components(); match (comps.next(), comps.next()) { ... }
//   ❓ 왜?: 에이전트의 도구는 "모델이 시키는 대로" 움직입니다. 모델(또는 모델을 속이는 문서)이 `../../.ssh/id_rsa` 를 읽으라고 해도
//           도구가 스스로 막아야 해요. 모든 도구 입력은 "믿지 않는 입력"입니다.
// ─────────────────────────────────────────────
fn safe_note_path(dir: &Path, name: &str) -> Result<PathBuf, String> {
    todo!("TODO(6)")
}

struct ReadNote {
    dir: PathBuf,
}

impl Tool for ReadNote {
    fn name(&self) -> &str {
        "read_note"
    }
    fn description(&self) -> &str {
        "Read a text note from the data/ directory by file name."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {"name": {"type": "string", "description": "file name, e.g. notes.txt"}},
            "required": ["name"]
        })
    }
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>> {
        // ─────────────────────────────────────────────
        // TODO(7): input["name"] → safe_note_path 로 검사 → tokio::fs::read_to_string 으로 읽기.
        //          파일이 없으면 Err("note not found: <name>").
        //   🐍 Python: path.read_text()       ⚙️ C++: std::ifstream
        //   🦀 Rust : 비동기 코드 안에서는 std::fs 대신 tokio::fs 를 쓰면 다른 작업을 막지 않아요 (ch14).
        //   💡 힌트: .await.map_err(|_| format!(...))
        //   🔬 실험: 이 도구에 `{"name": "../secret.txt"}` 를 주는 테스트가 통과하는지 확인하세요. 검사를 빼면 어떻게 될까요?
        // ─────────────────────────────────────────────
        Box::pin(async move { todo!("TODO(7)") })
    }
}

struct GetWeather;

impl Tool for GetWeather {
    fn name(&self) -> &str {
        "get_weather"
    }
    fn description(&self) -> &str {
        "Get the (fixed, fake) current weather for a city."
    }
    fn input_schema(&self) -> Value {
        json!({"type": "object", "properties": {"city": {"type": "string"}}, "required": ["city"]})
    }
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>> {
        Box::pin(async move {
            match str_field(input, "city")? {
                "Seoul" => Ok("18C, clear".to_string()),
                "Busan" => Ok("21C, cloudy".to_string()),
                other => Err(format!("no weather data for {other}")),
            }
        })
    }
}

#[derive(Debug, PartialEq)]
struct ToolOutcome {
    content: String,
    is_error: bool,
}

#[derive(Default)]
struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    fn new() -> Self {
        Self::default()
    }

    fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|b| b.as_ref())
    }

    // ─────────────────────────────────────────────
    // TODO(8): API 요청의 "tools" 배열을 만드세요. 각 도구의 definition() (name/description/input_schema) 을 모읍니다.
    //   🐍 Python: [t.definition() for t in self.tools]       ⚙️ C++: definitions()
    //   💡 힌트: self.tools.iter().map(...).collect()
    // ─────────────────────────────────────────────
    fn definitions(&self) -> Vec<Value> {
        todo!("TODO(8)")
    }

    // ─────────────────────────────────────────────
    // TODO(9): 도구를 이름으로 찾아 실행하고 ToolOutcome 으로 돌려주세요.
    //   - 없는 도구   → ToolOutcome { content: "unknown tool: <name>", is_error: true }
    //   - Ok(내용)    → is_error: false
    //   - Err(사유)   → is_error: true   (panic 이나 `?` 로 에러를 위로 던지지 않습니다!)
    //   🐍 Python: ToolRegistry.execute -> (content, is_error)       ⚙️ C++: ToolRegistry::execute
    //   🦀 Rust : self.get(name) 은 Option<&dyn Tool>. tool.run(input).await 가 Result<String, String>.
    //   ❓ 왜?: 도구 실패는 "프로그램의 실패"가 아니라 "모델에게 알려 줄 정보"입니다. is_error 결과를 받은 모델은
    //           다른 방법을 시도하거나(예: 다른 파일 이름) 사용자에게 사정을 설명할 수 있어요.
    // ─────────────────────────────────────────────
    async fn execute(&self, name: &str, input: &Value) -> ToolOutcome {
        todo!("TODO(9)")
    }
}

// ─────────────────────────────────────────────
// TODO(10): tool_result 블록 JSON 을 만드세요.
//   {"type": "tool_result", "tool_use_id": <id>, "content": <문자열>}   그리고 실패일 때만 "is_error": true 추가
//   🐍 Python: {"type": "tool_result", "tool_use_id": block.id, "content": content}   (python/main.py: run_agent)
//   ⚙️ C++  : json item = {{"type", "tool_result"}, ...}; if (is_error) item["is_error"] = true;
//   🦀 Rust : json! 매크로로 만들고, `block["is_error"] = json!(true)` 처럼 나중에 키를 추가할 수 있어요.
//   ❓ 왜 tool_use_id?: 모델이 한 턴에 도구를 여러 개 부르면(병렬 호출) 어느 결과가 어느 호출의 것인지 id 로 짝을 맞춥니다.
//           짝이 하나라도 빠지면 API 가 400 으로 거절해요. (목 서버도 똑같이 검사합니다)
// ─────────────────────────────────────────────
fn tool_result_block(tool_use_id: &str, outcome: &ToolOutcome) -> Value {
    todo!("TODO(10)")
}

// ───────────────────────── LLM 클라이언트 ─────────────────────────

/// 에이전트 루프는 "어떤 LLM 인지" 몰라도 됩니다 — 테스트에서는 대본대로 답하는 가짜를 끼웁니다.
trait Llm: Send + Sync {
    fn create<'a>(&'a self, req: &'a MessagesRequest<'a>) -> BoxFuture<'a, Result<MessagesResponse, LlmError>>;
}

struct AnthropicClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl AnthropicClient {
    fn new(cfg: &Config) -> Result<Self, LlmError> {
        // thinking 이 길어질 수 있어 넉넉한 타임아웃
        let http = reqwest::Client::builder().timeout(Duration::from_secs(600)).build()?;
        Ok(Self { http, base_url: cfg.base_url.clone(), api_key: cfg.api_key.clone() })
    }
}

impl Llm for AnthropicClient {
    fn create<'a>(&'a self, req: &'a MessagesRequest<'a>) -> BoxFuture<'a, Result<MessagesResponse, LlmError>> {
        Box::pin(async move {
            let resp = self
                .http
                .post(format!("{}/v1/messages", self.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("anthropic-beta", FALLBACK_BETA)
                .json(req)
                .send()
                .await?;
            let status = resp.status();
            let text = resp.text().await?;
            if !status.is_success() {
                return Err(parse_api_error(status.as_u16(), &text));
            }
            Ok(serde_json::from_str(&text)?)
        })
    }
}

// ───────────────────────── 출력 도우미 ─────────────────────────

fn preview(s: &str) -> String {
    let s = s.replace('\n', "\\n");
    if s.chars().count() <= PREVIEW_LEN {
        s
    } else {
        let head: String = s.chars().take(PREVIEW_LEN - 3).collect();
        format!("{head}...")
    }
}

// ───────────────────────── 에이전트 루프 ─────────────────────────

#[derive(Debug, Error)]
enum AgentError {
    #[error(transparent)]
    Llm(#[from] LlmError),
    #[error("the model refused the request")]
    Refused,
    #[error("the answer was cut off (max_tokens)")]
    MaxTokens,
    #[error("unexpected stop_reason: {0}")]
    UnexpectedStop(&'static str),
    #[error("agent did not finish within {0} iterations")]
    MaxIterations(u32),
}

#[derive(Debug)]
struct AgentOutcome {
    final_text: String,
    turns: u32,
    tool_calls: u32,
    tool_errors: u32,
}

// ─────────────────────────────────────────────
// TODO(11): 에이전트 루프. 이 챕터의 핵심입니다. 아래 틀을 따라 채우세요.
//
//   let mut messages = vec![Message::user_text(task)];   // 대화 기록 (매 요청에 전부 보냅니다)
//   for turn in 1..=MAX_ITERATIONS {                     // 무한 루프 방지 장치 (max-iterations guard)
//     ① MessagesRequest 를 만들어 llm.create(&request).await? 로 호출
//        (output_config: OutputConfig{effort:"medium"}, fallbacks: "default" — 다른 필드는 상수/인자 사용)
//     ② println!("--- turn {turn} (stop_reason: {}) ---", stop.as_str());
//     ③ stop == Refusal 이면 content 를 읽기 전에 Err(AgentError::Refused)
//     ④ response.blocks() 를 돌며 출력하고 text 를 모읍니다:
//          Thinking -> "thinking: [block kept]"      Text -> "assistant: {t}" (단, EndTurn 일 때는 출력 생략: 마지막 답변은 main 이 출력)
//          ToolUse  -> "tool_use: {name} {preview(&input.to_string())}"
//     ⑤ match stop:
//          EndTurn   -> Ok(AgentOutcome { final_text: text, turns: turn, tool_calls, tool_errors })
//          MaxTokens -> Err(AgentError::MaxTokens)
//          ToolUse | PauseTurn -> 계속,   그 밖 -> Err(AgentError::UnexpectedStop(other.as_str()))
//     ⑥ messages.push(Message { role: "assistant".into(), content: response.content })
//          ⚠️ response.content 를 "그대로" 넣습니다. thinking 블록을 지우거나 다시 만들면 API 가 400 으로 거절해요.
//     ⑦ PauseTurn 이면 `continue` (서버 도구가 잠시 멈춘 턴 → 그대로 다시 보내면 이어서 진행)
//     ⑧ ToolUse 블록마다 registry.execute(name, input).await 를 실행하고
//          println!("result: {name} {ERROR|ok} {preview(&outcome.content)}") 를 출력, tool_result_block 으로 결과 수집,
//          tool_calls / tool_errors 카운트 증가
//     ⑨ messages.push(Message { role: "user".into(), content: results })
//          ⚠️ 한 턴의 모든 tool_result 는 user 메시지 "하나"에 담습니다 (도구마다 메시지를 나누면 400!).
//   }
//   Err(AgentError::MaxIterations(MAX_ITERATIONS))
//
//   🐍 Python: while/for 루프 + client.beta.messages.create(...)     (python/main.py: run_agent)
//   ⚙️ C++  : for (int turn = 1; turn <= MAX_ITERATIONS; ++turn)      (cpp/main.cpp: run_agent)
//   🦀 Rust : llm 은 `&dyn Llm` — 진짜 클라이언트든 테스트용 가짜든 같은 코드로 돌아갑니다 (ch08 의 dyn Trait).
//             request 가 messages 를 "빌려" 가므로, 호출이 끝난 뒤에야 messages.push 할 수 있어요 (ch04 의 빌림 규칙!)
//   💡 힌트: 블록 목록은 `let blocks = response.blocks();` 로 먼저 얻은 뒤에 `response.content` 를 move 하세요.
//            (blocks 는 content 를 복제해 만든 값이라 move 후에도 쓸 수 있어요.)
//   🔬 실험(루프가 돌아간 뒤 해 보세요):
//      a) ⑥ 에서 content 대신 text 블록만 골라 새 Vec 을 보내 보세요 → 목 서버가 400 (thinking 블록 누락)
//      b) ⑧ 의 결과를 도구마다 별도 user 메시지로 push 해 보세요 → 400 (tool_use 에 대응하는 tool_result 가 "바로 다음 메시지"에 없음)
//      c) MAX_ITERATIONS 를 2 로 줄이면? (또는 `AGENT_TASK=MOCK_LOOP` 로 실행)
// ─────────────────────────────────────────────
async fn run_agent(llm: &dyn Llm, model: &str, registry: &ToolRegistry, task: &str) -> Result<AgentOutcome, AgentError> {
    todo!("TODO(11)")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_env_values(
        std::env::var("LLM_BASE_URL").ok(),
        std::env::var("ANTHROPIC_API_KEY").ok(),
        std::env::var("LLM_MODEL").ok(),
    )?;
    let llm = AnthropicClient::new(&cfg)?;

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator));
    registry.register(Box::new(WordCount));
    registry.register(Box::new(ReadNote { dir: PathBuf::from("data") }));
    registry.register(Box::new(GetWeather));

    let task = std::env::var("AGENT_TASK").unwrap_or_else(|_| DEFAULT_TASK.to_string());
    let out = run_agent(&llm, &cfg.model, &registry, &task).await.context("agent run failed")?;

    println!("--- final answer ---");
    println!("{}", out.final_text);
    println!("--- done: {} turns, {} tool calls, {} tool errors ---", out.turns, out.tool_calls, out.tool_errors);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    fn run<T: Tool>(tool: T, input: Value) -> Result<String, String> {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async { tool.run(&input).await })
    }

    #[test]
    fn calculator_evaluates() {
        assert_eq!(run(Calculator, json!({"expression": "(12+30)*2"})), Ok("84".to_string()));
        assert_eq!(run(Calculator, json!({"expression": "10 / 4"})), Ok("2.50".to_string()));
        assert_eq!(run(Calculator, json!({"expression": "-3 + 5"})), Ok("2".to_string()));
    }

    #[test]
    fn calculator_reports_errors_as_err() {
        assert_eq!(run(Calculator, json!({"expression": "1/0"})), Err("division by zero".to_string()));
        assert!(run(Calculator, json!({"expression": "2 +"})).unwrap_err().starts_with("invalid expression"));
        assert_eq!(run(Calculator, json!({})), Err("missing string field `expression`".to_string()));
    }

    #[test]
    fn word_count_counts_whitespace_separated_words() {
        assert_eq!(run(WordCount, json!({"text": "  one two\nthree  "})), Ok("3".to_string()));
        assert_eq!(run(WordCount, json!({"text": ""})), Ok("0".to_string()));
    }

    #[test]
    fn safe_note_path_rejects_traversal() {
        let dir = Path::new("data");
        assert_eq!(safe_note_path(dir, "notes.txt"), Ok(PathBuf::from("data/notes.txt")));
        for bad in ["../secret.txt", "/etc/passwd", "a/b.txt", "..", "", "x\\y"] {
            assert_eq!(safe_note_path(dir, bad), Err(format!("path not allowed: {bad}")), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn read_note_reads_inside_dir_only() {
        let dir = std::env::temp_dir().join(format!("ch17_notes_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "hello note").unwrap();
        let tool = || ReadNote { dir: dir.clone() };
        assert_eq!(run(tool(), json!({"name": "a.txt"})), Ok("hello note".to_string()));
        assert_eq!(run(tool(), json!({"name": "missing.txt"})), Err("note not found: missing.txt".to_string()));
        assert_eq!(run(tool(), json!({"name": "../a.txt"})), Err("path not allowed: ../a.txt".to_string()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn registry_turns_failures_into_error_results() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Calculator));
        let ok = reg.execute("calculator", &json!({"expression": "1+2"})).await;
        assert_eq!(ok, ToolOutcome { content: "3".into(), is_error: false });
        let bad = reg.execute("calculator", &json!({"expression": "1/0"})).await;
        assert_eq!(bad, ToolOutcome { content: "division by zero".into(), is_error: true });
        let unknown = reg.execute("teleport", &json!({})).await;
        assert_eq!(unknown, ToolOutcome { content: "unknown tool: teleport".into(), is_error: true });
    }

    #[test]
    fn definitions_follow_the_api_shape() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Calculator));
        reg.register(Box::new(GetWeather));
        let defs = reg.definitions();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0]["name"], "calculator");
        assert_eq!(defs[0]["input_schema"]["type"], "object");
        assert_eq!(defs[1]["input_schema"]["required"][0], "city");
    }

    #[test]
    fn tool_result_block_marks_errors_only_when_failed() {
        let ok = tool_result_block("toolu_1", &ToolOutcome { content: "84".into(), is_error: false });
        assert_eq!(ok, json!({"type": "tool_result", "tool_use_id": "toolu_1", "content": "84"}));
        let bad = tool_result_block("toolu_2", &ToolOutcome { content: "boom".into(), is_error: true });
        assert_eq!(bad["is_error"], json!(true));
        assert_eq!(bad["tool_use_id"], "toolu_2");
    }

    #[test]
    fn blocks_keep_unknown_types_as_other() {
        let resp: MessagesResponse = serde_json::from_value(json!({
            "content": [
                {"type": "thinking", "thinking": "", "signature": "sig"},
                {"type": "text", "text": "hi"},
                {"type": "tool_use", "id": "toolu_1", "name": "calculator", "input": {"expression": "1+1"}},
                {"type": "brand_new_block", "foo": 1}
            ],
            "stop_reason": "tool_use"
        }))
        .unwrap();
        let blocks = resp.blocks();
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0], Block::Thinking);
        assert_eq!(blocks[1], Block::Text { text: "hi".into() });
        assert!(matches!(&blocks[2], Block::ToolUse { id, name, .. } if id == "toolu_1" && name == "calculator"));
        assert_eq!(blocks[3], Block::Other);
        assert_eq!(resp.stop_reason, StopReason::ToolUse);
    }

    #[test]
    fn unknown_stop_reason_does_not_break_parsing() {
        let resp: MessagesResponse = serde_json::from_value(json!({"content": [], "stop_reason": "something_new"})).unwrap();
        assert_eq!(resp.stop_reason, StopReason::Unknown);
    }

    #[test]
    fn config_mock_vs_real() {
        let mock = Config::from_env_values(None, None, None).unwrap();
        assert_eq!(mock, Config { base_url: DEFAULT_BASE_URL.into(), api_key: "mock-key".into(), model: DEFAULT_MODEL.into() });
        assert!(!mock.is_real());
        let real_no_key = Config::from_env_values(Some("https://api.anthropic.com".into()), None, None);
        assert_eq!(real_no_key, Err(ConfigError::MissingApiKey));
        let real = Config::from_env_values(Some("https://api.anthropic.com/".into()), Some("sk-test".into()), None).unwrap();
        assert!(real.is_real());
        assert_eq!(real.base_url, "https://api.anthropic.com");
        assert_eq!(real.api_key, "sk-test");
    }

    #[test]
    fn api_errors_are_parsed() {
        let body = r#"{"type":"error","error":{"type":"invalid_request_error","message":"max_tokens: Field required"}}"#;
        assert_eq!(parse_api_error(400, body).to_string(), "API error 400: max_tokens: Field required");
        assert_eq!(parse_api_error(502, "Bad Gateway").to_string(), "API error 502: Bad Gateway");
    }

    /// 대본(미리 정한 응답 목록)대로 답하는 가짜 LLM. 마지막 응답은 계속 반복됩니다.
    struct ScriptedLlm {
        responses: Vec<MessagesResponse>,
        calls: AtomicUsize,
        seen: Mutex<Vec<Vec<Message>>>,
    }

    impl ScriptedLlm {
        fn new(responses: Vec<Value>) -> Self {
            let responses = responses.into_iter().map(|v| serde_json::from_value(v).unwrap()).collect();
            Self { responses, calls: AtomicUsize::new(0), seen: Mutex::new(Vec::new()) }
        }
    }

    impl Llm for ScriptedLlm {
        fn create<'a>(&'a self, req: &'a MessagesRequest<'a>) -> BoxFuture<'a, Result<MessagesResponse, LlmError>> {
            Box::pin(async move {
                self.seen.lock().unwrap().push(req.messages.to_vec());
                let i = self.calls.fetch_add(1, Ordering::SeqCst).min(self.responses.len() - 1);
                Ok(self.responses[i].clone())
            })
        }
    }

    fn calc_registry() -> ToolRegistry {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Calculator));
        reg
    }

    #[tokio::test]
    async fn agent_runs_tools_and_echoes_assistant_content_unchanged() {
        let assistant_turn = json!([
            {"type": "thinking", "thinking": "", "signature": "sig_1"},
            {"type": "text", "text": "calculating"},
            {"type": "tool_use", "id": "toolu_1", "name": "calculator", "input": {"expression": "6*7"}},
            {"type": "tool_use", "id": "toolu_2", "name": "calculator", "input": {"expression": "1/0"}}
        ]);
        let llm = ScriptedLlm::new(vec![
            json!({"content": assistant_turn, "stop_reason": "tool_use"}),
            json!({"content": [{"type": "text", "text": "done: 42"}], "stop_reason": "end_turn"}),
        ]);
        let out = run_agent(&llm, "m", &calc_registry(), "task").await.unwrap();
        assert_eq!(out.final_text, "done: 42");
        assert_eq!((out.turns, out.tool_calls, out.tool_errors), (2, 2, 1));

        let seen = llm.seen.lock().unwrap();
        assert_eq!(seen[0].len(), 1);
        let second = &seen[1];
        assert_eq!(second.len(), 3, "user, assistant, user(tool_results)");
        // 1) assistant content 는 thinking 블록 포함 원본 그대로
        assert_eq!(second[1].role, "assistant");
        assert_eq!(Value::Array(second[1].content.clone()), assistant_turn);
        // 2) 병렬 tool_use 두 개의 결과는 user 메시지 하나에, tool_use_id 로 짝지어
        assert_eq!(second[2].role, "user");
        assert_eq!(second[2].content.len(), 2);
        assert_eq!(second[2].content[0], json!({"type": "tool_result", "tool_use_id": "toolu_1", "content": "42"}));
        // 3) 실패한 도구는 is_error: true
        assert_eq!(second[2].content[1]["is_error"], json!(true));
    }

    #[tokio::test]
    async fn agent_stops_after_max_iterations() {
        let looping = json!({
            "content": [{"type": "tool_use", "id": "toolu_1", "name": "calculator", "input": {"expression": "1+1"}}],
            "stop_reason": "tool_use"
        });
        let llm = ScriptedLlm::new(vec![looping]);
        let err = run_agent(&llm, "m", &calc_registry(), "task").await.unwrap_err();
        assert!(matches!(err, AgentError::MaxIterations(10)));
        assert_eq!(llm.calls.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn agent_handles_refusal_and_max_tokens() {
        let refusal = ScriptedLlm::new(vec![json!({"content": [], "stop_reason": "refusal"})]);
        let err = run_agent(&refusal, "m", &calc_registry(), "task").await.unwrap_err();
        assert!(matches!(err, AgentError::Refused));

        let cut = ScriptedLlm::new(vec![json!({"content": [{"type": "text", "text": "abc"}], "stop_reason": "max_tokens"})]);
        let err = run_agent(&cut, "m", &calc_registry(), "task").await.unwrap_err();
        assert!(matches!(err, AgentError::MaxTokens));
    }
}
