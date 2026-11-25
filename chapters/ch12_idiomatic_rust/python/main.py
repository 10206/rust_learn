"""Ch12. Rust다운 코드(관용구) - Python 참조 구현.

주제: "설정(AgentConfig) 빌더 + 검증 + 파싱 + 리포트".
Rust 버전에서는 이 구조를 builder / TryFrom / Display / Default / 이터레이터로 표현합니다.
"""

from __future__ import annotations

from dataclasses import dataclass, field


class ConfigError(Exception):
    """Rust: enum ConfigError (+ Display + std::error::Error)."""


@dataclass
class AgentConfig:
    name: str = ""
    model: str = "default-model"
    max_tokens: int = 1024
    temperature: float = 1.0
    tools: list[str] = field(default_factory=list)
    system_prompt: str | None = None
    retries: int = 2
    timeout_secs: int = 30

    # Rust: impl fmt::Display for AgentConfig
    def __str__(self) -> str:
        prompt = "none" if self.system_prompt is None else f"{len(self.system_prompt)} chars"
        return (
            f"{self.name} | model={self.model} | tokens={self.max_tokens} | "
            f"temp={self.temperature:.2f} | retries={self.retries} | "
            f"timeout={self.timeout_secs}s | tools=[{join_tools(self.tools)}] | prompt={prompt}"
        )


def validate_temperature(value: float) -> float:
    # Rust: struct Temperature(f64) + impl TryFrom<f64>
    if not (0.0 <= value <= 2.0):
        raise ConfigError(f"temperature {value:.1f} out of range 0.0..=2.0")
    return value


class AgentConfigBuilder:
    """Rust 의 builder: 메서드 체인 (self 를 값으로 받아 self 를 돌려줌)."""

    def __init__(self, name: str) -> None:
        self._cfg = AgentConfig(name=name)

    def name(self, v: str) -> AgentConfigBuilder:
        self._cfg.name = v
        return self

    def model(self, v: str) -> AgentConfigBuilder:
        self._cfg.model = v
        return self

    def max_tokens(self, v: int) -> AgentConfigBuilder:
        self._cfg.max_tokens = v
        return self

    def temperature(self, v: float) -> AgentConfigBuilder:
        self._cfg.temperature = v
        return self

    def tool(self, v: str) -> AgentConfigBuilder:
        self._cfg.tools.append(v)
        return self

    def tools(self, vs: list[str]) -> AgentConfigBuilder:
        self._cfg.tools.extend(vs)
        return self

    def system_prompt(self, v: str) -> AgentConfigBuilder:
        self._cfg.system_prompt = v
        return self

    def retries(self, v: int) -> AgentConfigBuilder:
        self._cfg.retries = v
        return self

    def timeout_secs(self, v: int) -> AgentConfigBuilder:
        self._cfg.timeout_secs = v
        return self

    def build(self) -> AgentConfig:
        c = self._cfg
        if not c.name.strip():
            raise ConfigError("missing required field: name")
        if not c.model.strip():
            raise ConfigError("missing required field: model")
        if not (1 <= c.max_tokens <= 8192):
            raise ConfigError(f"max_tokens {c.max_tokens} out of range 1..=8192")
        validate_temperature(c.temperature)
        if c.retries > 5:
            raise ConfigError(f"retries {c.retries} exceeds limit 5")
        seen: set[str] = set()
        for t in c.tools:
            if t in seen:
                raise ConfigError(f"duplicate tool: {t}")
            seen.add(t)
        return c


def _parse_num(key: str, value: str, kind: type) -> int | float:
    try:
        return kind(value)
    except ValueError:
        raise ConfigError(f"invalid value for {key}: '{value}'") from None


def parse_config(spec: str) -> AgentConfig:
    """Rust: impl TryFrom<&str> for AgentConfig."""
    b = AgentConfigBuilder("")
    for pair in (p.strip() for p in spec.split(";")):
        if not pair:
            continue
        if "=" not in pair:
            raise ConfigError(f"malformed pair: '{pair}'")
        key, value = (s.strip() for s in pair.split("=", 1))
        match key:
            case "name":
                b.name(value)
            case "model":
                b.model(value)
            case "max_tokens":
                b.max_tokens(_parse_num(key, value, int))
            case "temperature":
                b.temperature(_parse_num(key, value, float))
            case "retries":
                b.retries(_parse_num(key, value, int))
            case "timeout":
                b.timeout_secs(_parse_num(key, value, int))
            case "tools":
                b.tools([t.strip() for t in value.split(",") if t.strip()])
            case "prompt":
                b.system_prompt(value)
            case _:
                raise ConfigError(f"unknown key: {key}")
    return b.build()


# ───────────── 작은 유틸리티 (Rust: 슬라이스/이터레이터/Option 콤비네이터) ─────────────
def join_tools(tools: list[str]) -> str:
    return ", ".join(tools)


def count_long_names(names: list[str], min_len: int) -> int:
    return sum(1 for n in names if len(n) >= min_len)


def find_tool(tools: list[str], name: str) -> int | None:
    return tools.index(name) if name in tools else None


def prompt_len(c: AgentConfig) -> int:
    return len(c.system_prompt) if c.system_prompt is not None else 0


def first_tool_upper(c: AgentConfig) -> str | None:
    return c.tools[0].upper() if c.tools else None


def describe_temperature(t: float) -> str:
    if t < 0.3:
        return "precise"
    if t < 0.8:
        return "balanced"
    return "creative"


def report(configs: list[AgentConfig]) -> str:
    lines = [f"{'name':<8}|{'tokens':>7}|{'temp':>6}"]
    for c in sorted(configs, key=lambda c: c.name):
        lines.append(f"{c.name:<8}|{c.max_tokens:>7}|{c.temperature:>6.2f}")
    lines.append(f"total tokens: {sum(c.max_tokens for c in configs)}")
    largest = max(configs, key=lambda c: c.max_tokens)
    lines.append(f"largest: {largest.name} ({largest.max_tokens})")
    return "\n".join(lines)


def opt(v: object) -> str:
    return "none" if v is None else str(v)


def main() -> None:
    print("== 1. builder ==")
    helper = (
        AgentConfigBuilder("helper")
        .model("claude-sonnet")
        .max_tokens(1024)
        .temperature(0.7)
        .tool("search")
        .tool("calc")
        .system_prompt("You are a helpful assistant.")
        .build()
    )
    writer = (
        AgentConfigBuilder("writer")
        .model("claude-opus")
        .max_tokens(4096)
        .temperature(0.3)
        .retries(1)
        .timeout_secs(60)
        .build()
    )
    coder = AgentConfigBuilder("coder").tool("shell").system_prompt("Write code.").build()
    for c in (helper, writer, coder):
        print(c)

    print("== 2. validation ==")
    cases = [
        ("empty name", lambda: AgentConfigBuilder("").build()),
        ("hot temperature", lambda: AgentConfigBuilder("a").temperature(3.5).build()),
        ("zero tokens", lambda: AgentConfigBuilder("a").max_tokens(0).build()),
        ("too many retries", lambda: AgentConfigBuilder("a").retries(9).build()),
        ("duplicate tool", lambda: AgentConfigBuilder("a").tool("search").tool("calc").tool("search").build()),
    ]
    for label, make in cases:
        try:
            make()
            print(f"  {label}: ok")
        except ConfigError as e:
            print(f"  {label}: error: {e}")

    print("== 3. parse ==")
    specs = [
        "name=writer;model=claude-opus;max_tokens=2048;temperature=0.3;tools=search,calc;prompt=Be brief",
        "name=scribe",
        "name=x; max_tokens=abc",
        "model=m1",
        "name=x;color=red",
        "name=x;oops",
        "name=x;temperature=9",
    ]
    for i, spec in enumerate(specs, start=1):
        try:
            print(f"  [{i}] ok: {parse_config(spec)}")
        except ConfigError as e:
            print(f"  [{i}] error: {e}")

    print("== 4. utilities ==")
    tools = ["search", "calc", "shell", "web_browser_long"]
    print(f"join_tools: {join_tools(tools)}")
    print(f"count_long_names(>=5): {count_long_names(tools, 5)}")
    print(f"find_tool shell: {opt(find_tool(tools, 'shell'))}")
    print(f"find_tool python: {opt(find_tool(tools, 'python'))}")
    print(f"prompt_len helper: {prompt_len(helper)}")
    print(f"prompt_len writer: {prompt_len(writer)}")
    print(f"first_tool_upper helper: {opt(first_tool_upper(helper))}")
    print(f"first_tool_upper writer: {opt(first_tool_upper(writer))}")
    for t in (0.0, 0.5, 1.5):
        print(f"temperature {t:.1f}: {describe_temperature(t)}")

    print("== 5. report ==")
    print(report([helper, writer, coder]))


if __name__ == "__main__":
    main()
