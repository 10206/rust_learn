//! Ch12 정답: Rust다운 코드 (관용구 모음)
//!
//! 실행: `cargo run -q -p ch12_idiomatic_rust --example solution`
//! 테스트: `cargo test -p ch12_idiomatic_rust --example solution`
//! 린트:   `cargo clippy -p ch12_idiomatic_rust --examples -- -D warnings`

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

// ───────────── 에러 타입: String 대신 enum ─────────────

/// 설정 오류. `String` 에러와 달리 호출자가 `match` 로 종류를 구분할 수 있습니다.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    MissingField(&'static str),
    MaxTokensOutOfRange(u32),
    TemperatureOutOfRange(f64),
    TooManyRetries(u8),
    DuplicateTool(String),
    MalformedPair(String),
    UnknownKey(String),
    InvalidValue { key: &'static str, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(name) => write!(f, "missing required field: {name}"),
            Self::MaxTokensOutOfRange(n) => write!(f, "max_tokens {n} out of range 1..=8192"),
            Self::TemperatureOutOfRange(t) => write!(f, "temperature {t:.1} out of range 0.0..=2.0"),
            Self::TooManyRetries(n) => write!(f, "retries {n} exceeds limit 5"),
            Self::DuplicateTool(t) => write!(f, "duplicate tool: {t}"),
            Self::MalformedPair(p) => write!(f, "malformed pair: '{p}'"),
            Self::UnknownKey(k) => write!(f, "unknown key: {k}"),
            Self::InvalidValue { key, value } => write!(f, "invalid value for {key}: '{value}'"),
        }
    }
}

// 비어 있는 impl 이지만 이것만으로 `Box<dyn Error>` / `?` / anyhow 와 연결됩니다.
impl Error for ConfigError {}

// ───────────── 뉴타입: 검증된 값은 타입으로 표시 ─────────────

/// 0.0..=2.0 범위가 보장된 온도. `Temperature` 를 가진 쪽은 다시 검사할 필요가 없어요.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature(f64);

impl Temperature {
    pub fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Temperature {
    type Error = ConfigError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (0.0..=2.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ConfigError::TemperatureOutOfRange(value))
        }
    }
}

// ───────────── 설정 구조체 + Default + Display ─────────────

#[derive(Debug, Clone, PartialEq)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub tools: Vec<String>,
    pub system_prompt: Option<String>,
    pub retries: u8,
    pub timeout_secs: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            model: "default-model".to_string(),
            max_tokens: 1024,
            temperature: 1.0,
            tools: Vec::new(),
            system_prompt: None,
            retries: 2,
            timeout_secs: 30,
        }
    }
}

impl AgentConfig {
    pub fn builder(name: impl Into<String>) -> AgentConfigBuilder {
        AgentConfigBuilder::new(name)
    }
}

// `to_string()` 을 직접 만들지 않고 Display 를 구현하면 `{}` 출력과 `.to_string()` 이 공짜로 따라옵니다.
impl fmt::Display for AgentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | model={} | tokens={} | temp={:.2} | retries={} | timeout={}s | tools=[{}] | prompt=",
            self.name,
            self.model,
            self.max_tokens,
            self.temperature,
            self.retries,
            self.timeout_secs,
            join_tools(&self.tools),
        )?;
        match &self.system_prompt {
            Some(p) => write!(f, "{} chars", p.len()),
            None => f.write_str("none"),
        }
    }
}

// ───────────── 빌더: self 를 값으로 받아 값으로 돌려준다 ─────────────

#[derive(Debug, Clone, Default)]
pub struct AgentConfigBuilder {
    cfg: AgentConfig,
}

impl AgentConfigBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self { cfg: AgentConfig { name: name.into(), ..AgentConfig::default() } }
    }

    // `impl Into<String>`: &str 도 String 도 받는다 (호출 쪽에서 .to_string() 불필요)
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.cfg.name = v.into();
        self
    }
    pub fn model(mut self, v: impl Into<String>) -> Self {
        self.cfg.model = v.into();
        self
    }
    pub fn max_tokens(mut self, v: u32) -> Self {
        self.cfg.max_tokens = v;
        self
    }
    pub fn temperature(mut self, v: f64) -> Self {
        self.cfg.temperature = v;
        self
    }
    pub fn tool(mut self, v: impl Into<String>) -> Self {
        self.cfg.tools.push(v.into());
        self
    }
    pub fn tools<I, S>(mut self, vs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.cfg.tools.extend(vs.into_iter().map(Into::into));
        self
    }
    pub fn system_prompt(mut self, v: impl Into<String>) -> Self {
        self.cfg.system_prompt = Some(v.into());
        self
    }
    pub fn retries(mut self, v: u8) -> Self {
        self.cfg.retries = v;
        self
    }
    pub fn timeout_secs(mut self, v: u32) -> Self {
        self.cfg.timeout_secs = v;
        self
    }

    /// 검증은 여기서 한 번만. 통과한 `AgentConfig` 는 항상 유효합니다.
    pub fn build(self) -> Result<AgentConfig, ConfigError> {
        let mut cfg = self.cfg;
        if cfg.name.trim().is_empty() {
            return Err(ConfigError::MissingField("name"));
        }
        if cfg.model.trim().is_empty() {
            return Err(ConfigError::MissingField("model"));
        }
        if !(1..=8192).contains(&cfg.max_tokens) {
            return Err(ConfigError::MaxTokensOutOfRange(cfg.max_tokens));
        }
        cfg.temperature = Temperature::try_from(cfg.temperature)?.get(); // `?` 로 에러 전파
        if cfg.retries > 5 {
            return Err(ConfigError::TooManyRetries(cfg.retries));
        }
        let mut seen = HashSet::new();
        if let Some(dup) = cfg.tools.iter().find(|t| !seen.insert(t.as_str())) {
            return Err(ConfigError::DuplicateTool(dup.clone()));
        }
        Ok(cfg)
    }
}

// ───────────── "key=value;key=value" 파싱: TryFrom<&str> ─────────────

fn parse_num<T: FromStr>(key: &'static str, value: &str) -> Result<T, ConfigError> {
    value.parse().map_err(|_| ConfigError::InvalidValue { key, value: value.to_string() })
}

impl TryFrom<&str> for AgentConfig {
    type Error = ConfigError;

    fn try_from(spec: &str) -> Result<Self, Self::Error> {
        let mut builder = AgentConfig::builder("");
        for pair in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
            let (key, value) = pair
                .split_once('=')
                .ok_or_else(|| ConfigError::MalformedPair(pair.to_string()))?;
            let (key, value) = (key.trim(), value.trim());
            builder = match key {
                "name" => builder.name(value),
                "model" => builder.model(value),
                "max_tokens" => builder.max_tokens(parse_num("max_tokens", value)?),
                "temperature" => builder.temperature(parse_num("temperature", value)?),
                "retries" => builder.retries(parse_num("retries", value)?),
                "timeout" => builder.timeout_secs(parse_num("timeout", value)?),
                "tools" => builder.tools(value.split(',').map(str::trim).filter(|t| !t.is_empty())),
                "prompt" => builder.system_prompt(value),
                other => return Err(ConfigError::UnknownKey(other.to_string())),
            };
        }
        builder.build()
    }
}

// ───────────── 작은 유틸리티: 슬라이스 / 이터레이터 / Option 콤비네이터 ─────────────

/// `&Vec<String>` 대신 `&[String]`: Vec, 배열, 슬라이스 모두 받을 수 있습니다.
pub fn join_tools(tools: &[String]) -> String {
    tools.join(", ")
}

pub fn count_long_names(names: &[String], min_len: usize) -> usize {
    names.iter().filter(|n| n.len() >= min_len).count()
}

/// `&String` 대신 `&str`
pub fn find_tool(tools: &[String], name: &str) -> Option<usize> {
    tools.iter().position(|t| t == name)
}

pub fn prompt_len(c: &AgentConfig) -> usize {
    c.system_prompt.as_deref().map_or(0, str::len)
}

pub fn first_tool_upper(c: &AgentConfig) -> Option<String> {
    c.tools.first().map(|t| t.to_uppercase())
}

pub fn describe_temperature(t: f64) -> &'static str {
    match t {
        t if t < 0.3 => "precise",
        t if t < 0.8 => "balanced",
        _ => "creative",
    }
}

pub fn report(configs: &[AgentConfig]) -> String {
    let mut sorted: Vec<&AgentConfig> = configs.iter().collect(); // 복제하지 않고 참조만 모음
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let mut lines = vec![format!("{:<8}|{:>7}|{:>6}", "name", "tokens", "temp")];
    lines.extend(sorted.iter().map(|c| format!("{:<8}|{:>7}|{:>6.2}", c.name, c.max_tokens, c.temperature)));
    lines.push(format!("total tokens: {}", configs.iter().map(|c| c.max_tokens).sum::<u32>()));
    if let Some(largest) = configs.iter().max_by_key(|c| c.max_tokens) {
        lines.push(format!("largest: {} ({})", largest.name, largest.max_tokens));
    }
    lines.join("\n")
}

fn show_opt<T: fmt::Display>(v: Option<T>) -> String {
    v.map_or_else(|| "none".to_string(), |x| x.to_string())
}

fn main() {
    println!("== 1. builder ==");
    let helper = AgentConfig::builder("helper")
        .model("claude-sonnet")
        .max_tokens(1024)
        .temperature(0.7)
        .tool("search")
        .tool("calc")
        .system_prompt("You are a helpful assistant.")
        .build()
        .expect("helper config is valid");
    let writer = AgentConfig::builder("writer")
        .model("claude-opus")
        .max_tokens(4096)
        .temperature(0.3)
        .retries(1)
        .timeout_secs(60)
        .build()
        .expect("writer config is valid");
    let coder = AgentConfig::builder("coder")
        .tool("shell")
        .system_prompt("Write code.")
        .build()
        .expect("coder config is valid");
    for c in [&helper, &writer, &coder] {
        println!("{c}");
    }

    println!("== 2. validation ==");
    let cases = [
        ("empty name", AgentConfig::builder("").build()),
        ("hot temperature", AgentConfig::builder("a").temperature(3.5).build()),
        ("zero tokens", AgentConfig::builder("a").max_tokens(0).build()),
        ("too many retries", AgentConfig::builder("a").retries(9).build()),
        ("duplicate tool", AgentConfig::builder("a").tool("search").tool("calc").tool("search").build()),
    ];
    for (label, result) in &cases {
        match result {
            Ok(_) => println!("  {label}: ok"),
            Err(e) => println!("  {label}: error: {e}"),
        }
    }

    println!("== 3. parse ==");
    let specs = [
        "name=writer;model=claude-opus;max_tokens=2048;temperature=0.3;tools=search,calc;prompt=Be brief",
        "name=scribe",
        "name=x; max_tokens=abc",
        "model=m1",
        "name=x;color=red",
        "name=x;oops",
        "name=x;temperature=9",
    ];
    for (i, spec) in specs.iter().enumerate() {
        match AgentConfig::try_from(*spec) {
            Ok(c) => println!("  [{}] ok: {c}", i + 1),
            Err(e) => println!("  [{}] error: {e}", i + 1),
        }
    }

    println!("== 4. utilities ==");
    let tools: Vec<String> = ["search", "calc", "shell", "web_browser_long"].map(String::from).to_vec();
    println!("join_tools: {}", join_tools(&tools));
    println!("count_long_names(>=5): {}", count_long_names(&tools, 5));
    println!("find_tool shell: {}", show_opt(find_tool(&tools, "shell")));
    println!("find_tool python: {}", show_opt(find_tool(&tools, "python")));
    println!("prompt_len helper: {}", prompt_len(&helper));
    println!("prompt_len writer: {}", prompt_len(&writer));
    println!("first_tool_upper helper: {}", show_opt(first_tool_upper(&helper)));
    println!("first_tool_upper writer: {}", show_opt(first_tool_upper(&writer)));
    for t in [0.0, 0.5, 1.5] {
        println!("temperature {t:.1}: {}", describe_temperature(t));
    }

    println!("== 5. report ==");
    println!("{}", report(&[helper, writer, coder]));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(name: &str, tokens: u32) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            model: "m".to_string(),
            max_tokens: tokens,
            temperature: 0.5,
            tools: vec!["a".to_string(), "b".to_string()],
            system_prompt: Some("hello".to_string()),
            retries: 2,
            timeout_secs: 30,
        }
    }

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn join_tools_basic() {
        assert_eq!(join_tools(&names(&["a", "b", "c"])), "a, b, c");
        assert_eq!(join_tools(&names(&[])), "");
    }

    #[test]
    fn count_long_names_counts_by_length() {
        assert_eq!(count_long_names(&names(&["ab", "abcde", "abcdefg"]), 5), 2);
        assert_eq!(count_long_names(&names(&[]), 1), 0);
    }

    #[test]
    fn find_tool_position() {
        let tools = names(&["x", "y", "z"]);
        let wanted = String::from("y");
        let missing = String::from("w");
        assert_eq!(find_tool(&tools, &wanted), Some(1));
        assert_eq!(find_tool(&tools, &missing), None);
    }

    #[test]
    fn prompt_len_and_first_tool() {
        let mut c = cfg("n", 10);
        assert_eq!(prompt_len(&c), 5);
        assert_eq!(first_tool_upper(&c), Some("A".to_string()));
        c.system_prompt = None;
        c.tools.clear();
        assert_eq!(prompt_len(&c), 0);
        assert_eq!(first_tool_upper(&c), None);
    }

    #[test]
    fn describe_temperature_buckets() {
        assert_eq!(describe_temperature(0.0), "precise");
        assert_eq!(describe_temperature(0.5), "balanced");
        assert_eq!(describe_temperature(1.9), "creative");
    }

    #[test]
    fn display_format() {
        assert_eq!(
            cfg("bot", 100).to_string(),
            "bot | model=m | tokens=100 | temp=0.50 | retries=2 | timeout=30s | tools=[a, b] | prompt=5 chars"
        );
    }

    #[test]
    fn report_is_sorted_with_totals() {
        let configs = vec![cfg("zed", 100), cfg("amy", 300)];
        let text = report(&configs);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(lines[1].starts_with("amy"));
        assert!(lines[2].starts_with("zed"));
        assert_eq!(lines[3], "total tokens: 400");
        assert_eq!(lines[4], "largest: amy (300)");
        // 원본 순서는 바뀌지 않아야 합니다.
        assert_eq!(configs[0].name, "zed");
    }

    // ── 아래 테스트들은 새 API(TODO 7~10)를 확인합니다 ──

    #[test]
    fn temperature_try_from() {
        assert_eq!(Temperature::try_from(0.7).unwrap().get(), 0.7);
        assert_eq!(Temperature::try_from(2.5).unwrap_err().to_string(), "temperature 2.5 out of range 0.0..=2.0");
        assert!(Temperature::try_from(f64::NAN).is_err());
    }

    #[test]
    fn builder_defaults_and_chain() {
        let c = AgentConfig::builder("bot").model("m").tool("a").tools(["b", "c"]).system_prompt("hi").build().unwrap();
        assert_eq!(c.name, "bot");
        assert_eq!(c.tools, vec!["a", "b", "c"]);
        assert_eq!(c.max_tokens, 1024);
        assert_eq!(c.system_prompt.as_deref(), Some("hi"));
        assert_eq!(AgentConfig::default().retries, 2);
    }

    #[test]
    fn builder_validation_errors() {
        let err = |b: AgentConfigBuilder| b.build().unwrap_err().to_string();
        assert_eq!(err(AgentConfig::builder("  ")), "missing required field: name");
        assert_eq!(err(AgentConfig::builder("a").max_tokens(9000)), "max_tokens 9000 out of range 1..=8192");
        assert_eq!(err(AgentConfig::builder("a").retries(6)), "retries 6 exceeds limit 5");
        assert_eq!(err(AgentConfig::builder("a").tool("x").tool("x")), "duplicate tool: x");
        // 에러는 표준 Error 트레이트 객체로도 다룰 수 있어야 합니다.
        let boxed: Box<dyn Error> = Box::new(AgentConfig::builder("").build().unwrap_err());
        assert!(boxed.to_string().contains("name"));
    }

    #[test]
    fn try_from_spec() {
        let c = AgentConfig::try_from("name=w; max_tokens=2048;tools=a, b ;prompt=x=y").unwrap();
        assert_eq!(c.name, "w");
        assert_eq!(c.max_tokens, 2048);
        assert_eq!(c.tools, vec!["a", "b"]);
        assert_eq!(c.system_prompt.as_deref(), Some("x=y"));
        let e = |s: &str| AgentConfig::try_from(s).unwrap_err().to_string();
        assert_eq!(e("name=x;max_tokens=abc"), "invalid value for max_tokens: 'abc'");
        assert_eq!(e("name=x;color=red"), "unknown key: color");
        assert_eq!(e("name=x;oops"), "malformed pair: 'oops'");
        assert_eq!(e("model=m"), "missing required field: name");
    }
}
