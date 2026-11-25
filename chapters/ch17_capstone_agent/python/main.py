"""Ch17. 캡스톤: 미니 AI 에이전트 (Python 참조 구현)

공식 `anthropic` SDK 를 쓰되, 도구 실행 루프는 직접(manual loop) 작성합니다.
Rust / C++ 버전과 같은 구조라서 1:1 로 대응시켜 볼 수 있어요.

환경 변수
  LLM_BASE_URL       기본값 http://127.0.0.1:8787 (목 서버).  실제 API: https://api.anthropic.com
  ANTHROPIC_API_KEY  목 서버에서는 생략 가능 ("mock-key"), 실제 API 에서는 필수
  LLM_MODEL          기본값 claude-opus-5-5
  AGENT_TASK         기본 과제 대신 다른 과제를 주고 싶을 때
"""

import json
import os
import sys
from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path

import anthropic

MAX_ITERATIONS = 10
MAX_TOKENS = 16000
PREVIEW_LEN = 70
DATA_DIR = Path("data")
DEFAULT_TASK = (
    "Read the note 'notes.txt', count its words, compute (12+30)*2, "
    "try to read '../secret.txt', and check the weather in Seoul. Then summarize."
)
SYSTEM_PROMPT = "You are a concise assistant. Use the provided tools when they help."


# ───────────────────────── 설정 ─────────────────────────

@dataclass
class Config:
    base_url: str
    api_key: str
    model: str

    @property
    def is_real(self) -> bool:
        return "api.anthropic.com" in self.base_url


def config_from_env() -> Config:
    base_url = os.environ.get("LLM_BASE_URL", "http://127.0.0.1:8787")
    cfg = Config(base_url, "", os.environ.get("LLM_MODEL", "claude-opus-5-5"))
    key = os.environ.get("ANTHROPIC_API_KEY")
    if key:
        cfg.api_key = key
    elif not cfg.is_real:
        cfg.api_key = "mock-key"  # 목 서버일 때만 가짜 키 허용
    else:
        raise SystemExit("ANTHROPIC_API_KEY is required for the real API")
    return cfg


# ───────────────────────── 도구 ─────────────────────────

class ToolError(Exception):
    """도구가 '실패했다'고 모델에게 알리고 싶을 때 (-> is_error: true)."""


class Tool(ABC):
    @property
    @abstractmethod
    def name(self) -> str: ...

    @property
    @abstractmethod
    def description(self) -> str: ...

    @property
    @abstractmethod
    def input_schema(self) -> dict: ...

    @abstractmethod
    def run(self, tool_input: dict) -> str: ...

    def definition(self) -> dict:
        return {"name": self.name, "description": self.description, "input_schema": self.input_schema}


def required_str(tool_input: dict, key: str) -> str:
    value = tool_input.get(key)
    if not isinstance(value, str):
        raise ToolError(f"missing string field `{key}`")
    return value


def eval_expr(text: str) -> float:
    """+ - * / 와 괄호, 소수를 지원하는 재귀 하강 계산기."""
    s = text.replace(" ", "")
    pos = 0

    def peek() -> str:
        return s[pos] if pos < len(s) else ""

    def expr() -> float:
        nonlocal pos
        v = term()
        while peek() in ("+", "-"):
            op = s[pos]
            pos += 1
            r = term()
            v = v + r if op == "+" else v - r
        return v

    def term() -> float:
        nonlocal pos
        v = factor()
        while peek() in ("*", "/"):
            op = s[pos]
            pos += 1
            r = factor()
            if op == "*":
                v *= r
            else:
                if r == 0:
                    raise ToolError("division by zero")
                v /= r
        return v

    def factor() -> float:
        nonlocal pos
        c = peek()
        if c == "-":
            pos += 1
            return -factor()
        if c == "(":
            pos += 1
            v = expr()
            if peek() != ")":
                raise ToolError("invalid expression: expected ')'")
            pos += 1
            return v
        start = pos
        while peek() != "" and (peek().isdigit() or peek() == "."):
            pos += 1
        if start == pos:
            raise ToolError(f"invalid expression: unexpected '{c}'" if c else "invalid expression: unexpected end")
        return float(s[start:pos])

    v = expr()
    if pos != len(s):
        raise ToolError(f"invalid expression: unexpected '{s[pos]}'")
    return v


def format_number(x: float) -> str:
    if x == int(x) and abs(x) < 1e15:
        return str(int(x))
    return f"{x:.2f}"


class Calculator(Tool):
    name = "calculator"
    description = "Evaluate an arithmetic expression with + - * / and parentheses."
    input_schema = {
        "type": "object",
        "properties": {"expression": {"type": "string", "description": "e.g. (12+30)*2"}},
        "required": ["expression"],
    }

    def run(self, tool_input: dict) -> str:
        return format_number(eval_expr(required_str(tool_input, "expression")))


class WordCount(Tool):
    name = "word_count"
    description = "Count the whitespace-separated words in a text."
    input_schema = {
        "type": "object",
        "properties": {"text": {"type": "string"}},
        "required": ["text"],
    }

    def run(self, tool_input: dict) -> str:
        return str(len(required_str(tool_input, "text").split()))


def safe_note_path(name: str) -> Path:
    """data/ 안의 '파일 이름 하나'만 허용. ../ 같은 경로 탈출은 거부합니다."""
    if name in ("", ".", "..") or "/" in name or "\\" in name:
        raise ToolError(f"path not allowed: {name}")
    return DATA_DIR / name


class ReadNote(Tool):
    name = "read_note"
    description = "Read a text note from the data/ directory by file name."
    input_schema = {
        "type": "object",
        "properties": {"name": {"type": "string", "description": "file name, e.g. notes.txt"}},
        "required": ["name"],
    }

    def run(self, tool_input: dict) -> str:
        name = required_str(tool_input, "name")
        path = safe_note_path(name)
        try:
            return path.read_text(encoding="utf-8")
        except FileNotFoundError:
            raise ToolError(f"note not found: {name}") from None


class GetWeather(Tool):
    name = "get_weather"
    description = "Get the (fixed, fake) current weather for a city."
    input_schema = {
        "type": "object",
        "properties": {"city": {"type": "string"}},
        "required": ["city"],
    }
    TABLE = {"Seoul": "18C, clear", "Busan": "21C, cloudy"}

    def run(self, tool_input: dict) -> str:
        city = required_str(tool_input, "city")
        if city not in self.TABLE:
            raise ToolError(f"no weather data for {city}")
        return self.TABLE[city]


class ToolRegistry:
    def __init__(self) -> None:
        self.tools: list[Tool] = []

    def register(self, tool: Tool) -> None:
        self.tools.append(tool)

    def get(self, name: str) -> Tool | None:
        return next((t for t in self.tools if t.name == name), None)

    def definitions(self) -> list[dict]:
        return [t.definition() for t in self.tools]

    def execute(self, name: str, tool_input: dict) -> tuple[str, bool]:
        """(내용, is_error). 도구 실패는 예외로 터뜨리지 않고 '결과'로 모델에게 돌려줍니다."""
        tool = self.get(name)
        if tool is None:
            return f"unknown tool: {name}", True
        try:
            return tool.run(tool_input), False
        except ToolError as e:
            return str(e), True


# ───────────────────────── 출력 도우미 ─────────────────────────

def preview(s: str) -> str:
    s = s.replace("\n", "\\n")
    return s if len(s) <= PREVIEW_LEN else s[: PREVIEW_LEN - 3] + "..."


def compact_json(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True)


# ───────────────────────── 에이전트 루프 ─────────────────────────

class AgentError(Exception):
    pass


@dataclass
class AgentOutcome:
    final_text: str
    turns: int
    tool_calls: int
    tool_errors: int


def run_agent(client: anthropic.Anthropic, cfg: Config, registry: ToolRegistry, task: str) -> AgentOutcome:
    messages: list[dict] = [{"role": "user", "content": task}]
    tools = registry.definitions()
    tool_calls = tool_errors = 0

    print(f"task: {task}")
    for turn in range(1, MAX_ITERATIONS + 1):
        # 실제 API 에서는 refusal fallback 을 켭니다 (README 참고). 목 서버는 그냥 받아 줍니다.
        response = client.beta.messages.create(
            model=cfg.model,
            max_tokens=MAX_TOKENS,
            system=SYSTEM_PROMPT,
            tools=tools,
            messages=messages,
            output_config={"effort": "medium"},
            betas=["server-side-fallback-2026-07-01"],
            fallbacks="default",
        )
        print(f"--- turn {turn} (stop_reason: {response.stop_reason}) ---")

        # refusal 은 content 를 읽기 전에 먼저 확인합니다.
        if response.stop_reason == "refusal":
            raise AgentError("the model refused the request")

        for block in response.content:
            if block.type == "thinking":
                print("thinking: [block kept]")
            elif block.type == "text" and response.stop_reason != "end_turn":
                print(f"assistant: {block.text}")  # 마지막 답변은 아래 'final answer' 에서 출력
            elif block.type == "tool_use":
                print(f"tool_use: {block.name} {preview(compact_json(block.input))}")

        if response.stop_reason == "end_turn":
            final = "".join(b.text for b in response.content if b.type == "text")
            return AgentOutcome(final, turn, tool_calls, tool_errors)
        if response.stop_reason == "max_tokens":
            raise AgentError("the answer was cut off (max_tokens)")

        # assistant 의 content 는 thinking 블록까지 '그대로' 되돌려 보냅니다 (고치거나 빼지 않기!).
        messages.append({"role": "assistant", "content": response.content})
        if response.stop_reason == "pause_turn":
            continue
        if response.stop_reason != "tool_use":
            raise AgentError(f"unexpected stop_reason: {response.stop_reason}")

        # 한 번의 assistant 턴에 나온 tool_result 는 user 메시지 '하나'에 모두 담습니다.
        results = []
        for block in response.content:
            if block.type != "tool_use":
                continue
            content, is_error = registry.execute(block.name, block.input)
            tool_calls += 1
            tool_errors += is_error
            print(f"result: {block.name} {'ERROR' if is_error else 'ok'} {preview(content)}")
            item = {"type": "tool_result", "tool_use_id": block.id, "content": content}
            if is_error:
                item["is_error"] = True
            results.append(item)
        messages.append({"role": "user", "content": results})

    raise AgentError(f"agent did not finish within {MAX_ITERATIONS} iterations")


def main() -> int:
    cfg = config_from_env()
    client = anthropic.Anthropic(base_url=cfg.base_url, api_key=cfg.api_key)
    registry = ToolRegistry()
    for tool in (Calculator(), WordCount(), ReadNote(), GetWeather()):
        registry.register(tool)

    task = os.environ.get("AGENT_TASK", DEFAULT_TASK)
    try:
        out = run_agent(client, cfg, registry, task)
    except AgentError as e:
        print(f"agent error: {e}", file=sys.stderr)
        return 1
    except anthropic.APIStatusError as e:
        print(f"API error {e.status_code}: {e.message}", file=sys.stderr)
        return 1

    print("--- final answer ---")
    print(out.final_text)
    print(f"--- done: {out.turns} turns, {out.tool_calls} tool calls, {out.tool_errors} tool errors ---")
    return 0


if __name__ == "__main__":
    sys.exit(main())
