# Ch17. 캡스톤: 미니 AI 에이전트

> 모델에게 과제를 주고, 모델이 요청하는 **도구를 실행해 결과를 돌려주는 루프**를 Rust 로 직접 만듭니다. 지금까지 배운 trait · serde · async · 에러 처리가 전부 한 프로그램에 모여요.  ·  예상 소요: 이론 50분 / 실습 120분  ·  선행: Ch08(trait/dyn), Ch14(async), Ch15(serde), Ch16(reqwest/에러)

## 🎯 학습 목표

- **에이전트 루프**(메시지 → 모델 → `stop_reason == "tool_use"` → 도구 실행 → `tool_result` → 반복)를 설명하고 구현할 수 있다.
- Claude Messages API 의 JSON 프로토콜(`content` 블록, `tool_use` / `tool_result`, `stop_reason`)을 serde 타입으로 모델링할 수 있다.
- 비동기 `Tool` trait 와 `ToolRegistry`(`Vec<Box<dyn Tool>>`)를 설계하고, **왜 `async fn` 대신 `BoxFuture` 를 쓰는지** 설명할 수 있다.
- 도구 실패를 `is_error` 결과로 모델에게 돌려주고, 최대 반복 횟수·`refusal`·`max_tokens` 같은 **안전장치**를 넣을 수 있다.
- 목(mock) 서버와 실제 API 를 같은 코드로 오가는 설정(환경 변수)과 API 키 안전 수칙을 안다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| HTTP + API 호출 | libcurl (직접) | 공식 `anthropic` SDK | `reqwest` (직접) — 공식 Rust SDK 는 없어요 |
| JSON 모델링 | `nlohmann::json` (동적) | SDK 가 만든 pydantic 객체 | `serde` 구조체/enum + `serde_json::Value` |
| 도구 인터페이스 | `class Tool { virtual ... = 0; }` | `class Tool(ABC)` | `trait Tool` + `Box<dyn Tool>` |
| 비동기 도구 | (여기서는 동기) | (여기서는 동기) | `fn run(..) -> BoxFuture<..>` |
| 도구 레지스트리 | `vector<unique_ptr<Tool>>` | `list[Tool]` | `Vec<Box<dyn Tool>>` |
| 도구 실패 | 예외 → `is_error` 결과로 변환 | 예외 → `is_error` 결과로 변환 | `Result<String, String>` → `ToolOutcome` |
| 설정 | `getenv` | `os.environ` | `std::env::var(..).ok()` → 순수 함수로 전달 |

## 📘 이론

### 1. 에이전트란? — "도구를 쓰는 루프"일 뿐입니다

LLM 은 글자만 만들어 낼 수 있어요. 그래서 우리가 **"이런 도구들이 있어요"** 라고 알려 주면, 모델은 답 대신
**"calculator 를 `{"expression":"(12+30)*2"}` 로 불러 주세요"** 라고 요청할 수 있습니다. 우리 프로그램이 그 도구를 **실제로 실행**하고
결과를 돌려주면, 모델이 그 결과를 보고 다음 행동을 정해요. 이 왕복을 **끝날 때까지 반복**하는 것이 에이전트입니다.

```text
┌──────────────────────────── 우리 프로그램 (에이전트 루프) ────────────────────────────┐
│                                                                                        │
│  messages = [ user: "과제" ]                                                           │
│        │                                                                               │
│        ▼                                                                               │
│  ┌───────────┐  POST /v1/messages   ┌──────────────┐                                   │
│  │ LlmClient │ ───────────────────► │  Claude API  │  (또는 tools/mock_server.py)      │
│  │ (reqwest) │ ◄─────────────────── │   "모델"     │                                   │
│  └───────────┘  content[], stop_reason └──────────────┘                                │
│        │                                                                               │
│        ▼                                                                               │
│  stop_reason 은?                                                                       │
│   ├─ end_turn   ──► 최종 답변 출력, 종료                                               │
│   ├─ refusal / max_tokens ──► 에러로 종료                                              │
│   └─ tool_use   ──► messages 에 assistant content 를 "그대로" 추가                     │
│                          │                                                             │
│                          ▼                                                             │
│                  ┌───────────────┐   name 으로 찾기   ┌─────────────────────────┐      │
│                  │ ToolRegistry  │ ─────────────────► │ dyn Tool: calculator,   │      │
│                  │ Vec<Box<dyn>> │ ◄───────────────── │ word_count, read_note.. │      │
│                  └───────────────┘  Ok(..)/Err(..)    └─────────────────────────┘      │
│                          │                                                             │
│                          ▼                                                             │
│          messages 에 user 메시지 "하나" 추가: [tool_result, tool_result, ...]          │
│                          │                                                             │
│                          └──────────────► 다시 맨 위로 (최대 10번)                     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

> 💡 **쉬운 설명 — 비서와 심부름꾼**
> 모델은 "생각만 할 수 있는 비서", 우리 프로그램은 "손발이 있는 심부름꾼"이에요. 비서가 쪽지에 "이 파일 읽어 와 줘"라고 쓰면(`tool_use`),
> 심부름꾼이 다녀와서 결과를 쪽지에 적어 건네고(`tool_result`), 비서가 "이제 답을 알겠어요"(`end_turn`)라고 할 때까지 반복합니다.
> 심부름꾼이 해야 할 일은 **쪽지를 정확히 전달하는 것**과 **위험한 심부름은 거절하는 것**이에요.

### 2. Messages API 프로토콜

요청 (`POST https://api.anthropic.com/v1/messages`):

```json
{
  "model": "claude-opus-5-5",
  "max_tokens": 16000,
  "system": "You are a concise assistant. ...",
  "tools": [
    {"name": "calculator", "description": "...",
     "input_schema": {"type": "object", "properties": {"expression": {"type": "string"}}, "required": ["expression"]}}
  ],
  "messages": [
    {"role": "user", "content": [{"type": "text", "text": "Compute (12+30)*2"}]}
  ],
  "output_config": {"effort": "medium"},
  "fallbacks": "default"
}
```

헤더: `content-type: application/json`, `x-api-key: $ANTHROPIC_API_KEY`, `anthropic-version: 2026-10-21`, `anthropic-beta: server-side-fallback-2026-07-01`

응답:

```json
{
  "id": "msg_...", "type": "message", "role": "assistant", "model": "claude-opus-5-5",
  "content": [
    {"type": "thinking", "thinking": "", "signature": "..."},
    {"type": "text", "text": "I will calculate that."},
    {"type": "tool_use", "id": "toolu_01", "name": "calculator", "input": {"expression": "(12+30)*2"}}
  ],
  "stop_reason": "tool_use",
  "usage": {"input_tokens": 100, "output_tokens": 42}
}
```

그다음 요청의 `messages` 는 이렇게 **쌓여 갑니다** (API 는 대화를 기억하지 않아요 — 매번 **전체 기록**을 보냅니다):

```json
[
  {"role": "user",      "content": [{"type": "text", "text": "Compute (12+30)*2"}]},
  {"role": "assistant", "content": [ ...위 응답의 content 를 한 글자도 바꾸지 않고... ]},
  {"role": "user",      "content": [
      {"type": "tool_result", "tool_use_id": "toolu_01", "content": "84"}
  ]}
]
```

**꼭 지켜야 할 규칙 5가지** (어기면 API 가 400 으로 거절하고, 이 챕터의 목 서버도 똑같이 검사합니다):

1. **assistant 의 `content` 는 그대로** 되돌려 보냅니다. `thinking` 블록(`signature` 포함)을 지우거나 고쳐 쓰면 안 돼요.
2. 한 턴에 `tool_use` 가 **여러 개**(병렬 호출)일 수 있고, 그 **모든 `tool_result` 를 user 메시지 하나**에 담아야 합니다. `tool_use_id` 로 짝을 맞춥니다.
3. 모든 `tool_use` 에는 **바로 다음 메시지**에 대응하는 `tool_result` 가 있어야 해요 (하나라도 빠지면 400).
4. 도구가 실패하면 `"is_error": true` 와 함께 **사유를 `content` 에** 담아 돌려줍니다. 모델이 사유를 보고 다른 방법을 찾아요.
5. `temperature` / `top_p` / `top_k` 는 이 API 버전(`2026-10-21`)의 모델에서 **보내면 400** 입니다. 사고(thinking)는 항상 켜져 있고, 깊이는 `output_config.effort` 로 조절해요.

### `stop_reason` 처리표

| 값 | 의미 | 우리가 할 일 |
|---|---|---|
| `end_turn` | 모델이 답을 마침 | 텍스트 블록을 모아 최종 답변으로 |
| `tool_use` | 도구를 요청함 | 도구 실행 → `tool_result` → 다음 반복 |
| `max_tokens` | 출력 한도에서 잘림 | 잘린 `tool_use` 는 실행하면 안 됨 → 에러 |
| `refusal` | 정책상 거절 (`content` 가 비어 있을 수 있음) | **`content` 를 읽기 전에** 먼저 확인 → 에러 |
| `pause_turn` | 서버 도구가 중간에 멈춤 | assistant 턴을 그대로 추가하고 다시 요청 |

그리고 **반복 횟수 상한**(`MAX_ITERATIONS = 10`)을 둡니다. 모델이 같은 도구를 계속 부르는 무한 루프는 **돈이 새는** 사고예요.

> 💡 **refusal fallback**: 정책 분류기가 요청을 거절하면, `"fallbacks": "default"` + 헤더 `anthropic-beta: server-side-fallback-2026-07-01` 를 보낸 요청은
> **서버가 대체 모델로 같은 요청을 다시 처리**해 줍니다. 그래도 거절되면 `stop_reason: "refusal"` 이 와요. (목 서버는 이 옵션을 그냥 받아 줍니다.)

### 3. serde 로 모델링하기 — "원본은 Value 로, 읽을 때만 타입으로"

```rust
#[derive(Serialize, Deserialize, Clone)]
struct Message { role: String, content: Vec<serde_json::Value> }   // ← 원본 JSON 그대로 보관

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]                  // Ch15: "type" 필드로 variant 선택
enum Block {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    Thinking,                                                       // 내용은 몰라도 돼요
    #[serde(other)] Other,                                          // 처음 보는 블록도 에러 없이 통과
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum StopReason { EndTurn, ToolUse, MaxTokens, Refusal, PauseTurn, StopSequence, #[serde(other)] Unknown }
```

**왜 `content` 를 `Vec<Block>` 으로 통째로 바꾸지 않나요?** `thinking` 블록의 `signature` 처럼 **우리가 이름도 모르는 필드**나, 앞으로 생길 새 블록 종류는
타입으로 바꾸는 순간 사라질 수 있어요. 사라진 채 되돌려 보내면 API 가 거절합니다. 그래서:

- **보관·전송용**: `Vec<Value>` (원본 그대로 → `response.content` 를 통째로 `messages` 에 move)
- **읽기용**: `response.blocks() -> Vec<Block>` (원본을 복제해서 타입 뷰를 만듦. 이해 못 하는 블록은 `Block::Other`)

Python SDK 도 같은 일을 합니다 (`messages.append({"role": "assistant", "content": response.content})`).

### 4. 비동기 `Tool` trait — 왜 `async fn` 이 아니라 `BoxFuture` 인가

Rust 1.75 부터 trait 안에 `async fn` 을 쓸 수 있지만, **그런 trait 는 `dyn` 으로 쓸 수 없어요.**

```rust
trait Tool { async fn run(&self) -> String; }
let v: Vec<Box<dyn Tool>> = ...;     // ❌
```

```text
error[E0038]: the trait `Tool` is not dyn compatible
  |
1 | trait Tool { async fn run(&self) -> String; }
  |       ----            ^^^ ...because method `run` is `async`
```

이유: `async fn` 은 호출마다 **서로 다른 크기의 Future 타입**을 돌려주는데, `dyn Tool` 의 vtable(Ch08)은 "반환 타입의 크기"가 하나로 정해져 있어야 해요.
해결: **Future 를 `Box` 에 담아** 크기를 고정합니다.

```rust
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>>;
}

impl Tool for WordCount {
    fn run<'a>(&'a self, input: &'a Value) -> BoxFuture<'a, Result<String, String>> {
        Box::pin(async move { /* 평범한 async 코드 */ })       // ← "async 블록을 Box 에 담는" 관용구
    }
}
```

| 선택지 | 장점 | 단점 |
|---|---|---|
| **`BoxFuture` (이 챕터)** | 외부 크레이트 없음, `dyn` OK, 원리가 보임 | `Box::pin(async move {..})` 보일러플레이트 |
| `#[async_trait]` 크레이트 | 문법이 깔끔 (`async fn` 처럼 씀) | 매크로 마법 (실제로는 위와 같은 일을 함) |
| enum 디스패치 (`enum AnyTool { Calc(..), .. }`) | 박싱 없음, `dyn` 불필요 | 도구를 추가할 때마다 enum 과 `match` 수정 |

> 💡 **쉬운 설명 — 왜 Box 가 필요해요?**
> 상자(`Vec`)에 물건을 담으려면 칸 크기가 같아야 해요. `async fn` 이 만드는 "할 일 목록(Future)"은 도구마다 크기가 달라서, 모두 **같은 크기의 포장 상자(`Box`)** 에 넣어 줍니다.
> `Pin` 은 "상자 안의 물건을 옮기지 마세요" 라는 스티커예요. 지금은 `Box::pin(...)` 이라고 외워 두면 충분합니다.

> 🤖 AI 가 만든 코드에서 `#[async_trait]` 가 자주 보여요. 위 표처럼 "같은 일을 편하게 해 주는 매크로"라고 이해하면 됩니다. 새 프로젝트에서 그대로 써도 좋아요.

AI 에게 요청할 때 trait 에 `Send + Sync` 가 붙는 이유: tokio 의 멀티스레드 런타임이 작업을 **다른 스레드로 옮길 수 있어서**(Ch13/14)
Future 와 도구가 스레드 사이를 건너도 안전해야 하기 때문이에요.

### 5. `ToolRegistry`, 그리고 "실패도 결과"

```rust
struct ToolRegistry { tools: Vec<Box<dyn Tool>> }          // Ch08 과 같은 모양

async fn execute(&self, name: &str, input: &Value) -> ToolOutcome {
    match self.get(name) {
        None => ToolOutcome { content: format!("unknown tool: {name}"), is_error: true },
        Some(tool) => match tool.run(input).await {
            Ok(content) => ToolOutcome { content, is_error: false },
            Err(content) => ToolOutcome { content, is_error: true },    // 에러를 "전파"하지 않고 "결과"로!
        },
    }
}
```

도구가 실패해도 **프로그램이 죽으면 안 돼요.** 모델은 "그 파일은 읽을 수 없대요" 라는 결과를 받으면 다른 방법을 시도하거나 사용자에게 사정을 설명할 수 있습니다.
이 챕터의 기본 과제에는 일부러 `../secret.txt` 읽기가 들어 있어요. `read_note` 가 거부하면 `is_error: true` 로 돌아가고, 모델은 그걸 알려 줍니다.

> ⚠️ **보안 — 도구 입력은 "믿지 않는 입력"입니다.** 도구는 모델이 시키는 대로 움직이는데, 모델은 (프롬프트 인젝션 등으로) 이상한 요청을 할 수 있어요.
> `read_note` 는 `Path::new(name).components()` 가 **파일 이름 하나(`Component::Normal`)** 일 때만 허용해서 `../`, 절대 경로, 하위 폴더를 모두 막습니다.
> 셸 명령 실행, 파일 쓰기 같은 도구를 AI 에게 만들어 달라고 할 때는 **허용 목록(allowlist)과 경로 검사**부터 확인하세요.

### 6. LLM 클라이언트를 trait 로 — 테스트를 위한 구멍

```rust
trait Llm: Send + Sync {
    fn create<'a>(&'a self, req: &'a MessagesRequest<'a>) -> BoxFuture<'a, Result<MessagesResponse, LlmError>>;
}
async fn run_agent(llm: &dyn Llm, ...) -> Result<AgentOutcome, AgentError>
```

진짜 `AnthropicClient`(reqwest)와 테스트용 `ScriptedLlm`(미리 정한 응답을 순서대로 돌려주는 가짜)이 같은 `Llm` trait 를 구현하니까,
**API 키도 네트워크도 없이** 에이전트 루프를 테스트할 수 있습니다. (이 챕터의 테스트가 정확히 그렇게 해요.)

### 7. 에러 설계 — 이번엔 세 겹

```text
LlmError   (thiserror) : Network / Api{status,message} / Decode     ← HTTP 계층 (Ch16 과 같은 방식)
AgentError (thiserror) : Llm(#[from] LlmError) / Refused / MaxTokens / MaxIterations(n) / UnexpectedStop
anyhow::Error          : main 에서 .context("agent run failed") 를 덧붙여 사용자에게 출력
```

각 층이 **자기 층의 종류**만 책임지고, 위층은 `#[from]` 으로 감쌉니다. `Api { status, message }` 에는 서버가 알려 준 이유(예: `` `temperature` may not be set ... ``)를 담아서 디버깅할 수 있게 해요.

## 🧯 자주 만나는 컴파일 에러

**1. `Tool` 이 dyn 호환이 아니에요 (E0038)** — 위 4절의 에러. `async fn` 을 trait 메서드에 쓰고 `Box<dyn Tool>` 을 만들 때 납니다.
고치기: 반환 타입을 `BoxFuture<'a, ...>` 로 바꾸고 `Box::pin(async move { .. })`, 또는 `#[async_trait]`.

**2. 빌림 충돌 (E0502)** — 루프 안에서 `messages` 를 요청에 빌려 준 채 `push` 하려고 할 때:

```text
error[E0502]: cannot borrow `messages` as mutable because it is also borrowed as immutable
   |         messages: &messages,        ← request 가 messages 를 빌려 가요
   ...
   |         messages.push(...)           ← 그런데 request/response 를 쓰는 중에 수정?
```
의미: Ch04 의 "읽는 참조가 살아 있는 동안은 수정 불가" 규칙입니다. 고치기: `llm.create(&request).await?` 가 끝난 뒤(= request 를 더 안 쓸 때)에 `push` 하세요.
`request` 를 블록 `{ }` 안에서 만들어 수명을 짧게 하는 것도 방법이에요.

**3. 값이 이동된 뒤에 사용 (E0382)** — `messages.push(Message { content: response.content, .. })` 로 `response.content` 를 **move** 한 다음 `response.blocks()` 를 부르면 에러예요.
고치기: `let blocks = response.blocks();` 를 **먼저** 하고 나서 `response.content` 를 move 합니다 (`blocks` 는 복제본이라 이후에도 사용 가능).

**4. 에러 타입 변환 실패 (E0277)** — `llm.create(..).await?` 에서 `LlmError` 를 `AgentError` 로 바꾸지 못할 때: `AgentError` 에 `#[error(transparent)] Llm(#[from] LlmError)` 를 추가하세요. (Ch16 과 같은 원리)

**5. `future cannot be sent between threads safely`** — 도구나 Future 안에 `Rc`/`RefCell` 같은 `!Send` 타입을 들고 `tokio::spawn`/`BoxFuture + Send` 에 넣을 때.
고치기: `Arc` / `Mutex` 로 바꾸거나 값을 `.await` 이전에 drop 하세요 (Ch13).

## 🛠️ 실습: 미니 에이전트 만들기

### 무엇을 만드나요

고정된 과제를 받아 4~5번의 왕복 끝에 답을 내는 에이전트입니다. 기본 과제:

> *Read the note 'notes.txt', count its words, compute (12+30)\*2, try to read '../secret.txt', and check the weather in Seoul. Then summarize.*

도구 4개: `calculator`, `word_count`, `read_note`(`data/` 안의 파일만), `get_weather`(고정 데이터).
**목 서버**(`tools/mock_server.py`)가 "모델" 역할을 대본대로 하므로, 출력이 매번 똑같아요. 목 서버는 실제 API 처럼 요청을 **검증**합니다 (헤더, 필수 필드, tool_use ↔ tool_result 짝, thinking 블록 유지 등).

예상 출력 (모든 언어 동일):

```text
task: Read the note 'notes.txt', count its words, compute (12+30)*2, try to read '../secret.txt', and check the weather in Seoul. Then summarize.
--- turn 1 (stop_reason: tool_use) ---
thinking: [block kept]
assistant: I will read the note and run the calculation in parallel.
tool_use: read_note {"name":"notes.txt"}
tool_use: calculator {"expression":"(12+30)*2"}
result: read_note ok Rust is a systems language.\nIt has ownership and borrowing.\nAgent...
result: calculator ok 84
--- turn 2 (stop_reason: tool_use) ---
thinking: [block kept]
assistant: Now I will count the words and also try a file outside data/.
tool_use: word_count {"text":"Rust is a systems language.\nIt has ownership and borrowin...
tool_use: read_note {"name":"../secret.txt"}
result: word_count ok 16
result: read_note ERROR path not allowed: ../secret.txt
--- turn 3 (stop_reason: tool_use) ---
thinking: [block kept]
assistant: One read was rejected, so I will not retry it. Checking the weather instead.
tool_use: get_weather {"city":"Seoul"}
result: get_weather ok 18C, clear
--- turn 4 (stop_reason: end_turn) ---
thinking: [block kept]
--- final answer ---
Summary:
- notes.txt has 16 words.
- (12+30)*2 = 84.
- Reading ../secret.txt failed: path not allowed: ../secret.txt
- Weather in Seoul: 18C, clear.
--- done: 4 turns, 5 tool calls, 1 tool errors ---
```

눈여겨볼 점: **turn 1 과 turn 2 에서 도구가 2개씩 병렬로 요청**되고, 결과는 user 메시지 하나로 돌아갑니다. turn 2 의 `read_note ../secret.txt` 는 **실패하는 도구 호출**이에요.

### 파일 구성

| 파일 | 설명 |
|---|---|
| `data/notes.txt` | `read_note` 가 읽는 메모 (단어 16개) |
| `python/main.py` | 공식 `anthropic` SDK. 루프는 **직접(manual) 작성**해서 Rust 와 구조가 같아요 (`pip install anthropic` 필요) |
| `cpp/main.cpp` | libcurl + nlohmann::json, `class Tool` 추상 클래스 + `ToolRegistry` |
| `rust/src/main.rs` | **실습 파일** (TODO 11개) |
| `rust/examples/solution.rs` | 정답 |
| `USES_MOCK_SERVER` | `compare.sh` 가 목 서버를 자동으로 켜고 끕니다 |

HTTP 연결(`AnthropicClient`), 계산기 파서(`eval_expr`), 출력 도우미, `main` 은 **제공**되어 있어서 에이전트 로직에 집중할 수 있어요.

### 실행 방법 (챕터 폴더에서)

```bash
# 터미널 A: 목 서버 (compare.sh 를 쓰면 필요 없어요)
python3 ../../tools/mock_server.py --port 8787

# 터미널 B
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x $(cat cpp/BUILD_FLAGS) && /tmp/x
cargo test -p ch17_capstone_agent                     # 가짜 LLM 으로 도는 단위 테스트 (네트워크 불필요)
cargo run -q -p ch17_capstone_agent                   # 실습
cargo run -q -p ch17_capstone_agent --example solution
../../scripts/compare.sh ch17                         # 세 언어 출력 비교 (목 서버 자동 실행)
../../scripts/compare.sh ch17 --solution
```

환경 변수:

| 변수 | 기본값 | 설명 |
|---|---|---|
| `LLM_BASE_URL` | `http://127.0.0.1:8787` | 목 서버. 실제 API 는 `https://api.anthropic.com` |
| `ANTHROPIC_API_KEY` | (목 서버일 때만) `mock-key` | 실제 API 에서는 **필수** — 없으면 시작 시 에러 |
| `LLM_MODEL` | `claude-opus-5-5` | 모델 이름 |
| `AGENT_TASK` | 위의 기본 과제 | 다른 과제로 바꿔 보기 |

목 서버 특수 과제 (테스트용): `AGENT_TASK=MOCK_REFUSAL` → `refusal`, `MOCK_MAX_TOKENS` → 잘린 답, `MOCK_LOOP` → 끝없이 도구 요청 (반복 상한 확인용).

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `MessagesResponse::blocks()` — 원본 JSON → 타입 있는 `Block` (모르면 `Other`) | serde `tag`, `from_value`, 관대한 파싱 |
| 2 | `Config::from_env_values()` — 기본값, 목 서버에서만 `mock-key` 허용 | 순수 함수 설계, `Option::filter`, 안전장치 |
| 3 | `parse_api_error()` — 에러 JSON 에서 `message` 추출 | serde 로 에러 응답 해석 |
| 4 | `Calculator::run` | `Box::pin(async move)`, `?` |
| 5 | `WordCount::run` | 문자열 처리 |
| 6 | `safe_note_path()` — 경로 탈출 거부 | `Path::components`, 입력 검증 |
| 7 | `ReadNote::run` — `tokio::fs` + 검사 | 비동기 파일 I/O |
| 8 | `ToolRegistry::definitions()` — API `tools` 배열 | 이터레이터 `map/collect` |
| 9 | `ToolRegistry::execute()` — 실패를 `is_error` 결과로 | `dyn Tool`, `Result` → `ToolOutcome` |
| 10 | `tool_result_block()` — `is_error` 는 실패일 때만 | `json!` 매크로 |
| 11 | **`run_agent()` 루프** — stop_reason 처리, content 그대로 되돌리기, tool_result 하나의 메시지로, 반복 상한 | 이 챕터 전체 |

TODO 1–10 은 작은 조각이고, 11 이 클라이맥스입니다. 11 번에는 단계별 틀(①~⑨)이 주석에 적혀 있어요.

### 완료 기준

`cargo test -p ch17_capstone_agent` 전부 통과 AND `../../scripts/compare.sh ch17` 에서 python vs cpp, python vs rust 모두 **출력이 똑같아요**.

### 🔬 실험 목록 (루프가 돌아간 뒤에!)

1. **thinking 블록 지우기** — TODO(11)⑥ 에서 `response.content` 대신 `text`/`tool_use` 블록만 걸러서 보내 보세요. 목 서버가 `400 ... Expected thinking ...` 으로 거절합니다. (실제 API 도 같아요.)
2. **tool_result 를 따로 보내기** — ⑨ 에서 도구마다 user 메시지를 따로 push 해 보세요 → `400 ... tool_use ids were found without tool_result blocks`.
3. **`tool_use_id` 틀리게 쓰기** — `tool_result_block` 에서 id 를 일부러 바꿔 보세요 → 400. (병렬 호출에서 id 가 짝을 맞추는 열쇠입니다.)
4. **반복 상한 확인** — `AGENT_TASK=MOCK_LOOP cargo run -q -p ch17_capstone_agent` → `agent did not finish within 10 iterations`. 상한을 없애면 어떻게 될까요?
5. **`temperature` 보내 보기** — `MessagesRequest` 에 `temperature: f32` 를 추가해 보내면 → `400 ... may not be set on this model/API version`.
6. **`read_note` 검사 끄기** — `safe_note_path` 가 항상 `Ok` 를 돌려주게 해 보면 `../secret.txt` 에서 무슨 일이 벌어질까요? (안전장치가 왜 "도구 쪽"에 있어야 하는지 체감해 보세요.)

## ▶️ 실제 Claude API 로 실행하기

> 🧪 **먼저**: 이 챕터의 Rust/C++ 코드는 **목 서버로만 검증**되어 있어요. 실제 API 에서는 모델의 말투·도구 호출 횟수가 매번 달라질 수 있고, 출력이 Python/C++ 와 똑같지 않은 것이 정상입니다.

```bash
export ANTHROPIC_API_KEY="sk-ant-..."          # 셸 세션에서만! 파일에 적지 마세요
export LLM_BASE_URL="https://api.anthropic.com"
cargo run -q -p ch17_capstone_agent --example solution
# 또는 python3 python/main.py
```

실제 호출에서 달라지는 점:

- 헤더 `x-api-key`, `anthropic-version: 2026-10-21`, 모델 `claude-opus-5-5`, `max_tokens: 16000`(비스트리밍), `output_config.effort = "medium"`.
- `temperature`/`top_p`/`top_k` 는 보내지 않습니다. thinking 은 항상 켜져 있어요 (`thinking: disabled` 를 보내지 마세요).
- `fallbacks: "default"` + `anthropic-beta: server-side-fallback-2026-07-01` 로 refusal 시 서버 측 대체 모델 재시도.
- 이 챕터의 코드는 **비스트리밍**이라, 긴 응답에서는 한참 기다릴 수 있어요 (타임아웃 600초). 실서비스에서는 스트리밍(SSE)을 씁니다 → 아래 "다음 단계".

> 💰 **비용 · 안전 수칙**
> - 에이전트 루프는 **반복할 때마다 전체 대화를 다시 보내므로** 토큰 사용량이 턴 수에 따라 빠르게 늘어요. `MAX_ITERATIONS` 상한과 `max_tokens` 를 꼭 유지하세요.
> - API 키는 **환경 변수로만** 다루고, 소스·`.env`·로그·커밋·스크린샷에 남기지 마세요. 실수로 커밋했다면 **즉시 키를 폐기(revoke)하고 재발급**하세요.
> - 이 프로젝트는 `.gitignore` 에 `.env` 를 넣어 두는 것을 권장해요. 에러 메시지·transcript 를 공유하기 전에 키가 섞여 있지 않은지 확인하세요.
> - 도구가 파일·네트워크·셸에 접근하게 만들수록 위험이 커져요. 읽기 전용 + 허용 목록부터 시작하세요.

## 🤖 바이브 코딩 포인트

- **에이전트 코드 리뷰 체크리스트**: ① 반복 상한이 있나? ② `stop_reason` 을 전부(`end_turn`/`tool_use`/`max_tokens`/`refusal`/`pause_turn`) 처리하나? ③ assistant `content` 를 그대로 되돌리나? ④ 병렬 `tool_use` 의 `tool_result` 를 한 메시지에 담나? ⑤ 도구 실패를 `is_error` 로 돌려주나, 아니면 `unwrap` 으로 죽나?
- **"도구 입력은 믿지 마"**: AI 가 만든 도구가 `std::fs::read_to_string(format!("data/{name}"))` 처럼 입력을 그대로 경로에 붙이면 경로 탈출 취약점이에요. *"이 도구의 입력 검증과 경로 제한을 점검해 줘"* 라고 요청하세요.
- **`async_trait` 이 보여도 놀라지 마세요**: 이 챕터의 `BoxFuture` 와 같은 일을 하는 매크로예요. 반대로 `trait Tool { async fn run(..) }` + `Box<dyn Tool>` 조합이 보이면 컴파일이 안 되는 코드(E0038)입니다.
- **프롬프트 팁 — 새 도구 추가**: *"`Tool` trait 를 구현하는 `HttpGet` 도구를 추가해 줘. 허용 도메인 목록만 접근하고, 타임아웃 5초, 응답은 처음 2000자만 돌려줘. 실패는 Err(String) 으로. 단위 테스트도 같이."* — 구체적인 제약(allowlist, 타임아웃, 크기 제한)을 말할수록 안전한 코드가 나와요.
- **테스트 먼저**: *"가짜 `Llm` 구현으로 이 에이전트 루프의 병렬 도구 호출, is_error, max-iterations 를 테스트해 줘."* — 네트워크 없는 테스트는 AI 가 만든 코드를 믿을 수 있게 해 주는 가장 빠른 방법입니다.
- **컴파일러를 대화 상대로**: 에러가 나면 전체 메시지를 그대로 AI 에게 붙여 넣고 "원인을 `Ch04`의 빌림 규칙으로 설명하고, 수정은 최소한으로" 라고 요청해 보세요.

## 🚀 다음 단계 (이 챕터를 넘어서)

1. **스트리밍(SSE)**: `"stream": true` 로 글자가 생성되는 대로 받기. `reqwest` 의 `bytes_stream()` + 줄 단위 `data:` 파싱, `content_block_delta` 이벤트 조립. (긴 `max_tokens` 에서는 사실상 필수예요.)
2. **병렬 도구 실행**: 지금은 한 턴의 도구를 순서대로 실행합니다. `tokio::task::JoinSet` / `futures::future::join_all` 로 동시에 실행해 보세요 (결과 순서는 `tool_use` 순서를 유지!).
3. **더 많은 도구**: 웹 검색, 파일 쓰기, 셸 — 단, 권한 확인(사람의 승인), 허용 목록, 타임아웃과 함께.
4. **MCP (Model Context Protocol)**: 도구를 별도 프로세스/서버로 분리해 표준 방식으로 연결. Rust 용 MCP SDK(`rmcp` 등) 도 있습니다.
5. **컨텍스트 관리**: 대화가 길어지면 오래된 도구 결과를 요약/정리 (서버 측 컴팩션 같은 기능 활용).
6. **관측성**: `tracing` 크레이트로 턴/도구 호출/토큰 사용량 로깅.
7. **AI 와 함께 확장하기**: 이 코드를 AI 에게 보여 주고 *"스트리밍을 추가해 줘, 단 기존 테스트는 그대로 통과해야 해"* 처럼 **작은 단계 + 기존 테스트 유지** 로 요청하세요. 돌아가는 테스트가 있는 코드는 AI 가 안전하게 바꿀 수 있는 코드입니다.

## ✅ 셀프 체크

1. 모델이 한 턴에 `tool_use` 를 두 개 요청했습니다. `tool_result` 는 어떻게 돌려줘야 하나요?
<details><summary>정답</summary>

두 결과를 **user 메시지 하나**의 `content` 배열에 담고, 각각 대응하는 `tool_use_id` 를 적습니다. 도구마다 메시지를 따로 보내거나 하나라도 빠뜨리면 400 이에요.
</details>

2. 응답의 `content` 를 타입 있는 `Vec<Block>` 으로 바꿨다가 다시 JSON 으로 만들어 `messages` 에 넣으면 안 되는 이유는?
<details><summary>정답</summary>

`thinking` 블록의 `signature` 같이 우리 타입에 없는 필드와, 앞으로 생길 새 블록이 사라질 수 있어서입니다. API 는 assistant 턴이 원본 그대로 돌아오기를 기대하고, 아니면 400 으로 거절해요. 원본은 `Vec<Value>` 로 보관·전송하고, 읽을 때만 `Block` 뷰를 씁니다.
</details>

3. `trait Tool { async fn run(&self) -> String; }` 로는 `Vec<Box<dyn Tool>>` 를 만들 수 없는 이유와 해결책은?
<details><summary>정답</summary>

`async fn` 은 호출마다 크기가 다른 Future 타입을 돌려줘서 vtable 로 가리킬 수 없어요(dyn 비호환, E0038). 반환 타입을 `Pin<Box<dyn Future<Output = T> + Send + 'a>>` (`BoxFuture`) 로 박싱하거나, `#[async_trait]` 매크로, 또는 enum 디스패치를 씁니다.
</details>

4. 도구가 에러를 냈을 때 `?` 로 `run_agent` 밖으로 전파하지 않고 `is_error: true` 결과로 돌려주는 이유는?
<details><summary>정답</summary>

도구 실패는 프로그램의 실패가 아니라 **모델이 알아야 할 정보**이기 때문이에요. 모델이 사유를 보고 다른 입력으로 재시도하거나 사용자에게 설명할 수 있습니다. (다만 API 호출 자체의 실패, refusal, 반복 초과 같은 것은 `AgentError` 로 전파합니다.)
</details>

5. `stop_reason == "refusal"` 을 `content` 를 읽기 **전에** 확인해야 하는 이유는? 그리고 `MAX_ITERATIONS` 가 필요한 이유는?
<details><summary>정답</summary>

refusal 응답은 `content` 가 비어 있거나 중간에 끊겨 있을 수 있어서, 먼저 확인하지 않고 텍스트를 꺼내면 빈 답변을 정상 답변으로 착각하거나 `unwrap` 이 터져요. `MAX_ITERATIONS` 는 모델이 같은 도구를 끝없이 호출하는 경우 비용이 무한히 늘어나는 것을 막는 안전장치입니다.
</details>

## 📚 더 알아보기

- The Rust Book: [Ch10 Traits](https://doc.rust-lang.org/book/ch10-02-traits.html), [Ch17 Async](https://doc.rust-lang.org/book/ch17-00-async-await.html), [Ch18 Trait objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- Rust by Example: [Traits](https://doc.rust-lang.org/rust-by-example/trait.html), [Error handling](https://doc.rust-lang.org/rust-by-example/error.html)
- Anthropic 문서: [Tool use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use), [Messages API](https://docs.anthropic.com/en/api/messages)
- 크레이트: [serde_json](https://docs.rs/serde_json), [reqwest](https://docs.rs/reqwest), [async-trait](https://docs.rs/async-trait), [tokio](https://docs.rs/tokio)
- 축하합니다! 🎉 17개 챕터를 마쳤어요. 이제 AI 가 만든 Rust 코드를 읽고, 컴파일러 에러를 이해하고, 구조를 지시할 수 있습니다. 직접 만든 에이전트에 도구를 하나 더 붙여 보세요.
