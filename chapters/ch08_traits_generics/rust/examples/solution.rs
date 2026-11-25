//! Ch08 정답: 에이전트 도구 레지스트리 (동기 버전)
//!
//! 실행: cargo run -q -p ch08_traits_generics --example solution
//! 테스트: cargo test -p ch08_traits_generics --example solution

use std::fmt;

// ─────────────────────────────────────────────
// 트레이트: "이런 메서드들을 가진 타입"이라는 약속(= C++의 추상 클래스 / Python의 Protocol)
// ─────────────────────────────────────────────
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Result<String, String>;

    // 기본 메서드: 구현하는 쪽이 따로 적지 않으면 이것을 씁니다.
    fn describe(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
}

// ── 도구 1: 계산기 ────────────────────────────
struct Calculator;

impl Tool for Calculator {
    fn name(&self) -> &str {
        "calculator"
    }
    fn description(&self) -> &str {
        "evaluates 'a op b' on integers (+ - * /)"
    }
    fn run(&self, input: &str) -> Result<String, String> {
        let invalid = || format!("invalid expression: {input}");
        let parts: Vec<&str> = input.split_whitespace().collect();
        // 슬라이스 패턴: 정확히 3조각일 때만 통과합니다.
        let [a, op, b] = parts[..] else {
            return Err(invalid());
        };
        let a: i64 = a.parse().map_err(|_| invalid())?;
        let b: i64 = b.parse().map_err(|_| invalid())?;
        let value = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => {
                if b == 0 {
                    return Err("division by zero".to_string());
                }
                a / b
            }
            _ => return Err(invalid()),
        };
        Ok(value.to_string())
    }
}

// ── 도구 2: 단어 세기 ──────────────────────────
struct WordCounter;

impl Tool for WordCounter {
    fn name(&self) -> &str {
        "word_counter"
    }
    fn description(&self) -> &str {
        "counts whitespace-separated words"
    }
    fn run(&self, input: &str) -> Result<String, String> {
        let n = input.split_whitespace().count();
        if n == 0 {
            Err("empty input".to_string())
        } else {
            Ok(n.to_string())
        }
    }
}

// ── 도구 3: 뒤집기 ────────────────────────────
struct Reverser;

impl Tool for Reverser {
    fn name(&self) -> &str {
        "reverser"
    }
    fn description(&self) -> &str {
        "reverses the characters of the input"
    }
    fn run(&self, input: &str) -> Result<String, String> {
        Ok(input.chars().rev().collect())
    }
}

// ─────────────────────────────────────────────
// ToolCall: Display 와 From 이라는 표준 트레이트 연습
// ─────────────────────────────────────────────
struct ToolCall {
    tool: String,
    input: String,
}

impl fmt::Display for ToolCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(\"{}\")", self.tool, self.input)
    }
}

impl From<(&str, &str)> for ToolCall {
    fn from((tool, input): (&str, &str)) -> Self {
        ToolCall {
            tool: tool.to_string(),
            input: input.to_string(),
        }
    }
}

// ─────────────────────────────────────────────
// 레지스트리: 서로 다른 타입을 한 Vec 에 담으려면 Box<dyn Tool> (동적 디스패치)
// ─────────────────────────────────────────────
struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    fn new() -> Self {
        ToolRegistry { tools: Vec::new() }
    }

    fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    fn names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    fn call(&self, name: &str, input: &str) -> Result<String, String> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| format!("unknown tool: {name}"))?;
        tool.run(input)
    }
}

// ─────────────────────────────────────────────
// 정적 vs 동적 디스패치
// ─────────────────────────────────────────────
fn show(result: &Result<String, String>) -> String {
    match result {
        Ok(v) => format!("ok: {v}"),
        Err(e) => format!("error: {e}"),
    }
}

// 정적: 컴파일러가 T 마다 함수 사본을 만듭니다 (단형화, C++ 템플릿과 같은 원리).
fn run_static<T: Tool>(tool: &T, input: &str) -> String {
    format!("{}(\"{}\") -> {}", tool.name(), input, show(&tool.run(input)))
}

// 동적: 하나의 함수가 vtable 을 통해 호출합니다 (C++ virtual 과 같은 원리).
fn run_dyn(tool: &dyn Tool, input: &str) -> String {
    format!("{}(\"{}\") -> {}", tool.name(), input, show(&tool.run(input)))
}

// ─────────────────────────────────────────────
// 제네릭 함수: C++ 의 template<std::totally_ordered T>
// ─────────────────────────────────────────────
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter().copied();
    let first = iter.next()?;
    Some(iter.fold(first, |best, x| if x > best { x } else { best }))
}

fn main() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(Calculator));
    registry.register(Box::new(WordCounter));
    registry.register(Box::new(Reverser));

    println!("== registry ==");
    println!("tools: {}", registry.names().join(", "));
    for tool in &registry.tools {
        println!("{}", tool.describe());
    }

    println!("== calls ==");
    let calls: Vec<ToolCall> = [
        ("calculator", "12 * 3"),
        ("word_counter", "the quick brown fox"),
        ("reverser", "hello"),
        ("calculator", "7 / 0"),
        ("calculator", "abc"),
        ("word_counter", ""),
        ("translator", "hi"),
    ]
    .into_iter()
    .map(ToolCall::from)
    .collect();
    for call in &calls {
        let result = registry.call(&call.tool, &call.input);
        println!("{call} -> {}", show(&result));
    }

    println!("== dispatch ==");
    println!("static : {}", run_static(&Reverser, "abc"));
    println!("dynamic: {}", run_dyn(&Reverser, "abc"));

    println!("== generics ==");
    println!("largest ints: {}", largest(&[3, 42, 7]).unwrap());
    println!("largest floats: {:.2}", largest(&[1.5, 3.5, 2.25]).unwrap());
    println!("largest chars: {}", largest(&['a', 'z', 'm']).unwrap());
    match largest::<i32>(&[]) {
        Some(v) => println!("largest empty: {v}"),
        None => println!("largest empty: none"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_basic_ops() {
        let c = Calculator;
        assert_eq!(c.run("12 * 3"), Ok("36".to_string()));
        assert_eq!(c.run("10 - 15"), Ok("-5".to_string()));
        assert_eq!(c.run("9 / 2"), Ok("4".to_string()));
    }

    #[test]
    fn calculator_errors() {
        let c = Calculator;
        assert_eq!(c.run("7 / 0"), Err("division by zero".to_string()));
        assert_eq!(c.run("abc"), Err("invalid expression: abc".to_string()));
        assert_eq!(c.run("1 ^ 2"), Err("invalid expression: 1 ^ 2".to_string()));
        assert_eq!(c.run("x + 2"), Err("invalid expression: x + 2".to_string()));
    }

    #[test]
    fn word_counter_and_reverser() {
        assert_eq!(WordCounter.run("a  b\tc"), Ok("3".to_string()));
        assert_eq!(WordCounter.run("   "), Err("empty input".to_string()));
        assert_eq!(Reverser.run("hello"), Ok("olleh".to_string()));
    }

    #[test]
    fn default_method_describe() {
        assert_eq!(
            Reverser.describe(),
            "reverser: reverses the characters of the input"
        );
    }

    #[test]
    fn tool_call_display_and_from() {
        let call = ToolCall::from(("reverser", "hi"));
        assert_eq!(call.tool, "reverser");
        assert_eq!(call.to_string(), "reverser(\"hi\")");
    }

    #[test]
    fn registry_dispatch() {
        let mut r = ToolRegistry::new();
        r.register(Box::new(Calculator));
        r.register(Box::new(Reverser));
        assert_eq!(r.names(), vec!["calculator", "reverser"]);
        assert_eq!(r.call("reverser", "ab"), Ok("ba".to_string()));
        assert_eq!(r.call("calculator", "2 + 2"), Ok("4".to_string()));
        assert_eq!(r.call("nope", "x"), Err("unknown tool: nope".to_string()));
    }

    #[test]
    fn static_and_dynamic_agree() {
        assert_eq!(run_static(&Reverser, "abc"), run_dyn(&Reverser, "abc"));
        assert_eq!(run_dyn(&Calculator, "1 / 0"), "calculator(\"1 / 0\") -> error: division by zero");
    }

    #[test]
    fn largest_generic() {
        assert_eq!(largest(&[3, 42, 7]), Some(42));
        assert_eq!(largest(&[1.5, -2.0]), Some(1.5));
        assert_eq!(largest(&['a', 'z', 'm']), Some('z'));
        assert_eq!(largest::<u8>(&[]), None);
    }
}
