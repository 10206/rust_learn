//! # Ch09. 라이프타임 — 제로카피 로그 파서
//!
//! ## 목표
//! - `LogEntry<'a>` 처럼 **참조를 들고 있는 구조체**를 만들고 `'a` 의 의미를 이해합니다.
//! - 라이프타임 생략 규칙(elision)과 `fn longer<'a>(..) -> &'a str` 같은 명시 표기를 구분합니다.
//! - "라이프타임은 **설명**이지 **제어**가 아니다" — 컴파일러에게 관계를 알려 줄 뿐 수명을 늘려 주지 않음을 체감합니다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`   (복사하는 str 슬라이스)
//! - C++   : `../cpp/main.cpp`     (`std::string_view`, 댕글링 위험은 프로그래머 책임)
//! - 데이터: `../data/app.log`
//!
//! ## 진행 방법
//! 1. `TODO(1)` 부터 번호 순서대로 `todo!()` 를 지우고 구현합니다. (`cargo test -p ch09_lifetimes`)
//! 2. 🔬 실험 블록(파일 중간)의 주석을 풀어 **진짜 컴파일러 에러**를 보고, 다시 주석 처리하세요.
//!
//! ## 완료 기준
//! `cargo test -p ch09_lifetimes` 모두 통과 AND `../../scripts/compare.sh ch09` 출력이 세 언어 모두 동일.

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 컴파일러가 안 쓴 코드를 알려 줍니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::collections::BTreeMap;
use std::fmt;
use std::fs;

// 문자열을 복사하지 않고 원본 텍스트의 "일부"를 가리키는 구조체.
// 'a = "이 구조체는 원본 텍스트보다 오래 살 수 없다"는 관계를 컴파일러에게 알려 주는 이름표입니다.
//   ⚙️ C++ : struct { std::string_view level, module, message; }  (관계 표시 없음 -> 댕글링 위험)
//   🐍 Python: dataclass(str, str, str)                             (슬라이스마다 복사됨, 안전하지만 느림)
#[derive(Debug, Clone, Copy, PartialEq)]
struct LogEntry<'a> {
    level: &'a str,
    module: &'a str,
    message: &'a str,
}

// ─────────────────────────────────────────────
// TODO(1): `impl fmt::Display for LogEntry<'_>` — 출력 형태: `[ERROR] db: query timeout`
//   🐍 Python: f"[{self.level}] {self.module}: {self.message}"        (python/main.py: LogEntry.__str__)
//   ⚙️ C++  : "[" + level + "] " + module + ": " + message            (cpp/main.cpp: to_string)
//   🦀 Rust : write!(f, "...", ...)
//   💡 힌트: impl 줄의 `'_` 는 "이름 붙일 필요 없는 라이프타임" 이라는 뜻이에요. 본문에서는 아무 라이프타임도 안 쓰입니다.
//   ❓ 왜?: 구조체에 라이프타임 매개변수가 있으면 impl 에서도 적어야 합니다 (`impl<'a> ... for LogEntry<'a>` 와 같은 뜻).
// ─────────────────────────────────────────────
impl fmt::Display for LogEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("TODO(1): [LEVEL] module: message 형태로 write!")
    }
}

const LEVELS: [&str; 4] = ["DEBUG", "INFO", "WARN", "ERROR"];

// ─────────────────────────────────────────────
// TODO(2): 한 줄을 파싱해 `LogEntry` 로 돌려주세요. 형식이 틀리면 None.
//   입력 예: "2024-05-01 10:00:01 INFO auth: user alice logged in"
//   규칙:  날짜, 시간, 레벨, 나머지 → 4조각(앞 3개는 공백 하나로 구분, 나머지는 공백 포함 그대로)
//          레벨은 LEVELS 안에 있어야 함 / 나머지는 ": " 로 module 과 message 로 나눔
//          module 은 비어 있거나 공백을 포함하면 안 되고, message 는 비어 있으면 안 됨
//   🐍 Python: line.split(" ", 3), rest.partition(": ")            (python/main.py: parse_line)
//   ⚙️ C++  : string_view::find / substr / remove_prefix            (cpp/main.cpp: parse_line)
//   🦀 Rust : `splitn(4, ' ')`, `split_once(": ")`, 그리고 `?` 로 None 전파.
//   💡 힌트: 결과의 &str 들은 전부 `line` 의 일부예요. 새 String 을 만들지 마세요!
//           시그니처 `&str -> Option<LogEntry<'_>>` 는 생략 규칙으로 "출력 라이프타임 = 입력 라이프타임"이 됩니다.
//   ❓ 왜?: 복사 없이 원본을 가리키므로 빠릅니다. 대신 `line` 이 먼저 사라지면 결과가 못 쓰이도록 컴파일러가 막아요.
// ─────────────────────────────────────────────
fn parse_line(line: &str) -> Option<LogEntry<'_>> {
    todo!("TODO(2): 한 줄 파싱")
}

// ─────────────────────────────────────────────
// TODO(3): 여러 줄 텍스트를 파싱해 (엔트리 목록, 건너뛴 줄 수) 를 돌려주세요.
//   - 공백뿐인 빈 줄은 건너뛰되 "건너뛴 줄 수" 에는 세지 않습니다.
//   - 파싱에 실패한 줄(None)만 skipped 로 셉니다.
//   🐍 Python: for line in text.splitlines()                      (python/main.py: parse_log)
//   ⚙️ C++  : '\n' 로 잘라 가며 string_view 를 줄여 나감              (cpp/main.cpp: parse_log)
//   🦀 Rust : text.lines() 가 돌려주는 &str 도 text 의 일부입니다.
//   💡 힌트: 여기서는 라이프타임을 직접 적었어요 — `<'a>` 가 `text` 와 반환 Vec 안의 &str 을 이어 줍니다.
//           `line.trim().is_empty()` 로 빈 줄 판별.
//   ❓ 왜?: 반환 타입 안의 'a 덕분에 호출자가 `text` 를 일찍 버리면 컴파일 에러가 납니다 (🔬 실험 2).
// ─────────────────────────────────────────────
fn parse_log<'a>(text: &'a str) -> (Vec<LogEntry<'a>>, usize) {
    todo!("TODO(3): 줄 단위 파싱")
}

// ─────────────────────────────────────────────
// TODO(4): level 이 같은 엔트리만 모아 새 Vec 으로 돌려주세요. (LogEntry 는 Copy 라서 복사해도 문자열은 복사되지 않아요)
//   🐍 Python: [e for e in entries if e.level == level]           (python/main.py: filter_by_level)
//   ⚙️ C++  : for 루프 + push_back                                 (cpp/main.cpp: filter_by_level)
//   🦀 Rust : iter().filter(..).copied().collect()
//   💡 힌트: 매개변수 `level: &str` 은 이 함수 안에서만 쓰고 결과에는 안 들어가므로 라이프타임이 달라도 됩니다.
//           결과 안의 &str 은 원본 텍스트('a)를 가리키므로 반환 타입이 `Vec<LogEntry<'a>>` 예요.
//   ❓ 왜?: 입력이 `&[LogEntry<'a>]`, `&str` 두 개라서 생략 규칙만으로는 어느 쪽인지 알 수 없어 'a 를 직접 적습니다.
// ─────────────────────────────────────────────
fn filter_by_level<'a>(entries: &[LogEntry<'a>], level: &str) -> Vec<LogEntry<'a>> {
    todo!("TODO(4): 레벨로 거르기")
}

// ─────────────────────────────────────────────
// TODO(5): 메시지가 가장 긴 엔트리의 message 를 `Option<&'a str>` 로 돌려주세요 (빈 목록이면 None).
//   🐍 Python: max((e.message for e in entries), key=len)          (python/main.py: longest_message)
//   ⚙️ C++  : std::max_element(...) -> string_view                  (cpp/main.cpp: longest_message)
//   🦀 Rust : 새 String 을 만들지 말고 원본 text 의 일부를 그대로 돌려줍니다.
//   💡 힌트: iter().map(|e| e.message).max_by_key(|m| m.len())
//   ❓ 왜?: 반환 타입이 `&'a str` 이라서 `entries` Vec 이 사라져도 결과는 계속 쓸 수 있어요 (원본 text 가 살아 있는 한).
//          만약 `'a` 대신 슬라이스의 라이프타임에 묶었다면 불필요하게 짧아졌을 거예요. 테스트 `longest_message_outlives_vec` 참고.
// ─────────────────────────────────────────────
fn longest_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str> {
    todo!("TODO(5): 가장 긴 메시지")
}

// ─────────────────────────────────────────────
// TODO(6): module 별 엔트리 개수를 `BTreeMap<&'a str, usize>` 로 세어 돌려주세요. (키 순서대로 정렬됨)
//   🐍 Python: counts.get(k, 0) + 1 → sorted                       (python/main.py: count_by_module)
//   ⚙️ C++  : std::map<string_view,int> ++counts[e.module]          (cpp/main.cpp: count_by_module)
//   🦀 Rust : `*counts.entry(key).or_insert(0) += 1;`  (바로 아래 count_by_level 을 참고하세요)
//   ❓ 왜?: 키도 원본 텍스트를 가리키는 &'a str 이라 문자열 복사가 한 번도 없습니다.
// ─────────────────────────────────────────────
fn count_by_module<'a>(entries: &[LogEntry<'a>]) -> BTreeMap<&'a str, usize> {
    todo!("TODO(6): module 별 개수")
}

// 이미 완성된 예시: 같은 모양을 level 에 적용했어요. TODO(6) 의 힌트로 쓰세요.
fn count_by_level<'a>(entries: &[LogEntry<'a>]) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for e in entries {
        *counts.entry(e.level).or_insert(0) += 1;
    }
    counts
}

// ─────────────────────────────────────────────
// TODO(7): 두 문자열 중 더 긴 쪽을 돌려주세요 (길이가 같으면 a).
//   🐍 Python: b if len(b) > len(a) else a                          (python/main.py: longer)
//   ⚙️ C++  : b.size() > a.size() ? b : a                           (cpp/main.cpp: longer)
//   🦀 Rust : 시그니처가 이 TODO 의 핵심! `<'a>` 를 직접 적어야 컴파일됩니다 — 🔬 실험 1 에서 안 적으면 어떻게 되는지 보세요.
//   💡 힌트: 이미 `<'a>` 가 적혀 있어요. 본문은 if/else 한 줄입니다.
//   ❓ 왜?: 반환값이 a 인지 b 인지는 실행해 봐야 알아서, 컴파일러는 "둘 중 짧게 사는 쪽만큼만 유효"하다고 가정합니다.
// ─────────────────────────────────────────────
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!("TODO(7): 더 긴 문자열")
}

// ─────────────────────────────────────────────
// 🔬 실험 1: 라이프타임을 안 적으면?  (E0106)
//   아래 함수의 주석을 풀고 `cargo build -p ch09_lifetimes` 를 해 보세요.
//   ❓ 컴파일러가 "a 에서 빌린 건지 b 에서 빌린 건지 모르겠다"고 하는 이유는? (⚙️ C++ string_view 는 이런 질문 자체가 없어요)
//   ❓ help 가 제안하는 수정은 무엇인가요? 그대로 적용하면 TODO(7) 의 시그니처와 같아집니다.
//
// fn longer_no_lifetime(a: &str, b: &str) -> &str {
//     if b.len() > a.len() { b } else { a }
// }
//
// 🔬 실험 1-b: 구조체 필드에 참조를 쓰면서 'a 를 안 적으면?  (E0106)
//
// struct BadEntry {
//     level: &str,
//     message: &str,
// }
//
// 🔬 실험 2: 원본(text)이 먼저 죽으면?  (E0597)  ← C++ string_view 댕글링 버그를 Rust 가 잡는 장면
//   아래 `main` 안의 EXPERIMENT 주석을 푸세요 (또는 별도 함수로 옮겨서 해 보세요).
//   ❓ "borrowed value does not live long enough" 에서 어느 줄이 빌렸고, 어느 줄에서 아직 쓰는지 에러 화살표로 찾아보세요.
//
// fn experiment_dangling() {
//     let entries;
//     {
//         let text = String::from("2024-05-01 10:00:01 INFO auth: hello");
//         entries = parse_log(&text).0;   // entries 는 text 를 빌림
//     }                                   // text 는 여기서 drop!
//     println!("{}", entries.len());      // 그런데 아직 쓴다 → 컴파일 에러
// }
//
// 🔬 실험 3: 지역 변수의 참조를 반환하면?  (E0515)
//   ❓ 같은 코드를 C++ 로 쓰면 컴파일·실행이 되고 가끔 이상한 값이 나옵니다. 왜 Rust 는 아예 막을까요?
//
// fn first_word_upper(text: &str) -> &str {
//     let word = text.split_whitespace().next().unwrap_or("");
//     let upper = word.to_uppercase();   // 새 String (이 함수가 소유)
//     &upper                              // 함수가 끝나면 upper 는 사라지는데 참조를 돌려줌
// }
//   💡 고치는 법: 반환 타입을 `String` 으로 바꾸고 `upper` 를 그대로 돌려주세요 (소유권 이동).
//
// 🔬 실험 4: `'static` 로 우겨 보기 — 라이프타임은 "설명"이지 "제어"가 아니에요.
//   `fn longer<'a>(..)` 를 `fn longer(a: &str, b: &str) -> &'static str` 로 바꾸고 `a` 를 반환해 보세요.
//   ❓ 어떤 에러(lifetime may not live long enough)가 나나요? 'static 이라고 쓴다고 값이 영원히 살아 주지는 않습니다.
// ─────────────────────────────────────────────

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
