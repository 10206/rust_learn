//! Ch09 정답: 제로카피 로그 파서
//!
//! 실행: cargo run -q -p ch09_lifetimes --example solution   (챕터 폴더에서)
//! 테스트: cargo test -p ch09_lifetimes --example solution

use std::collections::BTreeMap;
use std::fmt;
use std::fs;

// 문자열을 복사하지 않고 원본 텍스트의 "일부"를 가리키는 구조체.
// 'a = "이 구조체는 원본 텍스트보다 오래 살 수 없다"는 관계를 컴파일러에게 알려 주는 이름표입니다.
#[derive(Debug, Clone, Copy, PartialEq)]
struct LogEntry<'a> {
    level: &'a str,
    module: &'a str,
    message: &'a str,
}

// impl 쪽에서 이름이 필요 없으면 '_ (익명 라이프타임)을 씁니다.
impl fmt::Display for LogEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.level, self.module, self.message)
    }
}

const LEVELS: [&str; 4] = ["DEBUG", "INFO", "WARN", "ERROR"];

// 생략 규칙(elision): 입력 참조가 하나뿐이면 출력 참조는 같은 라이프타임.
// 아래는 fn parse_line<'a>(line: &'a str) -> Option<LogEntry<'a>> 와 같은 뜻입니다.
fn parse_line(line: &str) -> Option<LogEntry<'_>> {
    let mut parts = line.splitn(4, ' ');
    let _date = parts.next()?;
    let _time = parts.next()?;
    let level = parts.next()?;
    let rest = parts.next()?;
    if !LEVELS.contains(&level) {
        return None;
    }
    let (module, message) = rest.split_once(": ")?;
    if module.is_empty() || module.contains(' ') || message.is_empty() {
        return None;
    }
    Some(LogEntry {
        level,
        module,
        message,
    })
}

fn parse_log<'a>(text: &'a str) -> (Vec<LogEntry<'a>>, usize) {
    let mut entries = Vec::new();
    let mut skipped = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match parse_line(line) {
            Some(entry) => entries.push(entry),
            None => skipped += 1,
        }
    }
    (entries, skipped)
}

// level 은 이 함수 안에서만 쓰고 결과에는 안 들어가므로 별도(생략된) 라이프타임이면 충분합니다.
// 결과 안의 &str 은 원본 텍스트('a)를 가리키므로 'a 를 그대로 이어 줍니다.
fn filter_by_level<'a>(entries: &[LogEntry<'a>], level: &str) -> Vec<LogEntry<'a>> {
    entries.iter().filter(|e| e.level == level).copied().collect()
}

// 반환값은 "entries 슬라이스"가 아니라 "원본 텍스트('a)"를 빌립니다.
// 그래서 entries Vec 이 사라져도 반환한 &str 은 계속 쓸 수 있어요.
fn longest_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str> {
    entries.iter().map(|e| e.message).max_by_key(|m| m.len())
}

fn count_by_module<'a>(entries: &[LogEntry<'a>]) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for e in entries {
        *counts.entry(e.module).or_insert(0) += 1;
    }
    counts
}

// 이미 완성된 예시: 같은 모양을 level 에 적용
fn count_by_level<'a>(entries: &[LogEntry<'a>]) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for e in entries {
        *counts.entry(e.level).or_insert(0) += 1;
    }
    counts
}

// 두 입력과 출력이 모두 같은 'a: "결과는 둘 중 더 짧게 사는 쪽만큼만 유효하다"
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    if b.len() > a.len() { b } else { a }
}

fn main() {
    let text = fs::read_to_string("data/app.log").expect("data/app.log 를 읽을 수 없어요");
    // entries 안의 모든 &str 은 이 text 를 가리킵니다. text 가 먼저 사라지면 컴파일 에러!
    let (entries, skipped) = parse_log(&text);
    println!("parsed {} entries, skipped {} malformed lines", entries.len(), skipped);

    for level in ["ERROR", "WARN"] {
        println!("== {level} entries ==");
        for e in filter_by_level(&entries, level) {
            println!("{e}");
        }
    }

    println!("== longest message ==");
    if let Some(m) = longest_message(&entries) {
        println!("{m} ({} bytes)", m.len());
    }

    println!("== per module ==");
    for (module, n) in count_by_module(&entries) {
        println!("{module}: {n}");
    }

    println!("== per level ==");
    for (level, n) in count_by_level(&entries) {
        println!("{level}: {n}");
    }

    println!("== longer ==");
    println!("longer(\"abc\", \"de\") = {}", longer("abc", "de"));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
2024-05-01 10:00:01 INFO auth: user alice logged in
bad line here
2024-05-01 10:00:02 ERROR db: query timeout after 30s

2024-05-01 10:00:03 INFO auth: ok
2024-05-01 10:00:04 WARN api: slow
2024-05-01 10:00:05 FATAL api: unknown level
2024-05-01 10:00:06 INFO api missing colon
";

    #[test]
    fn parse_line_ok() {
        let e = parse_line("2024-05-01 10:00:01 INFO auth: user alice logged in").unwrap();
        assert_eq!(e.level, "INFO");
        assert_eq!(e.module, "auth");
        assert_eq!(e.message, "user alice logged in");
    }

    #[test]
    fn parse_line_rejects_bad_input() {
        assert!(parse_line("").is_none());
        assert!(parse_line("only two").is_none());
        assert!(parse_line("2024-05-01 10:00:01 TRACE auth: x").is_none());
        assert!(parse_line("2024-05-01 10:00:01 INFO auth no colon").is_none());
        assert!(parse_line("2024-05-01 10:00:01 INFO auth: ").is_none());
    }

    #[test]
    fn parse_line_is_zero_copy() {
        let text = String::from("2024-05-01 10:00:01 INFO auth: hello");
        let e = parse_line(&text).unwrap();
        let start = text.as_ptr() as usize;
        let end = start + text.len();
        for s in [e.level, e.module, e.message] {
            let p = s.as_ptr() as usize;
            assert!(p >= start && p < end, "복사가 일어났어요");
        }
    }

    #[test]
    fn parse_log_counts() {
        let (entries, skipped) = parse_log(SAMPLE);
        assert_eq!(entries.len(), 4);
        assert_eq!(skipped, 3); // 빈 줄은 세지 않아요
    }

    #[test]
    fn filter_and_display() {
        let (entries, _) = parse_log(SAMPLE);
        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].to_string(), "[ERROR] db: query timeout after 30s");
        assert!(filter_by_level(&entries, "DEBUG").is_empty());
    }

    #[test]
    fn longest_message_outlives_vec() {
        let text = String::from(SAMPLE);
        let longest = {
            let (entries, _) = parse_log(&text);
            longest_message(&entries) // entries 는 여기서 사라지지만 결과는 text 를 빌림
        };
        assert_eq!(longest, Some("query timeout after 30s"));
        assert_eq!(longest_message(&[]), None);
    }

    #[test]
    fn counts_are_sorted() {
        let (entries, _) = parse_log(SAMPLE);
        let counts: Vec<_> = count_by_module(&entries).into_iter().collect();
        assert_eq!(counts, vec![("api", 1), ("auth", 2), ("db", 1)]);
    }

    #[test]
    fn longer_picks_longer() {
        assert_eq!(longer("abc", "de"), "abc");
        assert_eq!(longer("a", "bcd"), "bcd");
        assert_eq!(longer("ab", "cd"), "ab");
    }
}
