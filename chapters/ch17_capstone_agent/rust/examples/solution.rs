//! Ch17 정답: 미니 AI 에이전트 (reqwest + serde + 비동기 Tool 트레이트)
//!
//! 실행: cargo run -q -p ch17_capstone_agent --example solution   (목 서버 필요: scripts/compare.sh ch17 --solution)
//! 테스트: cargo test -p ch17_capstone_agent --example solution

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

    /// 환경 변수 "값"을 받아 설정을 만듭니다 (std::env 를 직접 읽지 않아 테스트가 쉬워요).
    /// 가짜 키("mock-key")는 목 서버일 때만 허용합니다. 진짜 API 에 가짜 키를 보내는 사고를 막아요.
    fn from_env_values(base_url: Option<String>, api_key: Option<String>, model: Option<String>) -> Result<Config, ConfigError> {
        let base_url = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()).trim_end_matches('/').to_string();
        let model = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let mut cfg = Config { base_url, api_key: String::new(), model };
        match api_key.filter(|k| !k.is_empty()) {
            Some(key) => cfg.api_key = key,
            None if !cfg.is_real() => cfg.api_key = "mock-key".to_string(),
            None => return Err(ConfigError::MissingApiKey),
        }
        Ok(cfg)
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
    /// 원본 JSON 블록들을 타입 있는 Block 으로 해석합니다. 해석할 수 없는 블록은 Block::Other.
    fn blocks(&self) -> Vec<Block> {
        self.content
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap_or(Block::Other))
            .collect()
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

/// Anthropic 의 에러 응답 {"type":"error","error":{"type":..,"message":..}} 를 LlmError::Api 로 바꿉니다.
fn parse_api_error(status: u16, body: &str) -> LlmError {
    #[derive(Deserialize)]
    struct Envelope {
        error: Inner,
    }
    #[derive(Deserialize)]
    struct Inner {
        message: String,
    }
    let message = match serde_json::from_str::<Envelope>(body) {
        Ok(env) => env.error.message,
        Err(_) => body.to_string(), // JSON 이 아니면 본문을 그대로 (프록시/게이트웨이 에러 페이지 등)
    };
    LlmError::Api { status, message }
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
        Box::pin(async move {
            let expr = str_field(input, "expression")?;
            eval_expr(expr).map(format_number)
        })
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
        Box::pin(async move { Ok(str_field(input, "text")?.split_whitespace().count().to_string()) })
    }
}

/// dir 안의 "파일 이름 하나"만 허용합니다. `../x`, `/etc/passwd`, `a/b` 는 모두 거부.
/// 에이전트의 도구는 모델이 시키는 대로 움직이므로, 입력은 항상 "믿지 않는" 것이 안전해요.
fn safe_note_path(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let mut comps = Path::new(name).components();
    match (comps.next(), comps.next()) {
        (Some(Component::Normal(_)), None) if !name.contains('\\') => Ok(dir.join(name)),
        _ => Err(format!("path not allowed: {name}")),
    }
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
        Box::pin(async move {
            let name = str_field(input, "name")?;
            let path = safe_note_path(&self.dir, name)?;
            tokio::fs::read_to_string(&path).await.map_err(|_| format!("note not found: {name}"))
        })
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

    /// API 요청의 "tools" 배열
    fn definitions(&self) -> Vec<Value> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    /// 없는 도구나 실패한 도구도 panic/에러 전파 대신 "is_error 결과"로 만들어 모델에게 돌려줍니다.
    /// 모델이 결과를 보고 스스로 방향을 바꿀 수 있도록요.
    async fn execute(&self, name: &str, input: &Value) -> ToolOutcome {
        match self.get(name) {
            None => ToolOutcome { content: format!("unknown tool: {name}"), is_error: true },
            Some(tool) => match tool.run(input).await {
                Ok(content) => ToolOutcome { content, is_error: false },
                Err(content) => ToolOutcome { content, is_error: true },
            },
        }
    }
}

/// tool_result 블록 JSON. is_error 는 실패일 때만 넣습니다.
fn tool_result_block(tool_use_id: &str, outcome: &ToolOutcome) -> Value {
    let mut block = json!({"type": "tool_result", "tool_use_id": tool_use_id, "content": outcome.content});
    if outcome.is_error {
        block["is_error"] = json!(true);
    }
    block
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

async fn run_agent(llm: &dyn Llm, model: &str, registry: &ToolRegistry, task: &str) -> Result<AgentOutcome, AgentError> {
    let mut messages = vec![Message::user_text(task)];
    let tools = registry.definitions();
    let (mut tool_calls, mut tool_errors) = (0, 0);

    println!("task: {task}");
    for turn in 1..=MAX_ITERATIONS {
        let request = MessagesRequest {
            model,
            max_tokens: MAX_TOKENS,
            system: SYSTEM_PROMPT,
            tools: &tools,
            messages: &messages,
            output_config: OutputConfig { effort: "medium" },
            fallbacks: "default",
        };
        let response = llm.create(&request).await?;
        let stop = response.stop_reason;
        println!("--- turn {turn} (stop_reason: {}) ---", stop.as_str());

        // refusal 은 content 를 읽기 전에 먼저 확인합니다.
        if stop == StopReason::Refusal {
            return Err(AgentError::Refused);
        }

        let blocks = response.blocks();
        let mut text = String::new();
        for block in &blocks {
            match block {
                Block::Thinking => println!("thinking: [block kept]"),
                Block::Text { text: t } => {
                    text.push_str(t);
                    if stop != StopReason::EndTurn {
                        println!("assistant: {t}"); // 마지막 답변은 아래 'final answer' 에서 출력
                    }
                }
                Block::ToolUse { name, input, .. } => println!("tool_use: {name} {}", preview(&input.to_string())),
                Block::Other => {}
            }
        }

        match stop {
            StopReason::EndTurn => return Ok(AgentOutcome { final_text: text, turns: turn, tool_calls, tool_errors }),
            StopReason::MaxTokens => return Err(AgentError::MaxTokens),
            StopReason::ToolUse | StopReason::PauseTurn => {}
            other => return Err(AgentError::UnexpectedStop(other.as_str())),
        }

        // assistant 의 content 는 thinking 블록까지 '그대로' 되돌려 보냅니다 (response.content 를 통째로 move).
        messages.push(Message { role: "assistant".into(), content: response.content });
        if stop == StopReason::PauseTurn {
            continue; // 서버 도구가 잠시 멈춘 턴: 그대로 다시 보내면 이어서 진행해요.
        }

        // 한 번의 assistant 턴에 나온 tool_result 는 user 메시지 '하나'에 모두 담습니다.
        let mut results = Vec::new();
        for block in &blocks {
            if let Block::ToolUse { id, name, input } = block {
                let outcome = registry.execute(name, input).await;
                tool_calls += 1;
                tool_errors += u32::from(outcome.is_error);
                println!("result: {name} {} {}", if outcome.is_error { "ERROR" } else { "ok" }, preview(&outcome.content));
                results.push(tool_result_block(id, &outcome));
            }
        }
        messages.push(Message { role: "user".into(), content: results });
    }
    Err(AgentError::MaxIterations(MAX_ITERATIONS))
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
