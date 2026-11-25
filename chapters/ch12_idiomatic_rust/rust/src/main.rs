//! Ch12. Rust다운 코드(관용구) — 리팩터링 실습
//!
//! 이 파일의 프로그램은 **이미 잘 동작**하고 테스트도 통과합니다. 다만 C++/Python 스타일로 짜여 있어서
//! Rust 답지 않아요. 여러분의 일은 **동작은 그대로, 코드를 Rust 답게** 고치는 것입니다.
//!
//! 목표
//!   - `cargo clippy -p ch12_idiomatic_rust -- -D warnings` 가 **경고 0개** 가 되게 만들기
//!   - 그 과정에서 `&[T]`/`&str` 파라미터, 이터레이터, Option 콤비네이터, `Display`, `Default`,
//!     빌더, `TryFrom`, 뉴타입, 에러 enum 을 직접 써 보기
//!
//! 대응 파일: `../python/main.py`, `../cpp/main.cpp` (같은 설정/빌더/리포트 프로그램)
//!
//! 진행 방법
//!   1. 먼저 현재 상태를 확인하세요:  `cargo clippy -p ch12_idiomatic_rust -- -D warnings`  (경고가 잔뜩 나옵니다)
//!   2. TODO(1)부터 순서대로 고치고, 매번 `cargo test -p ch12_idiomatic_rust` 와 clippy 를 다시 돌려요.
//!      (clippy 메시지 끝의 `help:` 와 링크가 가장 좋은 선생님입니다.)
//!   3. TODO(7)~(10)은 새 코드를 짜는 단계입니다. 해당 테스트의 `#[ignore]` 줄을 지우면서 진행하세요.
//!   4. 마지막에 `cargo fmt -p ch12_idiomatic_rust` 로 포맷을 맞추세요. (TODO 12)
//!
//! 완료 기준
//!   - `cargo test -p ch12_idiomatic_rust` 전부 통과 (ignored 0개)
//!   - `cargo clippy -p ch12_idiomatic_rust -- -D warnings` 경고 없음
//!   - `../../scripts/compare.sh ch12` 에서 Python / C++ / Rust 출력이 똑같음 (처음부터 똑같아야 하고, 끝까지 똑같아야 해요!)

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 안 쓰는 코드에 대한 경고가 보입니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::error::Error;
use std::fmt;

// ═════════════════════════════════════════════
// 에러 타입
// ═════════════════════════════════════════════

// ─────────────────────────────────────────────
// TODO(7): 지금 모든 에러는 `String` 입니다. 종류를 구분할 수 있는 `enum ConfigError` 로 바꾸세요.
//   🐍 Python: class ConfigError(Exception)                        (python/main.py: ConfigError)
//   ⚙️ C++  : (Built 구조체의 error 문자열)                          (cpp/main.cpp: Built)
//   🦀 Rust : 변종(variant) 8개 + `impl fmt::Display` + `impl Error for ConfigError {}`
//       MissingField(&'static str)            -> "missing required field: {name}"
//       MaxTokensOutOfRange(u32)              -> "max_tokens {n} out of range 1..=8192"
//       TemperatureOutOfRange(f64)            -> "temperature {t:.1} out of range 0.0..=2.0"
//       TooManyRetries(u8)                    -> "retries {n} exceeds limit 5"
//       DuplicateTool(String)                 -> "duplicate tool: {t}"
//       MalformedPair(String)                 -> "malformed pair: '{p}'"
//       UnknownKey(String)                    -> "unknown key: {k}"
//       InvalidValue { key: &'static str, value: String } -> "invalid value for {key}: '{value}'"
//   💡 힌트: `#[derive(Debug, Clone, PartialEq)]`, Display 안에서는 `match self { Self::X(n) => write!(f, ...) }`
//   ❓ 왜?: 문자열 에러는 호출자가 `contains("range")` 같은 짓으로 구분해야 해요. enum 이면 `match` 가 빠짐없이(exhaustive)
//           처리했는지 컴파일러가 확인해 줍니다. (Ch06 복습)
//   (지금은 자리표시자 `Todo` 변종만 있어서 컴파일이 되도록 해 둔 상태입니다. 이 enum 전체를 새로 쓰세요.)
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    Todo,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("TODO(7): ConfigError 의 Display")
    }
}

impl Error for ConfigError {}

// ─────────────────────────────────────────────
// TODO(8): 뉴타입 `Temperature` — 0.0..=2.0 범위가 "타입으로 보장" 되게 하세요.
//   🐍 Python: validate_temperature(value)                          (python/main.py: validate_temperature)
//   ⚙️ C++  : (빌더 안의 if 문)                                      (cpp/main.cpp: build)
//   🦀 Rust : `impl TryFrom<f64> for Temperature { type Error = ConfigError; ... }`
//   💡 힌트: `(0.0..=2.0).contains(&value)`  (NaN 은 contains 가 false → 에러가 되어 안전해요)
//           에러는 `ConfigError::TemperatureOutOfRange(value)`
//   ❓ 왜?: "검증을 통과한 값"을 일반 f64 와 구분되는 타입으로 만들면, 이후 코드는 다시 검사할 필요가 없고
//           실수로 검증 안 된 값을 넘기면 컴파일 에러가 납니다. (parse, don't validate)
// ─────────────────────────────────────────────
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
        todo!("TODO(8): Temperature::try_from")
    }
}

// ═════════════════════════════════════════════
// AgentConfig  (C++ 스타일: 인자 8개짜리 생성자 + 직접 만든 to_string)
// ═════════════════════════════════════════════

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

impl AgentConfig {
    // ─────────────────────────────────────────────
    // TODO(6)(+9): 인자가 8개나 되는 생성자는 순서를 틀리기 쉽습니다. (clippy: too_many_arguments)
    //   → TODO(9) 의 빌더로 대체한 뒤 이 함수를 **삭제**하세요. (삭제 전에는 아래 작은 경고들도 고쳐 보세요)
    //   작은 경고들: `x.len() == 0` → `x.is_empty()`, `a < 1 || a > 8192` → `(1..=8192).contains(&a)`,
    //               `name: name` → `name`, `count = count + 1` → `count += 1`, `.clone()` 이 필요 없는 Copy 타입
    //   🐍 Python: AgentConfigBuilder("x").model(..).build()            (python/main.py: AgentConfigBuilder)
    //   ⚙️ C++  : AgentConfigBuilder("x").model(..).build()             (cpp/main.cpp: AgentConfigBuilder)
    //   🦀 Rust : 같은 모양의 빌더 (TODO 9)
    // ─────────────────────────────────────────────
    pub fn new(
        name: String,
        model: String,
        max_tokens: u32,
        temperature: f64,
        tools: Vec<String>,
        system_prompt: Option<String>,
        retries: u8,
        timeout_secs: u32,
    ) -> Result<AgentConfig, String> {
        if name.trim().len() == 0 {
            return Err(String::from("missing required field: name"));
        }
        if model.trim().len() == 0 {
            return Err(String::from("missing required field: model"));
        }
        if max_tokens < 1 || max_tokens > 8192 {
            return Err(format!("max_tokens {} out of range 1..=8192", max_tokens));
        }
        let t = temperature.clone();
        if t < 0.0 || t > 2.0 || t.is_nan() == true {
            return Err(format!("temperature {:.1} out of range 0.0..=2.0", t));
        }
        if retries > 5 {
            return Err(format!("retries {} exceeds limit 5", retries));
        }
        for i in 0..tools.len() {
            for j in 0..i {
                if tools[i] == tools[j] {
                    return Err(format!("duplicate tool: {}", tools[i]));
                }
            }
        }
        let cfg = AgentConfig {
            name: name,
            model: model,
            max_tokens: max_tokens,
            temperature: temperature,
            tools: tools,
            system_prompt: system_prompt,
            retries: retries,
            timeout_secs: timeout_secs,
        };
        return Ok(cfg);
    }

    // ─────────────────────────────────────────────
    // TODO(5): `to_string` 을 직접 만들지 말고 `impl fmt::Display for AgentConfig` 로 바꾸세요.
    //   (clippy: inherent_to_string — "Display 를 구현하면 to_string 은 공짜로 생깁니다")
    //   형식은 그대로: "{name} | model={model} | tokens={..} | temp={:.2} | retries={..} | timeout={..}s | tools=[a, b] | prompt={none | N chars}"
    //   🐍 Python: def __str__(self)                                   (python/main.py: AgentConfig.__str__)
    //   ⚙️ C++  : std::string to_string(const AgentConfig&)            (cpp/main.cpp: to_string)
    //   🦀 Rust : `impl fmt::Display for AgentConfig { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, ...) } }`
    //   💡 힌트: 문자열을 이어 붙이는 대신 `write!(f, "...", ...)?;` 를 쓰고, prompt 부분은 `match &self.system_prompt`
    //   ❓ 왜?: Display 가 있으면 `println!("{cfg}")`, `format!("{cfg}")`, `cfg.to_string()` 이 모두 됩니다.
    // ─────────────────────────────────────────────
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.name);
        out.push_str(" | model=");
        out.push_str(&self.model);
        out.push_str(&format!(" | tokens={}", self.max_tokens));
        out.push_str(&format!(" | temp={:.2}", self.temperature));
        out.push_str(&format!(" | retries={}", self.retries));
        out.push_str(&format!(" | timeout={}s", self.timeout_secs));
        out.push_str(&format!(" | tools=[{}]", join_tools(&self.tools)));
        out.push_str(" | prompt=");
        match &self.system_prompt {
            Some(p) => {
                out.push_str(&format!("{} chars", p.len()));
            }
            None => {
                out.push_str("none");
            }
        }
        return out;
    }

    /// 빌더 시작점 (TODO 9 에서 `AgentConfigBuilder::new` 를 구현하면 됩니다)
    pub fn builder(name: impl Into<String>) -> AgentConfigBuilder {
        AgentConfigBuilder::new(name)
    }
}

// ─────────────────────────────────────────────
// TODO(9): 빌더를 구현하세요. (+ `impl Default for AgentConfig`)
//   기본값: model="default-model", max_tokens=1024, temperature=1.0, tools=[], system_prompt=None, retries=2, timeout_secs=30,
//          name 은 빈 문자열(빌더 `new(name)` 이 채움)
//   🐍 Python: return self 로 체인                                   (python/main.py: AgentConfigBuilder)
//   ⚙️ C++  : `Builder&` 를 돌려주는 체인                            (cpp/main.cpp: AgentConfigBuilder)
//   🦀 Rust : 각 설정 메서드는 `mut self` 를 받아 `Self` 를 돌려줍니다. (소유권이 체인을 따라 흘러갑니다)
//   💡 힌트: - `impl Default for AgentConfig` 를 먼저 만들고, 빌더는 `cfg: AgentConfig` 하나만 들고 있게 하세요.
//           - 문자열 인자는 `impl Into<String>` → `v.into()`  (&str 도 String 도 받아요)
//           - `tools` 는 `I: IntoIterator<Item = S>, S: Into<String>` 로 `["a", "b"]` 같은 배열도 받게 해 보세요.
//           - `build(self)` 는 `Result<AgentConfig, ConfigError>`: 순서는 name → model → max_tokens → temperature(= TODO 8 의
//             `Temperature::try_from(..)?.get()`) → retries → 중복 tool. (위 `AgentConfig::new` 의 검사 순서와 같음)
//           - 중복 검사: `HashSet::insert` 가 false 를 돌려주면 중복. (`use std::collections::HashSet;`)
//   ❓ 왜?: 필드가 많을 때 "이름 있는 인자"가 없는 Rust 에서 가장 흔한 대안입니다. 검증을 build() 한 곳에 모을 수도 있어요.
// ─────────────────────────────────────────────
impl Default for AgentConfig {
    fn default() -> Self {
        todo!("TODO(9): AgentConfig::default")
    }
}

#[derive(Debug, Clone)]
pub struct AgentConfigBuilder {
    cfg: AgentConfig,
}

impl AgentConfigBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        todo!("TODO(9): AgentConfigBuilder::new")
    }
    pub fn name(mut self, v: impl Into<String>) -> Self {
        todo!("TODO(9): name")
    }
    pub fn model(mut self, v: impl Into<String>) -> Self {
        todo!("TODO(9): model")
    }
    pub fn max_tokens(mut self, v: u32) -> Self {
        todo!("TODO(9): max_tokens")
    }
    pub fn temperature(mut self, v: f64) -> Self {
        todo!("TODO(9): temperature")
    }
    pub fn tool(mut self, v: impl Into<String>) -> Self {
        todo!("TODO(9): tool")
    }
    pub fn tools<I, S>(mut self, vs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        todo!("TODO(9): tools")
    }
    pub fn system_prompt(mut self, v: impl Into<String>) -> Self {
        todo!("TODO(9): system_prompt")
    }
    pub fn retries(mut self, v: u8) -> Self {
        todo!("TODO(9): retries")
    }
    pub fn timeout_secs(mut self, v: u32) -> Self {
        todo!("TODO(9): timeout_secs")
    }
    pub fn build(self) -> Result<AgentConfig, ConfigError> {
        todo!("TODO(9): build")
    }
}

// ─────────────────────────────────────────────
// TODO(10): `parse_config(&String) -> Result<_, String>` (아래) 를 `impl TryFrom<&str> for AgentConfig` 로 바꾸세요.
//   형식: "name=writer;model=m;max_tokens=2048;temperature=0.3;retries=1;timeout=60;tools=a,b;prompt=Be brief"
//   - 앞뒤 공백 무시, 빈 조각은 건너뛰기, `=` 는 첫 번째 것만 기준 (prompt 값에 `=` 가 있어도 OK)
//   - `=` 없음 → MalformedPair, 모르는 키 → UnknownKey, 숫자 파싱 실패 → InvalidValue { key, value }
//   - 마지막은 `builder.build()` 의 결과를 그대로 돌려줍니다.
//   🐍 Python: parse_config(spec)                                    (python/main.py: parse_config)
//   ⚙️ C++  : parse_config(std::string_view)                         (cpp/main.cpp: parse_config)
//   🦀 Rust : `spec.split(';').map(str::trim).filter(|p| !p.is_empty())`, `pair.split_once('=')`,
//           `.ok_or_else(|| ConfigError::MalformedPair(..))?`,  숫자는 `value.parse().map_err(|_| ..)?`
//   💡 힌트: 숫자 파싱은 제네릭 도우미 `fn parse_num<T: FromStr>(key: &'static str, value: &str) -> Result<T, ConfigError>` 하나로 모을 수 있어요.
//           `builder = match key { "name" => builder.name(value), ... other => return Err(..) };`
//   ❓ 왜?: `TryFrom` 은 "실패할 수 있는 변환"의 표준 이름입니다. 호출 쪽이 `AgentConfig::try_from(s)?` 또는 `s.try_into()?`.
// ─────────────────────────────────────────────
impl TryFrom<&str> for AgentConfig {
    type Error = ConfigError;

    fn try_from(spec: &str) -> Result<Self, Self::Error> {
        todo!("TODO(10): TryFrom<&str>")
    }
}

/// (C++/Python 스타일) 설정 문자열 파서. TODO(10) 을 끝내면 삭제하세요.
pub fn parse_config(spec: &String) -> Result<AgentConfig, String> {
    let pairs: Vec<&str> = spec.split(';').collect();
    let mut name = String::new();
    let mut model = String::from("default-model");
    let mut max_tokens: u32 = 1024;
    let mut temperature: f64 = 1.0;
    let mut tools: Vec<String> = Vec::new();
    let mut system_prompt: Option<String> = None;
    let mut retries: u8 = 2;
    let mut timeout_secs: u32 = 30;
    for i in 0..pairs.len() {
        let pair = pairs[i].trim();
        if pair.len() == 0 {
            continue;
        }
        let eq = pair.find('=');
        if eq.is_none() {
            return Err(format!("malformed pair: '{}'", pair));
        }
        let eq = eq.unwrap();
        let key = pair[..eq].trim();
        let value = pair[eq + 1..].trim();
        if key == "name" {
            name = value.to_string();
        } else if key == "model" {
            model = value.to_string();
        } else if key == "max_tokens" {
            match value.parse::<u32>() {
                Ok(v) => max_tokens = v,
                Err(_) => return Err(format!("invalid value for max_tokens: '{}'", value)),
            }
        } else if key == "temperature" {
            match value.parse::<f64>() {
                Ok(v) => temperature = v,
                Err(_) => return Err(format!("invalid value for temperature: '{}'", value)),
            }
        } else if key == "retries" {
            match value.parse::<u8>() {
                Ok(v) => retries = v,
                Err(_) => return Err(format!("invalid value for retries: '{}'", value)),
            }
        } else if key == "timeout" {
            match value.parse::<u32>() {
                Ok(v) => timeout_secs = v,
                Err(_) => return Err(format!("invalid value for timeout: '{}'", value)),
            }
        } else if key == "tools" {
            for t in value.split(',') {
                let t = t.trim();
                if t.len() > 0 {
                    tools.push(t.to_string());
                }
            }
        } else if key == "prompt" {
            system_prompt = Some(value.to_string());
        } else {
            return Err(format!("unknown key: {}", key));
        }
    }
    AgentConfig::new(name, model, max_tokens, temperature, tools, system_prompt, retries, timeout_secs)
}

// ═════════════════════════════════════════════
// 작은 유틸리티들 (여기가 clippy 경고의 보물창고예요)
// ═════════════════════════════════════════════

// ─────────────────────────────────────────────
// TODO(1): 파라미터 타입을 "더 넓게 받는" 타입으로 바꾸세요. (clippy: ptr_arg)
//   `&Vec<String>` → `&[String]`,  `&String` → `&str`,  `&Vec<AgentConfig>` → `&[AgentConfig]`
//   대상: join_tools, count_long_names, find_tool, parse_config(TODO 10 에서 사라짐), report
//   🐍 Python: def join_tools(tools: list[str])                       (python/main.py: join_tools)
//   ⚙️ C++  : const std::vector<std::string>& / std::string_view      (cpp/main.cpp: find_tool)
//   🦀 Rust : 슬라이스 `&[T]` 와 `&str` 이 "읽기 전용 입력"의 표준 타입입니다.
//   ❓ 왜?: `&Vec<T>` 만 받으면 배열이나 일부 구간(`&v[1..3]`)을 넘길 수 없고, `&String` 만 받으면 리터럴 `"abc"` 를 못 넘겨요.
//           호출하는 쪽 코드는 거의 안 바뀝니다 (`&vec` 는 자동으로 `&[T]` 로 변환: Deref 강제 변환).
// TODO(2): 인덱스 for 루프 → 이터레이터. (clippy: needless_range_loop 등)
//   join_tools → `tools.join(", ")`,  count_long_names → `.iter().filter(..).count()` (반환 타입도 `usize` 로),
//   find_tool → `.iter().position(..)`,  report 안의 합계/최댓값/정렬 → `.sum()`, `.max_by_key()`, `.sort_by()`
//   🐍 Python: sum(1 for n in names if len(n) >= min_len)             (python/main.py: count_long_names)
//   ⚙️ C++  : std::ranges::count_if(names, pred)                      (cpp/main.cpp: count_long_names)
//   🦀 Rust : `names.iter().filter(|n| n.len() >= min_len).count()`
//   💡 힌트: 이터레이터는 경계 검사가 필요 없어서 (인덱스 실수가 없고) 보통 같은 속도이거나 더 빠릅니다.
// ─────────────────────────────────────────────
pub fn join_tools(tools: &Vec<String>) -> String {
    let mut out = String::new();
    for i in 0..tools.len() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&tools[i]);
    }
    return out;
}

pub fn count_long_names(names: &Vec<String>, min_len: usize) -> i32 {
    let mut count = 0;
    for i in 0..names.len() {
        if names[i].len() >= min_len {
            count = count + 1;
        }
    }
    return count;
}

pub fn find_tool(tools: &Vec<String>, name: &String) -> Option<usize> {
    for i in 0..tools.len() {
        if &tools[i] == name {
            return Some(i);
        }
    }
    return None;
}

// ─────────────────────────────────────────────
// TODO(3): `is_some()` + `unwrap()` 와 `match` 로 Some/None 을 뒤집는 코드를 Option 콤비네이터로 바꾸세요.
//   (clippy: unnecessary_unwrap, manual_map)
//   prompt_len       → `c.system_prompt.as_deref().map_or(0, str::len)`
//   first_tool_upper → `c.tools.first().map(|t| t.to_uppercase())`
//   🐍 Python: len(p) if p is not None else 0                          (python/main.py: prompt_len)
//   ⚙️ C++  : c.system_prompt ? c.system_prompt->size() : 0            (cpp/main.cpp: prompt_len)
//   🦀 Rust : `Option<String>` → `as_deref()` → `Option<&str>` → `map_or(기본값, 함수)`
//   ❓ 왜?: `unwrap()` 는 "여기서 None 이면 프로그램을 죽여라" 라는 뜻입니다. 콤비네이터는 None 경우를 코드에 드러내서
//           panic 가능성을 없애요. (AI 가 만든 코드의 `unwrap()` 을 리뷰할 때 같은 시선으로 보세요)
// ─────────────────────────────────────────────
pub fn prompt_len(c: &AgentConfig) -> usize {
    if c.system_prompt.is_some() {
        let p = c.system_prompt.as_ref().unwrap();
        return p.len();
    } else {
        return 0;
    }
}

pub fn first_tool_upper(c: &AgentConfig) -> Option<String> {
    match c.tools.first() {
        Some(t) => Some(t.to_uppercase()),
        None => None,
    }
}

// ─────────────────────────────────────────────
// TODO(4): `return` 으로 끝나는 if/else 사슬을 "마지막 표현식" 으로 바꾸세요. (clippy: needless_return)
//   Rust 에서는 블록의 마지막 줄(세미콜론 없는 표현식)이 곧 반환값입니다. 파일 전체의 `return x;` (함수 맨 끝) 도 같이 정리하세요.
//   describe_temperature 는 `match t { t if t < 0.3 => "precise", t if t < 0.8 => "balanced", _ => "creative" }` 모양으로.
//   🐍 Python: if t < 0.3: return "precise" ...                        (python/main.py: describe_temperature)
//   ⚙️ C++  : if (t < 0.3) return "precise"; ...                       (cpp/main.cpp: describe_temperature)
//   🦀 Rust : `if`/`match` 는 값을 돌려주는 표현식입니다. (Ch02 복습)
// ─────────────────────────────────────────────
pub fn describe_temperature(t: f64) -> &'static str {
    if t < 0.3 {
        return "precise";
    } else if t < 0.8 {
        return "balanced";
    } else {
        return "creative";
    }
}

// ─────────────────────────────────────────────
// TODO(1)+(2) 의 마무리: report. 복사(clone) 후 직접 만든 버블 정렬을 `Vec<&AgentConfig>` + `sort_by` 로,
//   합계와 최댓값을 이터레이터로 바꾸세요. (원본 `configs` 의 순서는 바뀌면 안 됩니다 — 테스트가 확인해요)
//   🐍 Python: sorted(configs, key=lambda c: c.name), max(configs, key=...)    (python/main.py: report)
//   ⚙️ C++  : std::ranges::sort(configs, {}, &AgentConfig::name)               (cpp/main.cpp: report)
//   🦀 Rust : `let mut sorted: Vec<&AgentConfig> = configs.iter().collect(); sorted.sort_by(|a, b| a.name.cmp(&b.name));`
//   💡 힌트: 행들을 `Vec<String>` 에 모았다가 마지막에 `lines.join("\n")`.  최댓값은 `max_by_key(|c| c.max_tokens)` 가 `Option` 을 돌려줘요.
//   ❓ 왜?: `.clone()` 으로 전체를 복사하면 비용이 들어요. 참조(`&AgentConfig`)만 모아 정렬하면 복사가 없습니다.
// ─────────────────────────────────────────────
pub fn report(configs: &Vec<AgentConfig>) -> String {
    let mut sorted: Vec<AgentConfig> = configs.clone();
    for i in 0..sorted.len() {
        for j in 0..sorted.len() - 1 - i {
            if sorted[j].name > sorted[j + 1].name {
                sorted.swap(j, j + 1);
            }
        }
    }
    let mut out = String::new();
    out.push_str(&format!("{:<8}|{:>7}|{:>6}\n", "name", "tokens", "temp"));
    for c in sorted.iter() {
        out.push_str(&format!("{:<8}|{:>7}|{:>6.2}\n", c.name, c.max_tokens, c.temperature));
    }
    let mut total: u32 = 0;
    for c in configs.iter() {
        total = total + c.max_tokens;
    }
    out.push_str(&format!("total tokens: {}\n", total));
    let mut best: Option<&AgentConfig> = None;
    for c in configs.iter() {
        if best.is_none() || c.max_tokens > best.unwrap().max_tokens {
            best = Some(c);
        }
    }
    if best.is_some() {
        let b = best.unwrap();
        out.push_str(&format!("largest: {} ({})", b.name, b.max_tokens));
    }
    return out;
}

fn show_opt<T: fmt::Display>(v: Option<T>) -> String {
    v.map_or_else(|| "none".to_string(), |x| x.to_string())
}

// ─────────────────────────────────────────────
// TODO(11): main 을 새 API 로 옮기세요. 출력은 한 글자도 바뀌면 안 됩니다.
//   - `AgentConfig::new(8개 인자)` 호출들 → `AgentConfig::builder("helper").model(..)...build()`
//   - `parse_config(&spec.to_string())` → `AgentConfig::try_from(*spec)` (또는 `spec.try_into()`)
//   - 에러 출력은 `{e}` (ConfigError 의 Display)
//   - `println!("{}", c.to_string())` → `println!("{c}")`
//   다 옮겼으면 `AgentConfig::new` 와 `parse_config` 를 삭제하세요. 그러면 too_many_arguments 같은 경고가 사라집니다.
//   💡 `.unwrap()`/`.expect()` 는 main 의 "절대 실패할 수 없는 고정 입력" 에서만 괜찮습니다. 일반 코드에서는 `?` 로 전파하세요.
//   🐍 Python / ⚙️ C++ main: 이미 빌더 스타일입니다 (python/main.py: main, cpp/main.cpp: main).
// ─────────────────────────────────────────────
fn main() {
    println!("== 1. builder ==");
    let helper = AgentConfig::new(
        "helper".to_string(),
        "claude-sonnet".to_string(),
        1024,
        0.7,
        vec!["search".to_string(), "calc".to_string()],
        Some("You are a helpful assistant.".to_string()),
        2,
        30,
    )
    .unwrap();
    let writer = AgentConfig::new("writer".to_string(), "claude-opus".to_string(), 4096, 0.3, Vec::new(), None, 1, 60).unwrap();
    let coder = AgentConfig::new(
        "coder".to_string(),
        "default-model".to_string(),
        1024,
        1.0,
        vec!["shell".to_string()],
        Some("Write code.".to_string()),
        2,
        30,
    )
    .unwrap();
    println!("{}", helper.to_string());
    println!("{}", writer.to_string());
    println!("{}", coder.to_string());

    println!("== 2. validation ==");
    let mk = |tokens: u32, temp: f64, retries: u8, tools: Vec<&str>, name: &str| {
        let tools: Vec<String> = tools.iter().map(|t| t.to_string()).collect();
        AgentConfig::new(name.to_string(), "default-model".to_string(), tokens, temp, tools, None, retries, 30)
    };
    let cases = vec![
        ("empty name", mk(1024, 1.0, 2, vec![], "")),
        ("hot temperature", mk(1024, 3.5, 2, vec![], "a")),
        ("zero tokens", mk(0, 1.0, 2, vec![], "a")),
        ("too many retries", mk(1024, 1.0, 9, vec![], "a")),
        ("duplicate tool", mk(1024, 1.0, 2, vec!["search", "calc", "search"], "a")),
    ];
    for (label, result) in cases.iter() {
        match result {
            Ok(_) => println!("  {}: ok", label),
            Err(e) => println!("  {}: error: {}", label, e),
        }
    }

    println!("== 3. parse ==");
    let specs = vec![
        "name=writer;model=claude-opus;max_tokens=2048;temperature=0.3;tools=search,calc;prompt=Be brief",
        "name=scribe",
        "name=x; max_tokens=abc",
        "model=m1",
        "name=x;color=red",
        "name=x;oops",
        "name=x;temperature=9",
    ];
    for i in 0..specs.len() {
        match parse_config(&specs[i].to_string()) {
            Ok(c) => println!("  [{}] ok: {}", i + 1, c.to_string()),
            Err(e) => println!("  [{}] error: {}", i + 1, e),
        }
    }

    println!("== 4. utilities ==");
    let tools: Vec<String> = vec![
        "search".to_string(),
        "calc".to_string(),
        "shell".to_string(),
        "web_browser_long".to_string(),
    ];
    println!("join_tools: {}", join_tools(&tools));
    println!("count_long_names(>=5): {}", count_long_names(&tools, 5));
    println!("find_tool shell: {}", show_opt(find_tool(&tools, &"shell".to_string())));
    println!("find_tool python: {}", show_opt(find_tool(&tools, &"python".to_string())));
    println!("prompt_len helper: {}", prompt_len(&helper));
    println!("prompt_len writer: {}", prompt_len(&writer));
    println!("first_tool_upper helper: {}", show_opt(first_tool_upper(&helper)));
    println!("first_tool_upper writer: {}", show_opt(first_tool_upper(&writer)));
    for t in [0.0, 0.5, 1.5] {
        println!("temperature {:.1}: {}", t, describe_temperature(t));
    }

    println!("== 5. report ==");
    println!("{}", report(&vec![helper, writer, coder]));
}

// ═════════════════════════════════════════════
// TODO(12): 마무리
//   1) `cargo fmt -p ch12_idiomatic_rust`  (rustfmt: 스타일 논쟁 종료)
//   2) 맨 위 `#![allow(...)]` 줄 삭제 → 경고(안 쓰는 import/변수/함수)를 정리
//   3) `cargo clippy -p ch12_idiomatic_rust -- -D warnings`  경고 0개,  `cargo test` 전부 통과(ignored 0)
//   4) 🔬 실험: 일부러 `let v: Vec<i32> = Vec::new(); if v.len() == 0 {}` 를 한 줄 넣고 clippy 가 뭐라고 하는지 보세요.
//      README 의 "알아 둘 clippy 린트" 표와 비교해 보세요.
// ═════════════════════════════════════════════

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

    // ── 아래 4개는 새 API(TODO 7~10)를 확인합니다. 처음엔 #[ignore] 로 건너뛰고,
    //    해당 TODO 를 끝낼 때마다 #[ignore] 줄을 지우고 `cargo test` 하세요. ──

    #[test]
    #[ignore = "TODO(8): Temperature 를 구현하면 #[ignore] 를 지우세요"]
    fn temperature_try_from() {
        assert_eq!(Temperature::try_from(0.7).unwrap().get(), 0.7);
        assert_eq!(Temperature::try_from(2.5).unwrap_err().to_string(), "temperature 2.5 out of range 0.0..=2.0");
        assert!(Temperature::try_from(f64::NAN).is_err());
    }

    #[test]
    #[ignore = "TODO(9): 빌더를 구현하면 #[ignore] 를 지우세요"]
    fn builder_defaults_and_chain() {
        let c = AgentConfig::builder("bot").model("m").tool("a").tools(["b", "c"]).system_prompt("hi").build().unwrap();
        assert_eq!(c.name, "bot");
        assert_eq!(c.tools, vec!["a", "b", "c"]);
        assert_eq!(c.max_tokens, 1024);
        assert_eq!(c.system_prompt.as_deref(), Some("hi"));
        assert_eq!(AgentConfig::default().retries, 2);
    }

    #[test]
    #[ignore = "TODO(7)+(9): ConfigError 와 빌더를 구현하면 #[ignore] 를 지우세요"]
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
    #[ignore = "TODO(10): TryFrom<&str> 를 구현하면 #[ignore] 를 지우세요"]
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
