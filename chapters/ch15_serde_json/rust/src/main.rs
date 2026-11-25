//! Ch15. Serde 와 JSON — LLM 대화 로그 변환기
//!
//! 🎯 목표
//!   - `#[derive(Serialize, Deserialize)]` 와 serde 속성(rename_all / tag / untagged / default / skip_serializing_if)을 익힙니다.
//!   - Claude Messages API 의 content block 모양(text / thinking / tool_use / tool_result)을 타입으로 모델링합니다.
//!   - 모르는 블록(미래에 추가될 타입)도 "원본 그대로" 보관했다가 되돌려 보내는 패턴을 익힙니다.
//!   - 타입을 쓰는 `serde_json::from_str` 와 느슨한 `serde_json::Value` 를 구분해서 씁니다.
//!
//! 📂 대응 파일: ../python/main.py,  ../cpp/main.cpp  (데이터: ../data/conversation.json)
//!
//! 🚶 진행 방법
//!   1. 타입 정의 위의 TODO(1)~(4) 에서 serde 속성을 "추가"합니다. (derive 는 이미 붙어 있어서 컴파일은 항상 됩니다)
//!   2. TODO(5)~(12) 의 `todo!()` 를 번호 순서대로 구현합니다.
//!   3. `cargo test -p ch15_serde_json` 으로 확인하세요. 막히면 정답: `--example solution`
//!
//! ✅ 완료 기준
//!   `cargo test -p ch15_serde_json` 이 모두 초록불이고,
//!   `../../scripts/compare.sh ch15` 가 Python / C++ / Rust 출력이 같다고 나와야 합니다.

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 안 쓴 코드에 대한 경고가 보입니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── 타입 정의 (Claude Messages API 의 모양) ─────────────────────

// ─────────────────────────────────────────────
// TODO(1): role 은 JSON 에서 소문자 문자열 "user" / "assistant" 입니다.
//   🐍 Python: role: str  ("user" / "assistant" 를 문자열로 비교)            (python/main.py: Message)
//   ⚙️ C++  : std::string role                                               (cpp/main.cpp: Message)
//   🦀 Rust : enum Role { User, Assistant } + 변환 규칙 속성
//   💡 힌트: 컨테이너 속성 `#[serde(rename_all = "lowercase")]` 를 enum 위에 붙입니다.
//   ❓ 왜?: 안 붙이면 JSON 에는 "User" 라고 대문자로 써야 합니다. 잘못된 값("robot")은 어떤 에러가 날까요?
// ─────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
}

// ─────────────────────────────────────────────
// TODO(2): `"type"` 필드 값으로 variant 를 고르는 "내부 태그(internally tagged)" enum 으로 만드세요.
//   🐍 Python: if raw["type"] == "text": ...                    (python/main.py: block_from_json)
//   ⚙️ C++  : if (kind == "text") ...                           (cpp/main.cpp: block_from_json)
//   🦀 Rust : `#[serde(tag = "type", rename_all = "snake_case")]`   -> ToolUse 가 "tool_use" 가 됩니다.
//   💡 힌트: ToolResult 의 is_error 는 없을 수 있고(Option), None 이면 JSON 에 쓰지 않아야 합니다.
//           `#[serde(default, skip_serializing_if = "Option::is_none")]` 를 그 필드에 붙이세요.
//   ❓ 왜?: 속성이 없으면 `{"Text": {"text": "..."}}` 처럼 "외부 태그" 모양이 되어 API 의 모양과 달라요.
//
//   🔬 실험 A: tag 속성을 잠깐 지우고 `cargo test` 의 에러 메시지를 읽어 보세요. 어떤 모양을 기대한다고 하나요?
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnownBlock {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
}

// ─────────────────────────────────────────────
// TODO(3): 모르는 블록을 "원본 그대로" 보관하는 래퍼 enum 으로 만드세요.
//   🐍 Python: 모르는 블록은 그냥 dict 로 둠                       (python/main.py: block_from_json 의 마지막 return raw)
//   ⚙️ C++  : std::variant 의 마지막 후보가 json                    (cpp/main.cpp: ContentBlock)
//   🦀 Rust : `#[serde(untagged)]` — 위에서부터 차례로 시도: Known 이 실패하면 Unknown(Value) 가 받습니다.
//   💡 힌트: MessageContent 도 `#[serde(untagged)]` 입니다 (문자열이거나 블록 배열).
//   ❓ 왜 `#[serde(other)]` 를 안 쓰나요?: other 는 "아무거나 받는" 단위 variant 라서 내용을 버립니다.
//      에이전트는 모르는 블록(예: redacted_thinking)도 다음 요청에 "그대로" 돌려보내야 해요.
//
//   🔬 실험 B: Known/Unknown 의 선언 순서를 바꾸면 어떻게 될까요? (Unknown 이 먼저면 모든 블록이 Unknown 이 됩니다!)
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentBlock {
    Known(KnownBlock),
    Unknown(Value),
}

/// `content` 는 문자열일 수도, 블록 배열일 수도 있습니다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    // 스키마는 자유 형식이므로 Value 로 둡니다.
    pub input_schema: Value,
}

// ─────────────────────────────────────────────
// TODO(4): 선택 필드 처리.
//   - system: 없을 수 있음(Option). 없으면 JSON 에도 쓰지 않음.
//   - stop_sequences: 입력에 없으면 빈 Vec, 비어 있으면 출력에서 생략.
//   🐍 Python: raw.get("system") / raw.get("stop_sequences", [])      (python/main.py: request_from_json)
//   ⚙️ C++  : raw.contains("system")                                  (cpp/main.cpp: request_from_json)
//   🦀 Rust : `#[serde(default, skip_serializing_if = "Option::is_none")]`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
//   ❓ 왜?: Option 필드는 default 없이도 누락을 허용하지만, Vec 은 default 가 없으면 "missing field" 에러가 납니다.
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub max_tokens: u32,
    pub system: Option<String>,
    pub tools: Vec<Tool>,
    pub messages: Vec<Message>,
    pub stop_sequences: Vec<String>,
}

// ─────────────────────────────────────────────
// TODO(10-a): 분석 결과 구조체. JSON 으로 나갈 때 messageCount 처럼 camelCase 여야 합니다.
//   💡 힌트: `#[serde(rename_all = "camelCase")]`
//   (함수 summarize 는 아래 TODO(10-b))
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub message_count: usize,
    pub tool_calls: usize,
    pub tool_errors: usize,
    pub tools_used: BTreeMap<String, usize>,
    pub unknown_blocks: Vec<String>,
}

// ── 파싱 / 분석 ────────────────────────────────────────────

// ─────────────────────────────────────────────
// TODO(5): JSON 문자열 -> 타입.
//   🐍 Python: request_from_json(json.loads(text))               (python/main.py: parse_request)
//   ⚙️ C++  : request_from_json(json::parse(text))                (cpp/main.cpp: parse_request)
//   🦀 Rust : serde_json::from_str(text)
//   💡 힌트: 반환 타입(Result<Request, _>)만 보고 from_str 이 어떤 타입으로 읽을지 추론합니다. parse_message 도 같은 모양!
//   ❓ 왜?: Python 은 dict 를 받고 직접 검사하지만, Rust 는 타입이 곧 검사기입니다. 모양이 안 맞으면 Err 가 나와요.
// ─────────────────────────────────────────────
pub fn parse_request(text: &str) -> Result<Request, serde_json::Error> {
    todo!("TODO(5): serde_json::from_str")
}

pub fn parse_message(text: &str) -> Result<Message, serde_json::Error> {
    todo!("TODO(5): serde_json::from_str")
}

// ─────────────────────────────────────────────
// TODO(6): 블록의 type 이름을 돌려주세요. 아는 블록은 "text"/"thinking"/"tool_use"/"tool_result",
//          모르는 블록은 원본 JSON 의 "type" 문자열 (없으면 "?").
//   🐍 Python: match block: case TextBlock(): ...                (python/main.py: block_kind)
//   ⚙️ C++  : std::visit + if constexpr                          (cpp/main.cpp: block_kind)
//   🦀 Rust : 중첩 패턴 `ContentBlock::Known(KnownBlock::Text { .. })` 과 `ContentBlock::Unknown(raw)`
//   💡 힌트: Value 에서 문자열 꺼내기: raw.get("type").and_then(Value::as_str)
// ─────────────────────────────────────────────
pub fn block_kind(block: &ContentBlock) -> String {
    todo!("TODO(6): match")
}

/// 문자열 content 는 text 블록 하나로 칩니다. (구현되어 있어요)
pub fn message_kinds(msg: &Message) -> Vec<String> {
    match &msg.content {
        MessageContent::Text(_) => vec!["text".to_string()],
        MessageContent::Blocks(blocks) => blocks.iter().map(block_kind).collect(),
    }
}

/// 문자열 content 에는 블록이 없으므로 빈 슬라이스를 돌려줍니다. (구현되어 있어요)
fn blocks_of(msg: &Message) -> &[ContentBlock] {
    match &msg.content {
        MessageContent::Text(_) => &[],
        MessageContent::Blocks(blocks) => blocks,
    }
}

/// 블록 종류별 개수 (구현되어 있어요. TODO(6) 이 맞아야 동작합니다)
pub fn count_blocks(req: &Request) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for kind in req.messages.iter().flat_map(message_kinds) {
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts
}

// ─────────────────────────────────────────────
// TODO(7): tool_use 블록에서 도구 이름별 호출 횟수를 세어 주세요.
//   🐍 Python: isinstance(b, ToolUseBlock)                         (python/main.py: tools_used)
//   ⚙️ C++  : std::get_if<ToolUseBlock>(&b)                        (cpp/main.cpp: tools_used)
//   🦀 Rust : if let ContentBlock::Known(KnownBlock::ToolUse { name, .. }) = block { ... }
//   💡 힌트: *counts.entry(name.clone()).or_insert(0) += 1;  그리고 blocks_of(msg) 를 활용하세요.
// ─────────────────────────────────────────────
pub fn tools_used(req: &Request) -> BTreeMap<String, usize> {
    todo!("TODO(7): 도구별 호출 수")
}

// ─────────────────────────────────────────────
// TODO(8): tool_use 와 tool_result 를 id 로 짝지어, 한 줄씩 `이름 입력JSON -> 결과` 형태로 돌려주세요.
//          - 입력 JSON 은 공백 없는 compact 형식:  calculator {"expression":"12 * 34"} -> 408
//          - is_error 가 true 이면 결과 앞에 "ERROR: ":   calculator {...} -> ERROR: error: division by zero
//          - 짝이 없는 결과는:  ? <tool_use_id> -> <content>
//   🐍 Python: calls[b.id] = b / calls.get(b.tool_use_id)         (python/main.py: tool_call_report)
//   ⚙️ C++  : std::map<std::string, const ToolUseBlock*>           (cpp/main.cpp: tool_call_report)
//   🦀 Rust : BTreeMap<&str, (&str, &Value)> — 값을 복사하지 않고 "빌려서" 담아 둡니다.
//   💡 힌트: serde_json::Value 는 Display 를 구현합니다: format!("{input}") 이 곧 compact JSON 입니다.
//   ❓ 왜?: tool_use 는 assistant 메시지에, tool_result 는 다음 user 메시지에 있어서 "id 로 연결"해야 합니다 (ch17 에이전트 루프의 핵심!).
// ─────────────────────────────────────────────
pub fn tool_call_report(req: &Request) -> Vec<String> {
    todo!("TODO(8): id 로 짝짓기")
}

// ─────────────────────────────────────────────
// TODO(9): 새 요청 만들기.
//   - model 은 그대로, max_tokens = 256, system = Some("Answer in one short sentence."), tools 는 그대로 복제
//   - messages 는 원본의 "처음 3개"를 복제하고, 마지막 메시지(블록 배열)에 text 블록
//     "Please summarize the results above in one sentence." 를 덧붙입니다.
//   - stop_sequences 는 비워 둡니다.
//   🐍 Python: req.messages[:3] + list(...)                        (python/main.py: build_followup)
//   ⚙️ C++  : out.messages.assign(begin, begin + 3)                 (cpp/main.cpp: build_followup)
//   🦀 Rust : req.messages.iter().take(3).cloned().collect(), messages.last_mut()
//   💡 힌트: `if let Some(Message { content: MessageContent::Blocks(blocks), .. }) = messages.last_mut()`
//   ❓ 왜?: 원본을 건드리지 않으려면 clone 이 필요합니다. 어디서 복사가 일어나는지 눈여겨보세요.
// ─────────────────────────────────────────────
pub fn build_followup(req: &Request) -> Request {
    todo!("TODO(9): 새 요청 만들기")
}

// ─────────────────────────────────────────────
// TODO(10-b): Summary 를 계산하세요.
//   message_count = 메시지 수, tool_calls = 도구 호출 총합, tool_errors = is_error == Some(true) 인 tool_result 수,
//   tools_used = tools_used(req), unknown_blocks = 모르는 블록의 type 이름 (정렬 + 중복 제거)
//   💡 힌트: matches!(b, ContentBlock::Known(KnownBlock::ToolResult { is_error: Some(true), .. })),
//           Vec::sort() + Vec::dedup()
// ─────────────────────────────────────────────
pub fn summarize(req: &Request) -> Summary {
    todo!("TODO(10-b): 요약")
}

// ─────────────────────────────────────────────
// TODO(11): 메시지 JSON 하나를 파싱해서 "ok" / "syntax" / "data" 로 분류하세요.
//   🐍 Python: json.JSONDecodeError 는 syntax, 그 밖의 ValueError 는 data        (python/main.py: classify_error)
//   ⚙️ C++  : json::parse_error vs DataError                                      (cpp/main.cpp: classify_error)
//   🦀 Rust : err.classify() -> serde_json::error::Category::{Io, Syntax, Data, Eof}
//   💡 힌트: Eof(JSON 이 중간에 끊김)도 "syntax" 로 취급합니다.
//   ❓ 왜?: Syntax = JSON 자체가 깨짐, Data = JSON 은 맞지만 우리 타입과 모양이 다름. 에러 보고서에서 구분하면 디버깅이 쉬워요.
// ─────────────────────────────────────────────
pub fn classify_error(text: &str) -> &'static str {
    todo!("TODO(11): Category 로 분류")
}

// ─────────────────────────────────────────────
// TODO(12): 타입으로 읽었다가 다시 JSON 으로 써도 원본과 똑같은지(손실 없는지) 검사하세요.
//   🦀 Rust : 원본은 `serde_json::from_str::<Value>`, 되돌린 것은 `serde_json::to_value(&req)`. Value 끼리 == 로 비교.
//   💡 힌트: 읽기/쓰기 중 하나라도 실패하면 false.
//   ❓ 왜?: Unknown 블록(redacted_thinking)을 Value 로 보관하는 이유가 바로 이것입니다. TODO(3) 을 틀리면 여기서 no 가 나옵니다.
//
//   🔬 실험 C: 데이터 파일에 `"extra_field": 1` 을 추가하고 다시 돌려 보세요. 최상위 Request 는 모르는 필드를 어떻게 처리하나요?
//      (기본은 "무시" 입니다. 엄격하게 하려면 `#[serde(deny_unknown_fields)]`. 단, 그러면 API 에 새 필드가 생길 때 깨집니다.)
//   🔬 실험 D: `serde_json::Value` 의 객체는 기본적으로 키를 알파벳 순으로 저장합니다(BTreeMap).
//      tool_use 의 input 에 {"b":1,"a":2} 를 넣고 to_string 해 보세요. (README 의 "키 순서" 참고)
// ─────────────────────────────────────────────
pub fn round_trip_lossless(text: &str) -> bool {
    todo!("TODO(12): 왕복 비교")
}

/// 2칸 들여쓰기 pretty JSON (Python: indent=2, ensure_ascii=False 와 같은 결과)
pub fn to_pretty<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("serialize")
}

fn main() {
    let text = std::fs::read_to_string("data/conversation.json").expect("data/conversation.json");
    let req = parse_request(&text).expect("valid conversation");

    println!("== request ==");
    println!("model: {}", req.model);
    println!("max_tokens: {}", req.max_tokens);
    let names: Vec<&str> = req.tools.iter().map(|t| t.name.as_str()).collect();
    println!("tools: {}", names.join(", "));
    println!("messages: {}", req.messages.len());

    println!("== messages ==");
    for (i, msg) in req.messages.iter().enumerate() {
        let role = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        println!("[{i}] {role}: {}", message_kinds(msg).join(", "));
    }

    println!("== block counts ==");
    for (kind, n) in count_blocks(&req) {
        println!("{kind}: {n}");
    }

    println!("== tools used ==");
    for (name, n) in tools_used(&req) {
        println!("{name}: {n}");
    }

    println!("== tool calls ==");
    for line in tool_call_report(&req) {
        println!("{line}");
    }

    println!("== bad input ==");
    let bad = [
        r#"{"role": "user", "content": "#,
        r#"{"role": "robot", "content": "hi"}"#,
        r#"{"role": "user"}"#,
        r#"{"role": "user", "content": [{"type": "text"}]}"#,
        r#"{"role": "user", "content": 42}"#,
    ];
    for (i, b) in bad.iter().enumerate() {
        println!("{}: {}", i + 1, classify_error(b));
    }

    println!("== round trip ==");
    println!("lossless: {}", if round_trip_lossless(&text) { "yes" } else { "no" });

    println!("== follow-up request ==");
    println!("{}", to_pretty(&build_followup(&req)));

    println!("== summary (original) ==");
    println!("{}", to_pretty(&summarize(&req)));

    println!("== summary (follow-up) ==");
    println!("{}", to_pretty(&summarize(&build_followup(&req))));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // 테스트는 cwd 에 의존하지 않도록 인라인 JSON 을 씁니다.
    const SAMPLE: &str = r#"{
      "model": "m",
      "max_tokens": 10,
      "system": "sys",
      "tools": [{"name": "calc", "description": "d", "input_schema": {"type": "object"}}],
      "messages": [
        {"role": "user", "content": "hi"},
        {"role": "assistant", "content": [
          {"type": "text", "text": "ok"},
          {"type": "tool_use", "id": "t1", "name": "calc", "input": {"x": 1}},
          {"type": "future_block", "payload": [1, 2, 3]}
        ]},
        {"role": "user", "content": [
          {"type": "tool_result", "tool_use_id": "t1", "content": "2", "is_error": true}
        ]},
        {"role": "user", "content": [
          {"type": "tool_result", "tool_use_id": "t9", "content": "x"}
        ]}
      ]
    }"#;

    #[test]
    fn parses_string_and_block_content() {
        let req = parse_request(SAMPLE).unwrap();
        assert_eq!(req.messages.len(), 4);
        assert_eq!(req.messages[0].content, MessageContent::Text("hi".into()));
        assert!(matches!(req.messages[1].content, MessageContent::Blocks(ref b) if b.len() == 3));
        assert_eq!(req.messages[1].role, Role::Assistant);
    }

    #[test]
    fn known_blocks_are_typed_and_unknown_is_raw() {
        let req = parse_request(SAMPLE).unwrap();
        let MessageContent::Blocks(blocks) = &req.messages[1].content else { panic!("blocks") };
        assert!(matches!(&blocks[0], ContentBlock::Known(KnownBlock::Text { text }) if text == "ok"));
        assert!(matches!(&blocks[1], ContentBlock::Known(KnownBlock::ToolUse { name, .. }) if name == "calc"));
        assert_eq!(blocks[2], ContentBlock::Unknown(json!({"type": "future_block", "payload": [1, 2, 3]})));
        assert_eq!(block_kind(&blocks[2]), "future_block");
    }

    #[test]
    fn serializes_tag_first_and_skips_none() {
        let block = ContentBlock::Known(KnownBlock::ToolResult {
            tool_use_id: "t1".into(),
            content: "42".into(),
            is_error: None,
        });
        assert_eq!(
            serde_json::to_string(&block).unwrap(),
            r#"{"type":"tool_result","tool_use_id":"t1","content":"42"}"#
        );
        let err = ContentBlock::Known(KnownBlock::ToolResult {
            tool_use_id: "t1".into(),
            content: "boom".into(),
            is_error: Some(true),
        });
        assert!(serde_json::to_string(&err).unwrap().ends_with(r#""is_error":true}"#));
    }

    #[test]
    fn request_optional_fields() {
        let req = parse_request(SAMPLE).unwrap();
        assert!(req.stop_sequences.is_empty()); // 없으면 default
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("stop_sequences").is_none()); // 비었으면 생략
        let mut no_sys = req.clone();
        no_sys.system = None;
        assert!(serde_json::to_value(&no_sys).unwrap().get("system").is_none());
    }

    #[test]
    fn counts_and_tool_usage() {
        let req = parse_request(SAMPLE).unwrap();
        let counts = count_blocks(&req);
        assert_eq!(counts["text"], 2); // 문자열 content + text 블록
        assert_eq!(counts["future_block"], 1);
        assert_eq!(counts["tool_result"], 2);
        assert_eq!(tools_used(&req)["calc"], 1);
    }

    #[test]
    fn report_pairs_calls_and_results() {
        let req = parse_request(SAMPLE).unwrap();
        assert_eq!(
            tool_call_report(&req),
            vec![r#"calc {"x":1} -> ERROR: 2"#.to_string(), "? t9 -> x".to_string()]
        );
    }

    #[test]
    fn followup_and_summary() {
        let req = parse_request(SAMPLE).unwrap();
        let follow = build_followup(&req);
        assert_eq!(follow.messages.len(), 3);
        assert_eq!(follow.max_tokens, 256);
        let MessageContent::Blocks(last) = &follow.messages[2].content else { panic!("blocks") };
        assert!(matches!(last.last(), Some(ContentBlock::Known(KnownBlock::Text { .. }))));
        let s = summarize(&req);
        assert_eq!((s.message_count, s.tool_calls, s.tool_errors), (4, 1, 1));
        assert_eq!(s.unknown_blocks, vec!["future_block".to_string()]);
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["messageCount"], 4); // camelCase
        assert_eq!(v["toolsUsed"]["calc"], 1);
    }

    #[test]
    fn error_classification() {
        assert_eq!(classify_error(r#"{"role":"user","content":"x"}"#), "ok");
        assert_eq!(classify_error(r#"{"role":"user","content":"#), "syntax");
        assert_eq!(classify_error(r#"{"role":"robot","content":"x"}"#), "data");
        assert_eq!(classify_error(r#"{"role":"user","content":42}"#), "data");
    }

    #[test]
    fn round_trip_keeps_unknown_blocks() {
        assert!(round_trip_lossless(SAMPLE));
        assert!(!round_trip_lossless("not json"));
    }

    #[test]
    fn pretty_uses_two_spaces() {
        let out = to_pretty(&json!({"a": [1], "b": {}}));
        assert_eq!(out, "{\n  \"a\": [\n    1\n  ],\n  \"b\": {}\n}");
    }
}
