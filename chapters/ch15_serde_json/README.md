# Ch15. Serde 와 JSON

> JSON 을 "dict 로 읽고 키를 일일이 검사"하는 대신, **타입을 선언하면 읽기/쓰기/검증이 자동**으로 생기는 Rust 의 방식. Claude Messages API 의 content block 이 그대로 연습 문제입니다.  ·  예상 소요: 이론 50분 / 실습 70분  ·  선행: Ch05 (enum), Ch06 (에러 처리), Ch08 (trait)

## 🎯 학습 목표

- `#[derive(Serialize, Deserialize)]` 한 줄이 무엇을 만들어 주는지 설명할 수 있습니다.
- 자주 쓰는 serde 속성(`rename_all`, `default`, `skip_serializing_if`, `tag`, `untagged`)을 읽고 쓸 수 있습니다.
- Claude Messages API 의 `content` (문자열 **또는** 블록 배열)와 content block(`text` / `thinking` / `tool_use` / `tool_result`)을 타입으로 모델링합니다.
- **모르는 블록을 원본 그대로 보관**했다가 되돌려 보내는 패턴(`Unknown(serde_json::Value)`)이 왜 에이전트에 필수인지 이해합니다.
- 타입 있는 파싱(`from_str::<T>`)과 느슨한 `serde_json::Value` 를 상황에 맞게 고르고, JSON 에러를 Syntax / Data 로 구분합니다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 (nlohmann::json) | Python (`json`) | Rust (serde + serde_json) |
|---|---|---|---|
| 문자열 → 값 | `json::parse(s)` | `json.loads(s)` | `serde_json::from_str::<T>(s)` |
| 값 → 문자열 | `j.dump()` / `j.dump(2)` | `json.dumps(o)` / `indent=2` | `to_string(&v)` / `to_string_pretty(&v)` |
| 타입 있는 변환 | `to_json` / `from_json` 을 **직접 작성** (또는 매크로 `NLOHMANN_DEFINE_TYPE_INTRUSIVE`) | `dataclass` + 직접 변환 (또는 pydantic) | `#[derive(Serialize, Deserialize)]` — **컴파일러가 생성** |
| 타입 없는 JSON | `json` (기본 타입이 곧 variant) | `dict` / `list` | `serde_json::Value` |
| 필드 이름 바꾸기 | 변환 함수 안에서 직접 | dict 키를 직접 | `#[serde(rename = "..")]`, `rename_all` |
| 없을 수 있는 필드 | `contains()` 검사 | `d.get("k")` | `Option<T>` / `#[serde(default)]` |
| 값이 비었으면 출력 생략 | `if (!v.empty())` | `if v:` | `#[serde(skip_serializing_if = "..")]` |
| 태그 있는 합집합 | `std::variant` + 직접 분기 | `match d["type"]` | `#[serde(tag = "type")] enum` |
| 키 순서 | 기본은 **정렬**, `ordered_json` 은 삽입 순서 | 삽입 순서 | 구조체는 **필드 선언 순서**, `Value` 는 기본 **정렬** |
| 모양이 틀리면 | 접근 시 예외 (`type_error`) | `KeyError`/`TypeError` (쓰는 시점) | `Err(serde_json::Error)` (**읽는 시점에 한 번에**) |

> 💡 **쉬운 설명 — "파싱(parse)" 과 "검증(validate)" 의 차이**
> Python 의 `json.loads` 는 글자 모양이 JSON 문법에 맞는지만 보고 `dict` 로 만들어 줘요. 그 뒤 `d["role"]` 이 있는지, 문자열인지는 **쓰다가** 에러가 납니다.
> Serde 는 "읽는 순간" 우리가 선언한 타입에 딱 맞는지 모두 확인합니다. 그래서 읽기에 성공한 값은 이후에 `KeyError` 같은 걱정 없이 쓸 수 있어요. ("Parse, don't validate")

## 📘 이론

> 이 장의 코드 조각은 <https://play.rust-lang.org> 에서 그대로 돌릴 수 있습니다. (Playground 는 `serde` 와 `serde_json` 을 지원합니다.)

### 1. derive 한 줄

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let u: User = serde_json::from_str(r#"{"name": "Mina", "age": 31}"#).unwrap();
    println!("{u:?}");
    println!("{}", serde_json::to_string(&u).unwrap()); // {"name":"Mina","age":31}
}
```

- `derive` 는 컴파일 시점에 "이 구조체를 JSON 으로/에서 바꾸는 코드"를 자동으로 써 줍니다. (C++ 템플릿 메타프로그래밍이나 Python 의 pydantic 이 하는 일을 컴파일러가 해 주는 셈)
- `from_str` 은 **반환 타입을 보고** 어떤 타입으로 읽을지 정합니다 (`let u: User` 또는 `from_str::<User>`).
- 모양이 안 맞으면 `Err` 가 나옵니다 (`unwrap()` 은 실습에서만! 실전은 `?`).
- **구조체의 필드 순서 = JSON 키 순서**입니다. (이 장에서 세 언어의 출력이 바이트 단위로 같아지는 비결)

### 2. 이름과 선택 필드: 자주 쓰는 속성들

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]            // message_count -> "messageCount"
struct Summary {
    message_count: usize,
    #[serde(rename = "type")]                  // type 은 Rust 예약어라 필드명으로 못 씀
    kind: String,
    #[serde(default)]                          // 없으면 Default::default() (빈 Vec)
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]   // None 이면 키 자체를 쓰지 않음
    note: Option<String>,
}
```

| 속성 | 하는 일 |
|---|---|
| `rename_all = "snake_case" / "camelCase" / "lowercase" ...` | 이름 규칙 일괄 변환 |
| `rename = ".."` | 한 필드/variant 만 이름 변경 |
| `default` | 입력에 없으면 기본값 (`Option` 은 없어도 `None` 이 되어 필요 없지만, `Vec` 등은 필요) |
| `skip_serializing_if = "경로"` | 조건이 참이면 출력에서 생략 (`Option::is_none`, `Vec::is_empty`) |
| `skip` | 읽기/쓰기에서 모두 제외 |
| `deny_unknown_fields` | 모르는 필드가 있으면 에러 (기본은 **무시**) |

> ⚠️ **C++/Python 개발자가 헷갈리는 점**: serde 의 기본값은 "**모르는 필드는 조용히 무시**"입니다. API 가 새 필드를 추가해도 안 깨지는 장점이 있지만, 오타 난 필드도 조용히 지나간다는 단점이 있어요. `deny_unknown_fields` 는 설정 파일처럼 "엄격해야 할 때"만 쓰세요.

### 3. enum 과 태그: Messages API 의 content block

Claude Messages API 는 블록마다 `"type"` 값으로 종류를 구분합니다.

```json
{"type": "text", "text": "안녕하세요"}
{"type": "tool_use", "id": "toolu_01...", "name": "calculator", "input": {"expression": "12 * 34"}}
{"type": "tool_result", "tool_use_id": "toolu_01...", "content": "408"}
```

Rust 에서는 이것이 정확히 **"내부 태그(internally tagged) enum"** 입니다.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KnownBlock {
    Text { text: String },
    Thinking { thinking: String, signature: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}
```

enum 의 태그 표현 방식은 4가지입니다 (같은 `Text { text: "hi" }` 가 JSON 으로 어떻게 나가는지):

| 속성 | JSON |
|---|---|
| (기본) 외부 태그 | `{"Text": {"text": "hi"}}` |
| `tag = "type"` 내부 태그 | `{"type": "Text", "text": "hi"}` ← **API 의 모양** |
| `tag = "t", content = "c"` 인접 태그 | `{"t": "Text", "c": {"text": "hi"}}` |
| `untagged` | `{"text": "hi"}` (태그 없음, 모양으로 추측) |

> 💡 **쉬운 설명 — 왜 enum 인가요?**
> "블록은 text 이거나 tool_use 이거나 ..." 중 **정확히 하나**입니다. Ch05 에서 배운 enum 이 바로 이런 "여러 모양 중 하나"를 표현하는 도구예요. Python 은 `dict["type"]` 으로 분기하고, C++ 은 `std::variant` 에 직접 분기 코드를 씁니다. Rust 는 `match` 가 모든 경우를 빠짐없이 다루는지 컴파일러가 검사해 줍니다.

### 4. 문자열 **또는** 배열: `untagged`

Messages API 의 `content` 는 두 가지 모양이 가능합니다.

```json
{"role": "user", "content": "그냥 문자열"}
{"role": "user", "content": [{"type": "text", "text": "블록 배열"}]}
```

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]                      // 태그 없음: 위에서부터 차례로 시도
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}
```

`untagged` 는 variant 를 **선언 순서대로 시도**해서 처음 성공하는 것을 씁니다. 단점은 모두 실패했을 때 에러 메시지가 불친절하다는 것입니다: `data did not match any variant of untagged enum MessageContent`.

### 5. 모르는 블록은 버리지 말고 "그대로" 들고 있기 (에이전트의 핵심 패턴)

API 는 계속 진화합니다. `redacted_thinking`, `server_tool_use`, `web_search_tool_result` ... 처럼 **우리 코드가 모르는 블록 타입이 언제든 추가**됩니다. 에이전트는 이런 블록을 만나도 (1) 죽으면 안 되고, (2) **다음 요청에 assistant 메시지를 그대로 돌려보낼 때 한 글자도 바꾸면 안 됩니다** (특히 `thinking` 의 `signature`).

선택지 세 가지:

| 방법 | 동작 | 평가 |
|---|---|---|
| 그냥 `KnownBlock` 만 | 모르는 type → 파싱 에러 | ❌ 새 블록이 오면 에이전트가 죽음 |
| `#[serde(other)] Unknown` | 모르는 type → 단위 variant `Unknown` | ⚠️ 죽진 않지만 **내용을 버립니다** → 되돌려 보낼 수 없음 |
| `untagged` 래퍼 + `Unknown(Value)` | 모르는 type → 원본 JSON 을 `Value` 로 보관 | ✅ **무손실 왕복** |

이 장이 쓰는 타입 (ch17 캡스톤에서도 같은 모양을 씁니다):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentBlock {
    Known(KnownBlock),          // 먼저 시도: type 이 text / thinking / tool_use / tool_result 이고 모양이 맞으면
    Unknown(serde_json::Value), // 실패하면 원본 그대로
}
```

```rust
// 사용할 때는 중첩 패턴으로 풀어 씁니다.
match block {
    ContentBlock::Known(KnownBlock::Text { text }) => println!("{text}"),
    ContentBlock::Known(KnownBlock::ToolUse { name, input, .. }) => run_tool(name, input),
    ContentBlock::Known(_) => {}
    ContentBlock::Unknown(raw) => println!("모르는 블록: {raw}"),   // 그냥 보관하고 되돌려 보냄
}
```

> ⚠️ **트레이드오프**: `untagged` 는 "Known 으로 읽기 **실패하면 전부** Unknown" 입니다. 즉 `{"type": "text"}` 처럼 text 블록인데 `text` 필드가 빠진 **잘못된 블록도 에러 없이 Unknown 이 됩니다**. 이 장의 `== bad input ==` 4번이 그 예입니다. 관대함(안 죽음)과 엄격함(오류 발견)의 맞교환이며, 에이전트에서는 보통 관대함이 낫습니다. 엄격하게 하고 싶으면 `Unknown` 으로 빠지기 전에 `tracing`/로그로 남기세요.

### 6. 타입 있는 파싱 vs `serde_json::Value`

```rust
use serde_json::{json, Value};

let v: Value = serde_json::from_str(r#"{"a": [1, 2, {"b": "x"}]}"#)?;
let x = v["a"][2]["b"].as_str();           // Option<&str> — 없으면 panic 이 아니라 Null/None
let built = json!({ "model": "m", "messages": [ {"role": "user", "content": "hi"} ] });
```

| | 타입 있는 `T` | `Value` |
|---|---|---|
| 검증 | 읽는 순간 전부 | 쓸 때마다 `as_str()` 등으로 |
| 오타/필드 누락 | 컴파일/파싱 시점에 발견 | 런타임에 `None` |
| 유연성 | 구조가 고정 | 무엇이든 담음 |
| 쓰임 | 우리가 **아는** 부분 | tool 의 `input`(도구마다 다름), JSON Schema, 모르는 블록 |

실전 규칙: **아는 부분은 타입으로, 모르거나 자유 형식인 부분만 `Value` 로.** 이 장의 `Request` 는 `tools[].input_schema`, `tool_use.input`, Unknown 블록만 `Value` 입니다.

> ⚠️ **키 순서 주의 (세 언어 출력을 맞추는 법)**
> - 구조체: 필드 **선언 순서**대로 출력 → 데이터 파일의 키 순서와 선언 순서를 맞췄습니다.
> - `serde_json::Value` 의 객체: 기본 설정에서는 키가 **알파벳 순으로 정렬**됩니다 (`{"b":1,"a":2}` → `{"a":2,"b":1}`). 삽입 순서를 지키려면 `serde_json` 의 `preserve_order` 기능을 켜야 합니다. 이 장의 데이터는 `Value` 안의 키를 알파벳 순으로 적어 두어서 영향이 없어요. (🔬 실험 D)
> - C++ `nlohmann::json` 도 기본은 정렬이라 `ordered_json` 을 썼고, Python `dict` 는 삽입 순서입니다.
> - JSON 의미상 객체의 키 순서는 중요하지 않지만, **서명(`signature`)이나 캐시 키, 테스트의 문자열 비교**처럼 바이트가 중요한 곳에서는 문제가 됩니다.

### 7. 에러 읽기: Syntax vs Data

```rust
match serde_json::from_str::<Message>(text) {
    Ok(m) => ...,
    Err(e) => {
        println!("{e}");                 // 사람이 읽는 메시지 (줄/열 포함)
        match e.classify() {
            serde_json::error::Category::Syntax | serde_json::error::Category::Eof => /* JSON 이 깨짐 */,
            serde_json::error::Category::Data => /* JSON 은 맞는데 우리 타입과 모양이 다름 */,
            serde_json::error::Category::Io => /* 읽기 실패 */,
        }
    }
}
```

실제 메시지 예:

```text
EOF while parsing a value at line 1 column 25                                  (Eof)
unknown variant `robot`, expected `user` or `assistant` at line 1 column 15       (Data)
missing field `content` at line 1 column 15                                     (Data)
invalid type: string "80", expected u16 at line 1 column 13                     (Data)
data did not match any variant of untagged enum MessageContent at line 1 column 28  (Data)
```

> ⚠️ 에이전트에서 **API 응답 파싱 실패**는 흔한 일입니다. 에러에 "어느 줄/열에서, 무엇을 기대했는지"가 들어 있으니, 로그에 `{e}` 를 꼭 남기세요. 서버가 준 본문도 같이 남기면 디버깅이 훨씬 쉽습니다.

### 8. 출력 형식과 유니코드

- `to_string` → 공백 없는 compact (`{"a":1}`). Python 의 기본 `json.dumps` 는 `{"a": 1}` 처럼 공백이 있으니 비교할 땐 `separators=(",", ":")` 를 쓰세요.
- `to_string_pretty` → 2칸 들여쓰기. Python `indent=2`, C++ `dump(2)` 와 같은 결과입니다.
- 한글/이모지: serde_json 은 **그대로 UTF-8** 로 출력합니다. Python 은 기본이 `\uXXXX` 이스케이프(`ensure_ascii=True`)이므로 `ensure_ascii=False` 가 필요합니다.
- 빈 배열/객체는 세 언어 모두 `[]` / `{}` 로 나옵니다.

## 🧯 자주 만나는 컴파일 에러

**1. derive 를 안 붙였는데 `to_string` / `from_str` 을 부름 (E0277)**
```
error[E0277]: the trait bound `User: serde::Serialize` is not satisfied
```
→ 의미: `User` 에 `Serialize` 가 구현되어 있지 않아요. 고치는 법: `#[derive(Serialize)]` (읽기는 `Deserialize`) 를 추가하고 `use serde::{Serialize, Deserialize};` 를 확인하세요.

**2. serde 의 `derive` 기능이 꺼져 있음**
```
error: cannot find derive macro `Serialize` in this scope
```
→ 의미: `Cargo.toml` 에서 serde 의 `derive` feature 가 없어요. 이 저장소는 workspace 설정에 이미 켜져 있습니다 (`serde = { workspace = true }`). 직접 프로젝트를 만든다면 `serde = { version = "1", features = ["derive"] }`.

**3. 필드 타입이 Serialize/Deserialize 가 아님 (E0277)**
```
error[E0277]: the trait bound `Instant: Serialize` is not satisfied
```
→ 의미: 구조체 안의 어떤 필드 타입이 serde 를 지원하지 않아요. 고치는 법: 그 필드를 `#[serde(skip)]` 하거나 지원되는 타입(`u64` 타임스탬프 등)으로 바꾸세요.

**4. 반환 타입을 알 수 없음 (E0283)**
```
error[E0283]: type annotations needed
  = note: the type must implement `serde_core::de::Deserialize<'_>`
```
→ 의미: `let v = serde_json::from_str(s)?;` 처럼 어떤 타입으로 읽을지 알 수 없어요. 고치는 법: `let v: Request = ...` 또는 `from_str::<Request>(s)`.

**5. 에러 타입 변환 (E0277, `?`)**
```
error[E0277]: `?` couldn't convert the error to `MyError`
```
→ 의미: `from_str(..)?` 가 `serde_json::Error` 를 돌려주는데 우리 함수의 에러 타입이 그걸 못 받아요. 고치는 법: Ch06 처럼 `impl From<serde_json::Error> for MyError` 를 만들거나 `Box<dyn Error>` / `anyhow` 를 쓰세요.

**6. 런타임 에러 (컴파일은 됨): `invalid type: map, expected a string`** — `untagged`/내부 태그의 모양이 실제 JSON 과 달라서 생깁니다. 에러 메시지의 줄·열과 기대 타입을 보고 JSON 샘플과 enum 모양을 비교하세요.

## 🛠️ 실습: LLM 대화 로그 변환기

### 무엇을 만드나요

`data/conversation.json` 은 실제 Claude Messages API 요청의 모양입니다 (`model`, `max_tokens`, `system`, `tools`, `messages`). 메시지에는 문자열 content, `thinking`/`text`/`tool_use`/`tool_result` 블록, 에러 결과(`is_error: true`), 그리고 모르는 블록(`redacted_thinking`)이 섞여 있고 한글/이모지/따옴표/줄바꿈도 들어 있어요.

프로그램은

1. JSON 을 **타입**으로 읽어서 (`parse_request`)
2. 블록 종류별 개수, 도구 사용 횟수, tool_use ↔ tool_result 짝지은 보고서를 출력하고
3. 일부러 망가뜨린 입력 5개가 어떤 종류의 에러(syntax/data)인지 분류하고
4. 타입 → JSON 왕복이 **무손실**인지 확인한 뒤
5. 처음 3개 메시지로 **새 요청**을 만들어 2칸 들여쓰기로 pretty-print 하고, camelCase 요약 JSON 을 출력합니다.

출력 예시 (일부):

```text
== messages ==
[0] user: text
[1] assistant: thinking, text, tool_use, tool_use
[2] user: tool_result, tool_result
...
[7] assistant: redacted_thinking, text
== tool calls ==
calculator {"expression":"12 * 34"} -> 408
calculator {"expression":"10 / 0"} -> ERROR: error: division by zero
get_version {} -> 1.4.2
== bad input ==
1: syntax
2: data
...
== round trip ==
lossless: yes
```

> 출력은 **세 언어에서 바이트 단위로 같아야** 합니다: 키 순서, 들여쓰기, 빈 `[]`/`{}`, 한글/이모지(이스케이프 없이), 마지막 줄바꿈까지.

### 파일 구성

| 경로 | 설명 |
|---|---|
| `data/conversation.json` | 입력 데이터 (Messages API 모양) |
| `python/main.py` | dataclass + 수동 변환 + `json.dumps(indent=2, ensure_ascii=False)` |
| `cpp/main.cpp` | `nlohmann::ordered_json` + `std::variant` + 수동 변환 |
| `rust/src/main.rs` | **실습 파일**: serde 속성 추가 + `todo!()` |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (chapters/ch15_serde_json 에서)

```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch15_serde_json                       # 실습
cargo test -p ch15_serde_json                         # 실습 테스트
cargo run -q -p ch15_serde_json --example solution    # 정답
../../scripts/compare.sh ch15                         # 세 언어 비교 (--solution 이면 정답 기준)
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `Role` 에 `rename_all = "lowercase"` | 이름 변환 |
| 2 | `KnownBlock` 에 `tag = "type"`, `rename_all = "snake_case"`, `is_error` 의 `default`/`skip_serializing_if` | 내부 태그 enum, 선택 필드 |
| 3 | `ContentBlock`, `MessageContent` 를 `untagged` 로 | 무손실 Unknown 패턴 |
| 4 | `Request` 의 `system`/`stop_sequences` 처리 | `default`, `skip_serializing_if` |
| 5 | `parse_request`, `parse_message` | `serde_json::from_str`, 반환 타입 추론 |
| 6 | `block_kind` | 중첩 패턴 매칭, `Value` 다루기 |
| 7 | `tools_used` | `if let` + `entry` API |
| 8 | `tool_call_report` | id 로 짝짓기, 빌려서 담는 `BTreeMap<&str, ..>` |
| 9 | `build_followup` | `clone`, `last_mut`, 새 `Request` 구성 |
| 10 | `Summary` 에 `rename_all = "camelCase"`, `summarize` | 집계, `matches!` |
| 11 | `classify_error` | `Error::classify()` |
| 12 | `round_trip_lossless` | `Value` 비교, 무손실 확인 |

### 완료 기준

- `cargo test -p ch15_serde_json` 전부 통과 (테스트는 인라인 JSON 을 써서 실행 위치와 무관합니다).
- `../../scripts/compare.sh ch15` 에서 Python / C++ / Rust 출력이 같다고 보고.
- 정답은 `cargo clippy -p ch15_serde_json --examples -- -D warnings` 도 통과합니다. 여러분의 풀이도 해 보세요.

### 🔬 실험 목록

| 실험 | 어디서 | 확인할 것 |
|---|---|---|
| A | TODO(2) | `tag` 속성을 지우면 에러 메시지가 어떤 모양을 기대하나요? (외부 태그) |
| B | TODO(3) | `ContentBlock` 에서 `Unknown` 을 `Known` 앞으로 옮기면? 모든 블록이 Unknown 이 됩니다 → `lossless` 는 yes 지만 `tools_used` 가 0 |
| C | TODO(12) | 데이터에 `"extra_field": 1` 추가 → 기본은 조용히 무시 (왕복에서는 사라지므로 lossless 가 no!). `deny_unknown_fields` 를 붙이면 에러. 어느 쪽이 에이전트에 안전할까요? |
| D | TODO(12) | `Value` 의 키 순서: `{"b":1,"a":2}` → 출력은 `{"a":2,"b":1}`. `preserve_order` 기능을 켜면(Cargo.toml `serde_json = { workspace = true, features = ["preserve_order"] }`) 어떻게 바뀌나요? (Cargo.lock 이 바뀌므로 실험 후 되돌리세요) |

## 🤖 바이브 코딩 포인트

AI 가 만든 serde 코드를 볼 때:

- **`Value` 남용**: 모든 것을 `serde_json::Value` 로 받아 `v["a"]["b"].as_str().unwrap()` 를 줄줄이 쓰면 위험합니다. 아는 구조는 구조체/enum 으로 모델링하고, 자유 형식 부분만 `Value` 로 두도록 요청하세요.
- **모르는 블록 처리**: content block enum 에 catch-all 이 있는지 확인하세요. 없으면 API 에 새 블록이 생기는 날 에이전트가 파싱 에러로 죽습니다. "`#[serde(untagged)]` 래퍼 + `Unknown(Value)` 로 무손실 보존" 이라고 지시하세요.
- **`unwrap()`/`expect()`**: 외부 입력(API 응답)을 `from_str(..).unwrap()` 하고 있으면 패닉의 원인입니다. `?` 와 `Result` 로 에러를 전파하게 하세요.
- **속성 확인**: `rename_all`/`tag` 가 실제 API 모양과 일치하는지 JSON 샘플로 **왕복 테스트**를 요청하세요 (`from_str` → `to_value` == 원본 `Value`). 이 장의 `round_trip_lossless` 가 그 테스트입니다.
- **`skip_serializing_if` 누락**: `Option` 필드를 `None` 인 채로 `"is_error": null` 로 보내면 API 가 거부할 수 있어요. 선택 필드는 `skip_serializing_if = "Option::is_none"` 이 있는지 확인하세요.
- **`#[serde(deny_unknown_fields)]` 를 API 응답 타입에** 붙이면 API 가 필드를 추가하는 날 깨집니다. (우리가 **받는** 타입에는 보통 붙이지 않습니다.)
- **프롬프트 팁**: "Claude Messages API 의 content block (text, thinking, tool_use, tool_result)을 serde 로 모델링해 줘. 모르는 block type 은 serde_json::Value 로 그대로 보존해서 다음 요청에 돌려보낼 수 있어야 하고, 왕복 테스트를 포함해 줘."

## ✅ 셀프 체크

1. `#[serde(tag = "type", rename_all = "snake_case")]` 가 붙은 enum 의 `ToolUse { id, name, input }` 은 JSON 에서 어떤 모양인가요?
   <details><summary>답</summary><code>{"type":"tool_use","id":"..","name":"..","input":{..}}</code> — 태그 필드 <code>type</code> 이 항상 맨 앞에 오고, 나머지 필드는 선언 순서대로 이어집니다.</details>
2. `#[serde(other)] Unknown` 대신 `Unknown(serde_json::Value)` 를 쓰는 이유는?
   <details><summary>답</summary><code>other</code> 는 값 없는 단위 variant 라서 모르는 블록의 내용을 버립니다. 에이전트는 모르는 블록(예: <code>redacted_thinking</code>)을 다음 요청에 그대로 돌려보내야 하므로 원본을 <code>Value</code> 로 보존해야 합니다.</details>
3. `Option<bool>` 필드에 `#[serde(default, skip_serializing_if = "Option::is_none")]` 를 붙이면 읽을 때와 쓸 때 각각 어떻게 되나요?
   <details><summary>답</summary>읽을 때: 키가 없으면 <code>None</code>. 쓸 때: <code>None</code> 이면 키를 아예 출력하지 않습니다 (<code>null</code> 로 쓰지 않음).</details>
4. `serde_json::Error::classify()` 의 `Syntax` 와 `Data` 는 무엇이 다른가요?
   <details><summary>답</summary><code>Syntax</code>: JSON 문법 자체가 깨짐(따옴표 누락 등). <code>Data</code>: JSON 은 올바르지만 우리 타입과 모양/값이 다름(필드 누락, 잘못된 enum 값, 타입 불일치). <code>Eof</code> 는 입력이 중간에 끊긴 경우입니다.</details>
5. 구조체를 `to_string_pretty` 로 출력했을 때 키 순서는 무엇이 결정하나요? `Value` 는요?
   <details><summary>답</summary>구조체는 필드 선언 순서. <code>Value</code> 의 객체는 기본적으로 알파벳 순(<code>BTreeMap</code>)이며, <code>preserve_order</code> 기능을 켜면 삽입 순서입니다.</details>

## 📚 더 알아보기

- [Serde 공식 문서](https://serde.rs/) — 특히 [Attributes](https://serde.rs/attributes.html), [Enum representations](https://serde.rs/enum-representations.html)
- [`serde_json` API 문서](https://docs.rs/serde_json) — `Value`, `json!` 매크로, `Error::classify`
- The Rust Book: [Ch06 Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html), [Ch09 Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- Claude API 문서 — Messages API 의 [content blocks](https://docs.claude.com/en/api/messages) 와 tool use 가이드
- 비교: [nlohmann/json](https://json.nlohmann.me/) (`ordered_json`), Python [`json`](https://docs.python.org/3/library/json.html)
