"""Ch08 Python 참조 구현: 에이전트 도구 레지스트리 (동기).

Python 은 "덕 타이핑"이라 별도 선언 없이 name/description/run 을 가진 객체면 도구로 쓸 수 있어요.
여기서는 typing.Protocol 로 "이런 모양이어야 해요"를 문서화만 합니다 (실행 시점에는 검사하지 않음).
Rust 의 trait 은 이 약속을 컴파일러가 강제해 준다는 점이 다릅니다.
"""
from __future__ import annotations

from typing import Protocol, TypeVar


class Tool(Protocol):
    def name(self) -> str: ...
    def description(self) -> str: ...
    def run(self, input: str) -> str: ...  # 실패하면 ToolError 를 던집니다

    # Protocol 은 기본 메서드를 둘 수 있지만, 여기서는 함수로 대신합니다.


class ToolError(Exception):
    """Rust 의 Err(String) 에 해당합니다."""


def describe(tool: Tool) -> str:  # Rust: Tool::describe (기본 메서드)
    return f"{tool.name()}: {tool.description()}"


class Calculator:
    def name(self) -> str:
        return "calculator"

    def description(self) -> str:
        return "evaluates 'a op b' on integers (+ - * /)"

    def run(self, input: str) -> str:
        parts = input.split()
        if len(parts) != 3:
            raise ToolError(f"invalid expression: {input}")
        a_text, op, b_text = parts
        try:
            a, b = int(a_text), int(b_text)
        except ValueError:
            raise ToolError(f"invalid expression: {input}") from None
        if op == "+":
            return str(a + b)
        if op == "-":
            return str(a - b)
        if op == "*":
            return str(a * b)
        if op == "/":
            if b == 0:
                raise ToolError("division by zero")
            # C++/Rust 의 정수 나눗셈은 0 방향으로 버리지만 Python 의 // 는 내림이라 음수에서 달라집니다.
            q = abs(a) // abs(b)
            return str(q if (a < 0) == (b < 0) else -q)
        raise ToolError(f"invalid expression: {input}")


class WordCounter:
    def name(self) -> str:
        return "word_counter"

    def description(self) -> str:
        return "counts whitespace-separated words"

    def run(self, input: str) -> str:
        n = len(input.split())
        if n == 0:
            raise ToolError("empty input")
        return str(n)


class Reverser:
    def name(self) -> str:
        return "reverser"

    def description(self) -> str:
        return "reverses the characters of the input"

    def run(self, input: str) -> str:
        return input[::-1]


class ToolCall:
    def __init__(self, tool: str, input: str) -> None:
        self.tool = tool
        self.input = input

    def __str__(self) -> str:  # Rust: impl Display
        return f'{self.tool}("{self.input}")'


class ToolRegistry:
    def __init__(self) -> None:
        self.tools: list[Tool] = []  # Rust: Vec<Box<dyn Tool>>

    def register(self, tool: Tool) -> None:
        self.tools.append(tool)

    def names(self) -> list[str]:
        return [t.name() for t in self.tools]

    def call(self, name: str, input: str) -> str:
        for tool in self.tools:
            if tool.name() == name:
                return tool.run(input)
        raise ToolError(f"unknown tool: {name}")


def try_run(call_fn) -> str:
    """Rust 의 Result<String,String> -> "ok: ..." / "error: ..." 표시."""
    try:
        return f"ok: {call_fn()}"
    except ToolError as e:
        return f"error: {e}"


def run_any(tool: Tool, input: str) -> str:
    # Python 은 정적/동적 구분이 없고 항상 동적입니다 (속성 조회 시점에 결정).
    return f'{tool.name()}("{input}") -> {try_run(lambda: tool.run(input))}'


T = TypeVar("T")


def largest(items: list[T]) -> T | None:
    # 타입 변수 T 는 힌트일 뿐 - 비교가 안 되는 타입을 넣으면 "실행 중에" TypeError 가 납니다.
    if not items:
        return None
    best = items[0]
    for x in items[1:]:
        if x > best:  # type: ignore[operator]
            best = x
    return best


def main() -> None:
    registry = ToolRegistry()
    registry.register(Calculator())
    registry.register(WordCounter())
    registry.register(Reverser())

    print("== registry ==")
    print(f"tools: {', '.join(registry.names())}")
    for tool in registry.tools:
        print(describe(tool))

    print("== calls ==")
    calls = [ToolCall(t, i) for t, i in [
        ("calculator", "12 * 3"),
        ("word_counter", "the quick brown fox"),
        ("reverser", "hello"),
        ("calculator", "7 / 0"),
        ("calculator", "abc"),
        ("word_counter", ""),
        ("translator", "hi"),
    ]]
    for call in calls:
        print(f"{call} -> {try_run(lambda: registry.call(call.tool, call.input))}")

    print("== dispatch ==")
    print(f"static : {run_any(Reverser(), 'abc')}")
    print(f"dynamic: {run_any(Reverser(), 'abc')}")

    print("== generics ==")
    print(f"largest ints: {largest([3, 42, 7])}")
    print(f"largest floats: {largest([1.5, 3.5, 2.25]):.2f}")
    print(f"largest chars: {largest(['a', 'z', 'm'])}")
    empty = largest([])
    print(f"largest empty: {'none' if empty is None else empty}")


if __name__ == "__main__":
    main()
