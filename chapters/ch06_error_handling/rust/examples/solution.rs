//! Ch06 정답: 설정 파일 파서
//!
//! 실행: `cargo run -q -p ch06_error_handling --example solution`
//! 테스트: `cargo test -p ch06_error_handling --example solution`

use std::fmt;
use std::fs;
use std::io;

// ── 타입 정의 ───────────────────────────────────────────────

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
    MissingEquals {
        line: usize,
    },
    UnknownKey {
        line: usize,
        key: String,
    },
    DuplicateKey {
        line: usize,
        key: String,
    },
    InvalidValue {
        line: usize,
        key: String,
        reason: String,
    },
    MissingKey(String),
    Io(io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingEquals { line } => write!(f, "line {line}: expected 'key = value'"),
            ConfigError::UnknownKey { line, key } => write!(f, "line {line}: unknown key '{key}'"),
            ConfigError::DuplicateKey { line, key } => {
                write!(f, "line {line}: duplicate key '{key}'")
            }
            ConfigError::InvalidValue { line, key, reason } => {
                write!(f, "line {line}: invalid value for '{key}': {reason}")
            }
            ConfigError::MissingKey(key) => write!(f, "missing required key '{key}'"),
            ConfigError::Io(_) => write!(f, "cannot read config file"),
        }
    }
}

impl std::error::Error for ConfigError {
    /// 원인(cause) 체인: Io 에러라면 안쪽의 io::Error 를 노출합니다.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// `From` 이 있으면 `?` 가 io::Error 를 ConfigError 로 자동 변환해 줍니다.
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
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
    (
        "retries too big",
        "name = x\nport = 80\nmax_retries = 300\n",
    ),
    ("missing name", "# only a port\nport = 80\n"),
    ("duplicate", "name = a\nname = b\n"),
    ("empty name", "name =\nport = 1\n"),
];

// ── 구현 ────────────────────────────────────────────────────

fn invalid(line: usize, key: &str, reason: &str) -> ConfigError {
    ConfigError::InvalidValue {
        line,
        key: key.to_string(),
        reason: reason.to_string(),
    }
}

/// 한 줄을 (key, value) 로 나눕니다. 빈 줄/주석은 `Ok(None)`.
fn parse_line(line_no: usize, raw: &str) -> Result<Option<(String, String)>, ConfigError> {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let (key, value) = line
        .split_once('=')
        .ok_or(ConfigError::MissingEquals { line: line_no })?;
    Ok(Some((key.trim().to_string(), value.trim().to_string())))
}

/// 0..=max 범위의 10진수 문자열을 읽습니다.
fn parse_number(line: usize, key: &str, value: &str, max: u64) -> Result<u64, ConfigError> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(line, key, "not a valid number"));
    }
    let range_msg = format!("out of range (0-{max})");
    // 숫자만 있는데 u64 에도 안 들어가면 "범위 초과" 입니다.
    let n: u64 = value.parse().map_err(|_| invalid(line, key, &range_msg))?;
    if n > max {
        return Err(invalid(line, key, &range_msg));
    }
    Ok(n)
}

fn parse_bool(line: usize, key: &str, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(invalid(line, key, "expected true or false")),
    }
}

fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let mut name: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut debug: Option<bool> = None;
    let mut max_retries: Option<u8> = None;

    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        // `?` : Err 이면 이 함수에서 즉시 return Err(...) 합니다.
        let Some((key, value)) = parse_line(line_no, raw)? else {
            continue;
        };
        let dup = || ConfigError::DuplicateKey {
            line: line_no,
            key: key.clone(),
        };
        match key.as_str() {
            "name" => {
                if name.is_some() {
                    return Err(dup());
                }
                if value.is_empty() {
                    return Err(invalid(line_no, &key, "must not be empty"));
                }
                name = Some(value);
            }
            "port" => {
                if port.is_some() {
                    return Err(dup());
                }
                port = Some(parse_number(line_no, &key, &value, u16::MAX as u64)? as u16);
            }
            "debug" => {
                if debug.is_some() {
                    return Err(dup());
                }
                debug = Some(parse_bool(line_no, &key, &value)?);
            }
            "max_retries" => {
                if max_retries.is_some() {
                    return Err(dup());
                }
                max_retries = Some(parse_number(line_no, &key, &value, u8::MAX as u64)? as u8);
            }
            _ => {
                return Err(ConfigError::UnknownKey { line: line_no, key });
            }
        }
    }

    Ok(Config {
        // Option -> Result 변환: 없으면 MissingKey 에러
        name: name.ok_or_else(|| ConfigError::MissingKey("name".to_string()))?,
        port: port.ok_or_else(|| ConfigError::MissingKey("port".to_string()))?,
        debug: debug.unwrap_or(false),
        max_retries: max_retries.unwrap_or(3),
    })
}

/// 파일을 읽어 파싱합니다. read_to_string 의 io::Error 는 `?` 가 From 으로 변환합니다.
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let text = fs::read_to_string(path)?;
    parse_config(&text)
}

/// 파싱에 실패하면 기본 포트 8080 을 씁니다. (콤비네이터 연습)
fn port_or_default(text: &str) -> u16 {
    parse_config(text).map(|c| c.port).unwrap_or(8080)
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
