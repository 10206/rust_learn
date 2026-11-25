//! Ch15. Serde 와 JSON — 정답 (solution)
//!
//! 대응 파일: ../python/main.py, ../cpp/main.cpp
//! 실행: cargo run -q -p ch15_serde_json --example solution
//! 테스트: cargo test -p ch15_serde_json --example solution

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── 타입 정의 (Claude Messages API 의 모양) ─────────────────────

/// TODO(1): role 은 JSON 에서 소문자 문자열 "user" / "assistant" 입니다.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// TODO(2): `"type"` 필드 값으로 variant 를 고르는 "내부 태그(internally tagged)" enum.
/// 필드 선언 순서가 곧 직렬화할 때의 키 순서입니다 ("type" 이 항상 맨 앞).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
        // 없으면 None, None 이면 JSON 에 아예 쓰지 않습니다.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// TODO(3): 모르는 블록은 `Unknown(Value)` 로 "원본 그대로" 보관합니다.
/// `#[serde(untagged)]` 는 위에서부터 차례로 시도합니다: Known 이 실패하면 Unknown 이 받습니다.
/// (`#[serde(other)]` 는 데이터를 버리는 단위 variant 만 가능해서, 되돌려 보내야 하는 에이전트에는 부적합합니다.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentBlock {
    Known(KnownBlock),
    Unknown(Value),
}

/// `content` 는 문자열일 수도, 블록 배열일 수도 있습니다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
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

/// TODO(4): 선택 필드 처리. system 은 없을 수 있고, stop_sequences 는 없으면 빈 Vec, 비어 있으면 출력 생략.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub max_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub tools: Vec<Tool>,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
}

/// TODO(10): 분석 결과. JSON 에서는 camelCase 로 나갑니다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub message_count: usize,
    pub tool_calls: usize,
    pub tool_errors: usize,
    pub tools_used: BTreeMap<String, usize>,
    pub unknown_blocks: Vec<String>,
}

// ── 파싱 / 분석 ────────────────────────────────────────────

/// TODO(5): JSON 문자열 -> 타입
pub fn parse_request(text: &str) -> Result<Request, serde_json::Error> {
    serde_json::from_str(text)
}

pub fn parse_message(text: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(text)
}

/// TODO(6): 블록의 type 이름. 모르는 블록은 원본 JSON 의 "type" 값.
pub fn block_kind(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Known(KnownBlock::Text { .. }) => "text".to_string(),
        ContentBlock::Known(KnownBlock::Thinking { .. }) => "thinking".to_string(),
        ContentBlock::Known(KnownBlock::ToolUse { .. }) => "tool_use".to_string(),
        ContentBlock::Known(KnownBlock::ToolResult { .. }) => "tool_result".to_string(),
        ContentBlock::Unknown(raw) => raw
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string(),
    }
}

/// 문자열 content 는 text 블록 하나로 칩니다.
pub fn message_kinds(msg: &Message) -> Vec<String> {
    match &msg.content {
        MessageContent::Text(_) => vec!["text".to_string()],
        MessageContent::Blocks(blocks) => blocks.iter().map(block_kind).collect(),
    }
}

/// 문자열 content 에는 블록이 없으므로 빈 슬라이스를 돌려줍니다.
fn blocks_of(msg: &Message) -> &[ContentBlock] {
    match &msg.content {
        MessageContent::Text(_) => &[],
        MessageContent::Blocks(blocks) => blocks,
    }
}

pub fn count_blocks(req: &Request) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for kind in req.messages.iter().flat_map(message_kinds) {
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts
}

/// TODO(7): tool_use 블록에서 도구 이름별 호출 횟수
pub fn tools_used(req: &Request) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for msg in &req.messages {
        for block in blocks_of(msg) {
            if let ContentBlock::Known(KnownBlock::ToolUse { name, .. }) = block {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// TODO(8): tool_use 와 tool_result 를 id 로 짝지어 `이름 입력 -> 결과` 한 줄씩.
pub fn tool_call_report(req: &Request) -> Vec<String> {
    // 호출 id -> (이름, 입력). 값은 빌려 오므로 복사가 없습니다.
    let mut calls: BTreeMap<&str, (&str, &Value)> = BTreeMap::new();
    let mut lines = Vec::new();
    for block in req.messages.iter().flat_map(blocks_of) {
        match block {
            ContentBlock::Known(KnownBlock::ToolUse { id, name, input }) => {
                calls.insert(id, (name, input));
            }
            ContentBlock::Known(KnownBlock::ToolResult { tool_use_id, content, is_error }) => {
                let head = match calls.get(tool_use_id.as_str()) {
                    Some((name, input)) => format!("{name} {input}"), // Value 의 Display 는 compact JSON
                    None => format!("? {tool_use_id}"),
                };
                let tail = if *is_error == Some(true) {
                    format!("ERROR: {content}")
                } else {
                    content.clone()
                };
                lines.push(format!("{head} -> {tail}"));
            }
            _ => {}
        }
    }
    lines
}

/// TODO(9): 처음 3개 메시지 + 요약 요청 text 블록을 담은 "새 요청" 만들기.
pub fn build_followup(req: &Request) -> Request {
    let mut messages: Vec<Message> = req.messages.iter().take(3).cloned().collect();
    if let Some(Message { content: MessageContent::Blocks(blocks), .. }) = messages.last_mut() {
        blocks.push(ContentBlock::Known(KnownBlock::Text {
            text: "Please summarize the results above in one sentence.".to_string(),
        }));
    }
    Request {
        model: req.model.clone(),
        max_tokens: 256,
        system: Some("Answer in one short sentence.".to_string()),
        tools: req.tools.clone(),
        messages,
        stop_sequences: Vec::new(),
    }
}

/// TODO(10)
pub fn summarize(req: &Request) -> Summary {
    let used = tools_used(req);
    let tool_errors = req
        .messages
        .iter()
        .flat_map(blocks_of)
        .filter(|b| matches!(b, ContentBlock::Known(KnownBlock::ToolResult { is_error: Some(true), .. })))
        .count();
    let mut unknown_blocks: Vec<String> = req
        .messages
        .iter()
        .flat_map(blocks_of)
        .filter(|b| matches!(b, ContentBlock::Unknown(_)))
        .map(block_kind)
        .collect();
    unknown_blocks.sort();
    unknown_blocks.dedup();
    Summary {
        message_count: req.messages.len(),
        tool_calls: used.values().sum(),
        tool_errors,
        tools_used: used,
        unknown_blocks,
    }
}

/// TODO(11): 메시지 JSON 하나를 파싱해서 "ok" / "syntax" / "data" 로 분류.
pub fn classify_error(text: &str) -> &'static str {
    use serde_json::error::Category;
    match parse_message(text) {
        Ok(_) => "ok",
        // Eof(중간에 끊김)도 문법 오류로 취급합니다.
        Err(e) => match e.classify() {
            Category::Syntax | Category::Eof => "syntax",
            Category::Data | Category::Io => "data",
        },
    }
}

/// TODO(12): 타입으로 읽었다가 다시 JSON 으로 써도 원본과 똑같은가? (Value 비교는 키 순서와 무관)
pub fn round_trip_lossless(text: &str) -> bool {
    let original: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    match parse_request(text).and_then(|req| serde_json::to_value(&req)) {
        Ok(again) => again == original,
        Err(_) => false,
    }
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
