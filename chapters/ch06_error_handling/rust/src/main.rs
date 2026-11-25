//! # Ch06. 에러 처리 — 설정 파일 파서
//!
//! ## 목표
//! - panic 과 `Result` 의 역할 차이를 이해하고, 예상 가능한 실패는 `Result` 로 돌려준다.
//! - `?` 연산자, `Option`/`Result` 콤비네이터(`ok_or`, `map_err`, `unwrap_or` ...)를 쓴다.
//! - 커스텀 에러 enum + `Display` + `std::error::Error` + `From` 변환을 직접 구현한다.
//!
//! ## 대응 파일
//! - Python : `../python/main.py`  (예외 클래스 계층)
//! - C++    : `../cpp/main.cpp`    (예외 + std::optional)
//!
//! ## 진행 방법
//! 1. `TODO(1)` 부터 번호 순서대로 `todo!()` 를 구현합니다. 위에서 아래로 갈수록 앞의 것을 재사용합니다.
//! 2. `cargo test -p ch06_error_handling` 로 중간 확인.
//! 3. 프로그램은 `data/app.conf` 를 읽으므로 반드시 **챕터 디렉터리**(`chapters/ch06_error_handling`)에서
//!    `cargo run -q -p ch06_error_handling` 으로 실행하세요.
//!
//! ## 완료 기준
//! `cargo test -p ch06_error_handling` 이 모두 초록불이고
//! `../../scripts/compare.sh ch06` 에서 세 언어의 출력이 같으면 끝입니다.

// 다 풀고 나면 이 줄을 지워 보세요 (아직 쓰지 않은 코드에 대한 경고를 잠시 꺼 둡니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::fmt;
use std::fs;
use std::io;

// ── 타입 정의 (제공됨) ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct Config {
    name: String,
    port: u16,
    debug: bool,
    max_retries: u8,
}

/// 이 프로그램에서 일어날 수 있는 "예상 가능한 실패"를 전부 한 enum 에 모았습니다.
#[derive(Debug)]
enum ConfigError {
    MissingEquals { line: usize },
    UnknownKey { line: usize, key: String },
    DuplicateKey { line: usize, key: String },
    InvalidValue { line: usize, key: String, reason: String },
    MissingKey(String),
    Io(io::Error),
}

// ─────────────────────────────────────────────
// TODO(1): 사용자에게 보여 줄 에러 메시지(Display)를 variant 별로 만드세요.
//   MissingEquals  -> "line 2: expected 'key = value'"
//   UnknownKey     -> "line 3: unknown key 'colour'"
//   DuplicateKey   -> "line 2: duplicate key 'name'"
//   InvalidValue   -> "line 2: invalid value for 'port': out of range (0-65535)"
//   MissingKey     -> "missing required key 'name'"
//   Io             -> "cannot read config file"
//   🐍 Python: class MissingEquals(ConfigError): super().__init__(f"...")   (python/main.py)
//   ⚙️ C++  : std::runtime_error(std::format(...))                          (cpp/main.cpp: missing_equals ...)
//   🦀 Rust : `match self { ... }` 안에서 `write!(f, "...", ...)`.
//   💡 힌트: `ConfigError::UnknownKey { line, key } => write!(f, "line {line}: unknown key '{key}'")`
//           (`{line}` 처럼 변수 이름을 문자열 안에 바로 쓸 수 있어요.)
//   ❓ 왜?: Debug 는 개발자용, Display 는 사용자용입니다. `{e}` 는 Display, `{e:?}` 는 Debug 를 씁니다.
// ─────────────────────────────────────────────
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("TODO(1): variant 별 메시지")
    }
}

// ─────────────────────────────────────────────
// TODO(2): `Error` 트레이트의 `source()` — "이 에러의 원인"을 돌려줍니다.
//   Io(e) 라면 Some(e), 나머지는 None.
//   🐍 Python: raise ConfigIoError() from e   (__cause__)              (python/main.py: load_config)
//   ⚙️ C++  : std::throw_with_nested (거의 쓰지 않음)
//   🦀 Rust : 에러 체인(원인의 원인...)을 따라갈 수 있게 해 주는 표준 방법입니다.
//   💡 힌트: 반환 타입이 `Option<&(dyn std::error::Error + 'static)>` 로 길지만, `Some(e)` 라고만 쓰면 됩니다.
//   ❓ 왜?: anyhow 같은 라이브러리(ch16)가 이 체인을 "Caused by:" 로 예쁘게 출력해 줍니다.
// ─────────────────────────────────────────────
impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!("TODO(2): Io 일 때만 Some(e)")
    }
}

// ─────────────────────────────────────────────
// TODO(3): io::Error -> ConfigError 변환. 이게 있으면 `?` 가 자동으로 변환해 줍니다.
//   🐍 Python: except OSError as e: raise ConfigIoError() from e        (python/main.py: load_config)
//   ⚙️ C++  : catch 해서 다시 throw ConfigError("...")                   (cpp/main.cpp: load_config)
//   🦀 Rust : `impl From<io::Error> for ConfigError { fn from(e) -> Self { ... } }`
//   💡 힌트: 한 줄이면 됩니다. `ConfigError::Io(e)`.
//   ❓ 왜?: `?` 는 에러를 함수의 반환 에러 타입으로 `From::from` 변환한 뒤 return 합니다.
// ─────────────────────────────────────────────
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        todo!("TODO(3): ConfigError::Io 로 감싸기")
    }
}

// ── 제공되는 코드 ───────────────────────────────────────────

fn show(cfg: &Config) -> String {
    format!(
        "name={} port={} debug={} max_retries={}",
        cfg.name, cfg.port, cfg.debug, cfg.max_retries
    )
}

const BAD_INPUTS: [(&str, &str); 10] = [
    ("no equals", "name = x\nport 80\n"),
    ("unknown key", "name = x\nport = 80\ncolour = red\n"),
    ("port too big", "name = x\nport = 70000\n"),
    ("port not number", "name = x\nport = eighty\n"),
    ("negative port", "name = x\nport = -1\n"),
    ("bad bool", "name = x\nport = 80\ndebug = maybe\n"),
    ("retries too big", "name = x\nport = 80\nmax_retries = 300\n"),
    ("missing name", "# only a port\nport = 80\n"),
    ("duplicate", "name = a\nname = b\n"),
    ("empty name", "name =\nport = 1\n"),
];

/// InvalidValue 를 짧게 만드는 도우미 (제공됨).
fn invalid(line: usize, key: &str, reason: &str) -> ConfigError {
    ConfigError::InvalidValue {
        line,
        key: key.to_string(),
        reason: reason.to_string(),
    }
}

// ── 구현할 부분 ─────────────────────────────────────────────

// ─────────────────────────────────────────────
// TODO(4): 한 줄을 (key, value) 로 나눕니다 (둘 다 앞뒤 공백 제거).
//   - 빈 줄, `#` 로 시작하는 줄(앞 공백 무시) => Ok(None)
//   - `=` 가 없으면 => Err(ConfigError::MissingEquals { line: line_no })
//   - `=` 가 여러 개면 첫 번째에서만 나눕니다. ("name = a=b" => value 는 "a=b")
//   🐍 Python: line.partition("=")                         (python/main.py: parse_line)
//   ⚙️ C++  : line.find('=') + substr                       (cpp/main.cpp: parse_line)
//   🦀 Rust : `Result<Option<(String, String)>, ConfigError>` — "에러일 수도, 값이 없을 수도"를 겹쳐서 표현.
//   💡 힌트: `str::split_once('=')` 는 `Option<(&str, &str)>` 를 돌려줍니다.
//           `.ok_or(에러값)?` 로 Option 을 Result 로 바꾸고 `?` 로 조기 반환할 수 있어요.
//   ❓ 왜?: "값이 없음(None)"과 "잘못됨(Err)"은 다른 사건이라 타입 수준에서 분리합니다.
// ─────────────────────────────────────────────
fn parse_line(line_no: usize, raw: &str) -> Result<Option<(String, String)>, ConfigError> {
    todo!("TODO(4): 주석/빈 줄/키-값 분리")
}

// ─────────────────────────────────────────────
// TODO(5): 0..=max 범위의 10진수 문자열을 u64 로 읽습니다.
//   - 비었거나 숫자(0-9)가 아닌 글자가 있으면(예: "-1", "eighty") => invalid(line, key, "not a valid number")
//   - max 를 넘으면(아주 긴 숫자 포함) => invalid(line, key, &format!("out of range (0-{max})"))
//   🐍 Python: value.isdigit(), int(value)                  (python/main.py: parse_number)
//   ⚙️ C++  : std::all_of + std::stoll                      (cpp/main.cpp: parse_number)
//   🦀 Rust : `value.parse::<u64>()` 는 `Result<u64, ParseIntError>` 를 돌려줍니다.
//   💡 힌트: 먼저 `value.bytes().all(|b| b.is_ascii_digit())` 로 모양을 검사하고,
//           parse 의 에러는 `.map_err(|_| invalid(...))?` 로 우리 에러 타입으로 바꿉니다.
//   ❓ 왜?: `map_err` 는 "에러의 종류만 바꾸고 성공 값은 그대로" 두는 콤비네이터입니다.
// ─────────────────────────────────────────────
fn parse_number(line: usize, key: &str, value: &str, max: u64) -> Result<u64, ConfigError> {
    todo!("TODO(5): 검증 + parse + 범위 검사")
}

// ─────────────────────────────────────────────
// TODO(6): "true" / "false" 만 허용합니다. 그 외(대소문자 다른 "True" 포함)는
//   invalid(line, key, "expected true or false").
//   🐍 Python: if value == "true": ...                      (python/main.py: parse_bool)
//   ⚙️ C++  : if (value == "true") ...                      (cpp/main.cpp: parse_bool)
//   🦀 Rust : `match value { "true" => Ok(true), ... }`
//   💡 힌트: 문자열도 match 할 수 있고, 마지막 팔은 `_ => Err(...)`.
//   ❓ 왜?: 느슨한 파싱("yes", "1" 도 허용)은 나중에 버그가 됩니다. 처음부터 엄격하게!
// ─────────────────────────────────────────────
fn parse_bool(line: usize, key: &str, value: &str) -> Result<bool, ConfigError> {
    todo!("TODO(6): match 로 bool 파싱")
}

// ─────────────────────────────────────────────
// TODO(7): 전체 텍스트를 Config 로 파싱합니다.
//   규칙: 키는 name(비어 있으면 안 됨, 메시지 "must not be empty"), port(0..=65535),
//         debug(기본 false), max_retries(0..=255, 기본 3). 같은 키가 두 번이면 DuplicateKey(두 번째 줄 번호),
//         모르는 키는 UnknownKey. name/port 가 끝까지 없으면 MissingKey("name") / MissingKey("port").
//   🐍 Python: raise ... / 변수 4개를 None 으로 시작      (python/main.py: parse_config)
//   ⚙️ C++  : std::optional 4개 + throw                   (cpp/main.cpp: parse_config)
//   🦀 Rust : `let mut name: Option<String> = None;` 처럼 시작해서 줄마다 채워 넣습니다.
//   💡 힌트: - `for (idx, raw) in text.lines().enumerate()` 의 줄 번호는 idx + 1.
//           - `let Some((key, value)) = parse_line(line_no, raw)? else { continue; };`
//           - `match key.as_str() { "name" => ..., "port" => ..., _ => return Err(UnknownKey ...) }`
//           - u16/u8 변환은 `parse_number(..., u16::MAX as u64)? as u16` (이미 범위를 검사했으니 안전).
//           - 마지막에 `name.ok_or_else(|| ConfigError::MissingKey("name".to_string()))?`,
//             `debug.unwrap_or(false)`, `max_retries.unwrap_or(3)`.
//   ❓ 왜?: 에러가 나면 첫 에러에서 바로 `return Err(..)` — 호출한 쪽이 반드시 처리해야 하는 값이 됩니다.
// ─────────────────────────────────────────────
fn parse_config(text: &str) -> Result<Config, ConfigError> {
    todo!("TODO(7): 줄 단위로 파싱해 Config 만들기")
}

// ─────────────────────────────────────────────
// TODO(8): 파일을 읽어 파싱합니다. 두 줄이면 됩니다.
//   🐍 Python: open(path) ... except OSError                (python/main.py: load_config)
//   ⚙️ C++  : std::ifstream + rdbuf                         (cpp/main.cpp: load_config)
//   🦀 Rust : `let text = fs::read_to_string(path)?;` — io::Error 가 TODO(3) 의 From 으로 자동 변환됩니다.
//   💡 힌트: 이어서 `parse_config(&text)` 를 그대로 반환.
//   ❓ 왜?: `?` 한 글자가 C++ 의 try/catch/rethrow 를 대신합니다. 하지만 "여기서 실패할 수 있다"는 점은 눈에 보입니다.
// ─────────────────────────────────────────────
fn load_config(path: &str) -> Result<Config, ConfigError> {
    todo!("TODO(8): read_to_string + parse_config")
}

// ─────────────────────────────────────────────
// TODO(9): 파싱에 실패하면 기본 포트 8080 을 돌려줍니다. (콤비네이터 연습 — if/match 없이 한 줄로!)
//   🐍 Python: try: ... except ConfigError: return 8080      (python/main.py: port_or_default)
//   ⚙️ C++  : try { ... } catch (const ConfigError&) { ... } (cpp/main.cpp: port_or_default)
//   🦀 Rust : `parse_config(text).map(|c| c.port).unwrap_or(8080)`
//   💡 힌트: `Result::map` 은 Ok 안의 값만 바꾸고, `unwrap_or(기본값)` 은 Err 를 기본값으로 대체합니다.
//   ❓ 왜?: 에러 내용을 버려도 되는 곳에서는 이렇게 "의도적으로" 버립니다. 조용히 삼키는 게 아니라 코드에 드러나요.
// ─────────────────────────────────────────────
fn port_or_default(text: &str) -> u16 {
    todo!("TODO(9): map + unwrap_or")
}

fn main() {
    match load_config("data/app.conf") {
        Ok(cfg) => println!("loaded: {}", show(&cfg)),
        Err(e) => println!("error: {e}"),
    }
    match load_config("data/missing.conf") {
        Ok(cfg) => println!("loaded: {}", show(&cfg)),
        Err(e) => println!("error: {e}"),
    }

    println!("-- bad inputs --");
    for (label, text) in BAD_INPUTS {
        match parse_config(text) {
            Ok(cfg) => println!("{label:<16} ok?! {}", show(&cfg)),
            Err(e) => println!("{label:<16} error: {e}"),
        }
    }

    println!("-- defaults --");
    let minimal = "name = tiny\nport = 9000\n";
    match parse_config(minimal) {
        Ok(cfg) => println!("minimal: {}", show(&cfg)),
        Err(e) => println!("error: {e}"),
    }
    println!("port_or_default(good) = {}", port_or_default(minimal));
    println!("port_or_default(bad)  = {}", port_or_default("port = x"));

    // 🔬 실험 1: 아래 함수의 주석을 풀어 보세요. (기대: E0277 `?` couldn't convert the error to `ConfigError`)
    //   TODO(3) 의 `From<io::Error>` 가 없으면 `?` 가 변환 방법을 몰라서 컴파일이 안 됩니다.
    //   (TODO(3) 를 구현한 상태에서는 오히려 컴파일됩니다. 일부러 `impl From` 블록을 잠시 주석 처리해 보세요.)
    // fn experiment_question_mark() -> Result<String, ConfigError> {
    //     let text = fs::read_to_string("data/app.conf")?;
    //     Ok(text)
    // }

    // 🔬 실험 2: Result 를 무시해 보세요. `parse_config(minimal);` (세미콜론만 붙여 결과를 버림)
    //   기대: warning: unused `Result` that must be used — Python 의 "조용히 무시되는 예외"와 달리
    //   컴파일러가 경고해 줍니다. ❓ 에러를 정말 버리고 싶다면 어떻게 써야 의도가 드러날까요? (`let _ = ...`)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_text(text: &str) -> String {
        parse_config(text).unwrap_err().to_string()
    }

    #[test]
    fn parse_line_skips_blank_and_comments() {
        assert!(matches!(parse_line(1, ""), Ok(None)));
        assert!(matches!(parse_line(2, "   # note"), Ok(None)));
    }

    #[test]
    fn parse_line_splits_on_first_equals() {
        let (k, v) = parse_line(1, "  name =  a=b  ").unwrap().unwrap();
        assert_eq!((k.as_str(), v.as_str()), ("name", "a=b"));
        let e = parse_line(7, "oops").unwrap_err();
        assert_eq!(e.to_string(), "line 7: expected 'key = value'");
    }

    #[test]
    fn parse_number_ranges() {
        assert_eq!(parse_number(1, "port", "65535", 65535).unwrap(), 65535);
        assert_eq!(parse_number(1, "port", "0", 65535).unwrap(), 0);
        let e = parse_number(4, "port", "65536", 65535).unwrap_err();
        assert_eq!(
            e.to_string(),
            "line 4: invalid value for 'port': out of range (0-65535)"
        );
        let e = parse_number(4, "port", "99999999999999999999999", 65535).unwrap_err();
        assert!(e.to_string().ends_with("out of range (0-65535)"));
        let e = parse_number(4, "port", "-1", 65535).unwrap_err();
        assert!(e.to_string().ends_with("not a valid number"));
        assert!(parse_number(1, "port", "", 65535).is_err());
    }

    #[test]
    fn parse_bool_strict() {
        assert_eq!(parse_bool(1, "debug", "true").unwrap(), true);
        assert_eq!(parse_bool(1, "debug", "false").unwrap(), false);
        assert!(parse_bool(1, "debug", "True").is_err());
    }

    #[test]
    fn full_config_and_defaults() {
        let cfg = parse_config("name = a\nport = 1\n").unwrap();
        assert_eq!(
            cfg,
            Config {
                name: "a".into(),
                port: 1,
                debug: false,
                max_retries: 3
            }
        );
        let cfg = parse_config("# c\nname = b\nport = 2\ndebug = true\nmax_retries = 9\n").unwrap();
        assert_eq!(
            cfg,
            Config {
                name: "b".into(),
                port: 2,
                debug: true,
                max_retries: 9
            }
        );
    }

    #[test]
    fn error_messages_have_line_numbers() {
        assert_eq!(
            err_text("name = x\nport 80\n"),
            "line 2: expected 'key = value'"
        );
        assert_eq!(
            err_text("name = x\ncolour = red\n"),
            "line 2: unknown key 'colour'"
        );
        assert_eq!(
            err_text("name = a\nname = b\n"),
            "line 2: duplicate key 'name'"
        );
        assert_eq!(
            err_text("name =\nport = 1\n"),
            "line 1: invalid value for 'name': must not be empty"
        );
    }

    #[test]
    fn missing_required_keys() {
        assert_eq!(err_text("port = 80\n"), "missing required key 'name'");
        assert_eq!(err_text("name = x\n"), "missing required key 'port'");
    }

    #[test]
    fn io_error_is_converted_with_from() {
        let e = load_config("definitely/not/here.conf").unwrap_err();
        assert!(matches!(e, ConfigError::Io(_)));
        assert_eq!(e.to_string(), "cannot read config file");
        use std::error::Error;
        assert!(e.source().is_some());
    }

    #[test]
    fn port_default_combinator() {
        assert_eq!(port_or_default("name = a\nport = 1234\n"), 1234);
        assert_eq!(port_or_default("garbage"), 8080);
    }
}
