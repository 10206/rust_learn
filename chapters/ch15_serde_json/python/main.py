"""Ch15. Serde 와 JSON - Python 참조 구현.

Rust 버전(rust/examples/solution.rs)과 함수 이름이 1:1 로 대응됩니다.
Python 의 json 모듈은 "dict 로 읽기"만 해 주고, 타입 검사는 직접 해야 합니다.
Rust 의 Serde 는 이 검사를 derive 한 줄로 자동 생성합니다.
"""

import json
from dataclasses import dataclass, field
from typing import Any

# ── 타입 정의 ─────────────────────────────────────────────
# Rust: enum KnownBlock { Text{..}, Thinking{..}, ToolUse{..}, ToolResult{..} }  (#[serde(tag = "type")])
# Rust: enum ContentBlock { Known(KnownBlock), Unknown(Value) }                   (#[serde(untagged)])
# Python 에서는 "모르는 블록"을 그냥 dict 로 들고 있으면 됩니다.


@dataclass
class TextBlock:
    text: str


@dataclass
class ThinkingBlock:
    thinking: str
    signature: str


@dataclass
class ToolUseBlock:
    id: str
    name: str
    input: Any


@dataclass
class ToolResultBlock:
    tool_use_id: str
    content: str
    is_error: bool | None = None


# Unknown 블록 = 원본 dict 그대로 (한 글자도 잃지 않고 되돌려 보내기 위해)
ContentBlock = TextBlock | ThinkingBlock | ToolUseBlock | ToolResultBlock | dict


@dataclass
class Message:
    role: str  # "user" | "assistant"
    content: str | list[ContentBlock]  # Rust: #[serde(untagged)] enum MessageContent


@dataclass
class Tool:
    name: str
    description: str
    input_schema: Any


@dataclass
class Request:
    model: str
    max_tokens: int
    system: str | None
    tools: list[Tool]
    messages: list[Message]
    stop_sequences: list[str] = field(default_factory=list)


@dataclass
class Summary:
    message_count: int
    tool_calls: int
    tool_errors: int
    tools_used: dict[str, int]
    unknown_blocks: list[str]


# ── JSON -> 타입 (역직렬화) ────────────────────────────────


def _need(obj: dict, key: str, kind: type) -> Any:
    """키가 있고 타입이 맞는지 검사합니다. (Serde 가 자동으로 해 주는 일)"""
    if not isinstance(obj, dict) or key not in obj:
        raise ValueError(f"missing field `{key}`")
    value = obj[key]
    if not isinstance(value, kind) or (kind is int and isinstance(value, bool)):
        raise ValueError(f"invalid type for `{key}`")
    return value


def block_from_json(raw: Any) -> ContentBlock:
    """TODO(3): 아는 블록이면 타입으로, 아니면(또는 모양이 안 맞으면) 원본 그대로 보관."""
    try:
        kind = raw["type"]
        if kind == "text":
            return TextBlock(_need(raw, "text", str))
        if kind == "thinking":
            return ThinkingBlock(_need(raw, "thinking", str), _need(raw, "signature", str))
        if kind == "tool_use":
            return ToolUseBlock(_need(raw, "id", str), _need(raw, "name", str), _need(raw, "input", object))
        if kind == "tool_result":
            is_error = raw.get("is_error")
            if is_error is not None and not isinstance(is_error, bool):
                raise ValueError("invalid type for `is_error`")
            return ToolResultBlock(_need(raw, "tool_use_id", str), _need(raw, "content", str), is_error)
    except (KeyError, TypeError, ValueError):
        pass
    return raw  # Unknown: Rust 의 ContentBlock::Unknown(Value)


def message_from_json(raw: Any) -> Message:
    role = _need(raw, "role", str)
    if role not in ("user", "assistant"):
        raise ValueError(f"unknown variant `{role}`")
    if "content" not in raw:
        raise ValueError("missing field `content`")
    content = raw["content"]
    if isinstance(content, str):
        return Message(role, content)
    if isinstance(content, list):
        return Message(role, [block_from_json(b) for b in content])
    raise ValueError("data did not match any variant of untagged enum MessageContent")


def request_from_json(raw: Any) -> Request:
    return Request(
        model=_need(raw, "model", str),
        max_tokens=_need(raw, "max_tokens", int),
        system=raw.get("system"),
        tools=[Tool(_need(t, "name", str), _need(t, "description", str), _need(t, "input_schema", object)) for t in _need(raw, "tools", list)],
        messages=[message_from_json(m) for m in _need(raw, "messages", list)],
        stop_sequences=raw.get("stop_sequences", []),
    )


# TODO(5)
def parse_request(text: str) -> Request:
    return request_from_json(json.loads(text))


def parse_message(text: str) -> Message:
    return message_from_json(json.loads(text))


# ── 타입 -> JSON (직렬화) ──────────────────────────────────


def block_to_json(block: ContentBlock) -> Any:
    # dict 는 키 순서가 "넣은 순서" 이므로, Rust 의 필드 선언 순서와 똑같이 넣어 줍니다.
    match block:
        case TextBlock(text):
            return {"type": "text", "text": text}
        case ThinkingBlock(thinking, signature):
            return {"type": "thinking", "thinking": thinking, "signature": signature}
        case ToolUseBlock(id, name, input):
            return {"type": "tool_use", "id": id, "name": name, "input": input}
        case ToolResultBlock(tool_use_id, content, is_error):
            out = {"type": "tool_result", "tool_use_id": tool_use_id, "content": content}
            if is_error is not None:  # Rust: skip_serializing_if = "Option::is_none"
                out["is_error"] = is_error
            return out
        case _:
            return block  # Unknown: 그대로


def message_to_json(msg: Message) -> Any:
    content = msg.content if isinstance(msg.content, str) else [block_to_json(b) for b in msg.content]
    return {"role": msg.role, "content": content}


def request_to_json(req: Request) -> Any:
    out: dict[str, Any] = {"model": req.model, "max_tokens": req.max_tokens}
    if req.system is not None:
        out["system"] = req.system
    out["tools"] = [{"name": t.name, "description": t.description, "input_schema": t.input_schema} for t in req.tools]
    out["messages"] = [message_to_json(m) for m in req.messages]
    if req.stop_sequences:  # Rust: skip_serializing_if = "Vec::is_empty"
        out["stop_sequences"] = req.stop_sequences
    return out


def summary_to_json(s: Summary) -> Any:
    # Rust: #[serde(rename_all = "camelCase")]
    return {
        "messageCount": s.message_count,
        "toolCalls": s.tool_calls,
        "toolErrors": s.tool_errors,
        "toolsUsed": s.tools_used,
        "unknownBlocks": s.unknown_blocks,
    }


def to_pretty(obj: Any) -> str:
    # indent=2, ensure_ascii=False: Rust serde_json::to_string_pretty 와 같은 모양
    return json.dumps(obj, indent=2, ensure_ascii=False)


def to_compact(obj: Any) -> str:
    return json.dumps(obj, separators=(",", ":"), ensure_ascii=False)


# ── 분석 ───────────────────────────────────────────────────

KNOWN = (TextBlock, ThinkingBlock, ToolUseBlock, ToolResultBlock)


def block_kind(block: ContentBlock) -> str:
    """TODO(6): 블록의 type 이름. 모르는 블록은 원본의 "type" 값."""
    match block:
        case TextBlock():
            return "text"
        case ThinkingBlock():
            return "thinking"
        case ToolUseBlock():
            return "tool_use"
        case ToolResultBlock():
            return "tool_result"
        case _:
            return block.get("type", "?") if isinstance(block, dict) else "?"


def message_kinds(msg: Message) -> list[str]:
    """문자열 content 는 text 블록 하나로 칩니다."""
    if isinstance(msg.content, str):
        return ["text"]
    return [block_kind(b) for b in msg.content]


def count_blocks(req: Request) -> dict[str, int]:
    counts: dict[str, int] = {}
    for msg in req.messages:
        for kind in message_kinds(msg):
            counts[kind] = counts.get(kind, 0) + 1
    return dict(sorted(counts.items()))


def tools_used(req: Request) -> dict[str, int]:
    counts: dict[str, int] = {}
    for msg in req.messages:
        if isinstance(msg.content, list):
            for b in msg.content:
                if isinstance(b, ToolUseBlock):
                    counts[b.name] = counts.get(b.name, 0) + 1
    return dict(sorted(counts.items()))


def tool_call_report(req: Request) -> list[str]:
    """tool_use 와 tool_result 를 id 로 짝지어 한 줄씩 보고합니다."""
    calls: dict[str, ToolUseBlock] = {}
    lines = []
    for msg in req.messages:
        if isinstance(msg.content, str):
            continue
        for b in msg.content:
            if isinstance(b, ToolUseBlock):
                calls[b.id] = b
            elif isinstance(b, ToolResultBlock):
                call = calls.get(b.tool_use_id)
                head = f"{call.name} {to_compact(call.input)}" if call else f"? {b.tool_use_id}"
                tail = f"ERROR: {b.content}" if b.is_error else b.content
                lines.append(f"{head} -> {tail}")
    return lines


def build_followup(req: Request) -> Request:
    """처음 3개 메시지 + 요약 요청 text 블록을 담은 새 요청을 만듭니다."""
    messages = [Message(m.role, list(m.content) if isinstance(m.content, list) else m.content) for m in req.messages[:3]]
    last = messages[-1]
    assert isinstance(last.content, list)
    last.content.append(TextBlock("Please summarize the results above in one sentence."))
    return Request(
        model=req.model,
        max_tokens=256,
        system="Answer in one short sentence.",
        tools=list(req.tools),
        messages=messages,
        stop_sequences=[],
    )


def summarize(req: Request) -> Summary:
    used = tools_used(req)
    errors = sum(
        1
        for m in req.messages
        if isinstance(m.content, list)
        for b in m.content
        if isinstance(b, ToolResultBlock) and b.is_error
    )
    unknown = sorted({block_kind(b) for m in req.messages if isinstance(m.content, list) for b in m.content if not isinstance(b, KNOWN)})
    return Summary(len(req.messages), sum(used.values()), errors, used, unknown)


def classify_error(text: str) -> str:
    """TODO(11): 메시지 JSON 하나를 파싱해 "ok" / "syntax" / "data" 로 분류."""
    try:
        parse_message(text)
        return "ok"
    except json.JSONDecodeError:  # ValueError 의 하위 클래스이므로 먼저 검사
        return "syntax"
    except ValueError:
        return "data"


def round_trip_lossless(text: str) -> bool:
    """TODO(12): 타입으로 읽었다가 다시 JSON 으로 써도 원본과 똑같은가?"""
    return request_to_json(parse_request(text)) == json.loads(text)


def main() -> None:
    with open("data/conversation.json", encoding="utf-8") as f:
        text = f.read()
    req = parse_request(text)

    print("== request ==")
    print(f"model: {req.model}")
    print(f"max_tokens: {req.max_tokens}")
    print(f"tools: {', '.join(t.name for t in req.tools)}")
    print(f"messages: {len(req.messages)}")

    print("== messages ==")
    for i, msg in enumerate(req.messages):
        print(f"[{i}] {msg.role}: {', '.join(message_kinds(msg))}")

    print("== block counts ==")
    for kind, n in count_blocks(req).items():
        print(f"{kind}: {n}")

    print("== tools used ==")
    for name, n in tools_used(req).items():
        print(f"{name}: {n}")

    print("== tool calls ==")
    for line in tool_call_report(req):
        print(line)

    print("== bad input ==")
    bad = [
        '{"role": "user", "content": ',
        '{"role": "robot", "content": "hi"}',
        '{"role": "user"}',
        '{"role": "user", "content": [{"type": "text"}]}',
        '{"role": "user", "content": 42}',
    ]
    for i, b in enumerate(bad, 1):
        print(f"{i}: {classify_error(b)}")

    print("== round trip ==")
    print(f"lossless: {'yes' if round_trip_lossless(text) else 'no'}")

    print("== follow-up request ==")
    print(to_pretty(request_to_json(build_followup(req))))

    print("== summary (original) ==")
    print(to_pretty(summary_to_json(summarize(req))))

    print("== summary (follow-up) ==")
    print(to_pretty(summary_to_json(summarize(build_followup(req)))))


if __name__ == "__main__":
    main()
