//! # Ch08. 트레이트와 제네릭 — 에이전트 도구 레지스트리 (동기 버전)
//!
//! ## 목표
//! - `trait Tool` 을 정의하고, 서로 다른 타입(Calculator / WordCounter / Reverser)이 구현하게 합니다.
//! - `Vec<Box<dyn Tool>>` (동적 디스패치) 와 `fn f<T: Tool>` (정적 디스패치) 의 차이를 몸으로 익힙니다.
//! - `Display`, `From` 같은 표준 트레이트를 직접 구현하고, 제네릭 함수 `largest<T>` 를 만듭니다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`
//! - C++   : `../cpp/main.cpp`
//!
//! ## 진행 방법
//! 1. 아래 `TODO(1)` 부터 번호 순서대로 `todo!()` 를 지우고 구현합니다.
//! 2. 한 개 풀 때마다 `cargo test -p ch08_traits_generics` 로 확인하세요.
//! 3. 🔬 실험 블록은 주석을 풀어서 컴파일러 에러를 직접 보는 용도예요. 보고 나면 다시 주석 처리합니다.
//!
//! ## 완료 기준
//! `cargo test -p ch08_traits_generics` 가 모두 초록색이고,
//! `../../scripts/compare.sh ch08` 에서 Python / C++ / Rust 출력이 똑같으면 끝!
//! (막히면 `examples/solution.rs` 를 열어 보세요.)

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 컴파일러가 안 쓴 코드를 알려 줍니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::fmt;

// ─────────────────────────────────────────────
// 트레이트 정의 (이미 완성된 뼈대)
//   trait = "이런 메서드를 가진 타입"이라는 약속.
//   🐍 Python: typing.Protocol / 덕 타이핑   ⚙️ C++: 순수 가상 함수를 가진 추상 클래스 / concept
// ─────────────────────────────────────────────
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Result<String, String>;

    // ─────────────────────────────────────────────
    // TODO(1): 기본 메서드 `describe` 를 구현하세요. "이름: 설명" 형태의 String 을 돌려줍니다.
    //   예) "reverser: reverses the characters of the input"
    //   🐍 Python: f"{tool.name()}: {tool.description()}"          (python/main.py: describe)
    //   ⚙️ C++  : virtual std::string describe() const { ... }     (cpp/main.cpp: Tool::describe)
    //   🦀 Rust : trait 안에 본문을 적으면 "기본 구현"이 됩니다. 구현하는 타입은 안 적어도 돼요.
    //   💡 힌트: `self.name()` 과 `self.description()` 을 부를 수 있어요. format! 을 쓰세요.
    //   ❓ 왜?: 기본 메서드는 같은 트레이트의 다른 메서드만 써서 공통 동작을 한 번만 적게 해 줍니다.
    // ─────────────────────────────────────────────
    fn describe(&self) -> String {
        todo!("TODO(1): name 과 description 을 합쳐 반환")
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

    // ─────────────────────────────────────────────
    // TODO(2): "12 * 3" 같은 입력을 계산해 결과를 문자열로 돌려주세요.
    //   - 정확히 3개 조각(숫자 연산자 숫자)이 아니거나 숫자가 아니거나 모르는 연산자 → Err("invalid expression: <입력 그대로>")
    //   - 0으로 나누기 → Err("division by zero")
    //   - 정수(i64) 계산, 나눗셈은 정수 나눗셈 (9 / 2 = 4)
    //   🐍 Python: parts = input.split(); int(...)                  (python/main.py: Calculator.run)
    //   ⚙️ C++  : split + std::stoll + if/else 사슬                  (cpp/main.cpp: Calculator::run)
    //   🦀 Rust : split_whitespace(), parse::<i64>(), match op { ... }, Err 반환은 `return Err(..)`
    //   💡 힌트: `let [a, op, b] = parts[..] else { return Err(..) };` 같은 슬라이스 패턴을 쓰면 깔끔해요.
    //           parse 에러는 `.map_err(|_| ...)?` 로 우리 에러 문자열로 바꿔 보세요.
    //   ❓ 왜?: 예외 대신 Result 로 "실패할 수 있음"을 타입에 드러내면 호출하는 쪽이 처리를 잊을 수 없어요.
    // ─────────────────────────────────────────────
    fn run(&self, input: &str) -> Result<String, String> {
        todo!("TODO(2): 계산기 구현")
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

    // ─────────────────────────────────────────────
    // TODO(3): 공백으로 구분된 단어 개수를 문자열로 돌려주세요. 단어가 0개면 Err("empty input").
    //   🐍 Python: len(input.split())                  (python/main.py: WordCounter.run)
    //   ⚙️ C++  : split_ws(input).size()               (cpp/main.cpp: WordCounter::run)
    //   🦀 Rust : input.split_whitespace().count()
    //   💡 힌트: usize 는 `.to_string()` 으로 문자열이 됩니다.
    // ─────────────────────────────────────────────
    fn run(&self, input: &str) -> Result<String, String> {
        todo!("TODO(3): 단어 세기 구현")
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

    // ─────────────────────────────────────────────
    // TODO(4): 입력 문자열을 글자(char) 단위로 뒤집어 Ok 로 돌려주세요.
    //   🐍 Python: input[::-1]                          (python/main.py: Reverser.run)
    //   ⚙️ C++  : std::string(input.rbegin(), input.rend())   (cpp/main.cpp: Reverser::run)
    //   🦀 Rust : chars() 이터레이터를 뒤집고 String 으로 모읍니다.
    //   💡 힌트: `.chars().rev().collect::<String>()`
    //   ❓ 왜?: Rust 의 String 은 UTF-8 이라 바이트가 아니라 char 단위로 뒤집어야 한글도 안 깨져요.
    // ─────────────────────────────────────────────
    fn run(&self, input: &str) -> Result<String, String> {
        todo!("TODO(4): 뒤집기 구현")
    }
}

// ─────────────────────────────────────────────
// ToolCall: "어떤 도구를 어떤 입력으로 부를지" 한 건
// ─────────────────────────────────────────────
struct ToolCall {
    tool: String,
    input: String,
}

// ─────────────────────────────────────────────
// TODO(5): `impl fmt::Display for ToolCall` — 출력 형태는 `reverser("hi")` (입력은 큰따옴표로 감쌈).
//   🐍 Python: def __str__(self): return f'{self.tool}("{self.input}")'      (python/main.py: ToolCall.__str__)
//   ⚙️ C++  : std::string to_string() const / operator<<                     (cpp/main.cpp: ToolCall::to_string)
//   🦀 Rust : `write!(f, "...", ...)` 의 결과를 그대로 반환합니다.
//   💡 힌트: 따옴표 문자는 문자열 안에서 `\"` 로 적습니다.
//   ❓ 왜?: Display 를 구현하면 `println!("{call}")` 과 `.to_string()` 이 공짜로 따라와요.
// ─────────────────────────────────────────────
impl fmt::Display for ToolCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("TODO(5): write! 로 tool(\"input\") 출력")
    }
}

// ─────────────────────────────────────────────
// TODO(6): `impl From<(&str, &str)> for ToolCall` — ("reverser", "hi") 튜플로부터 ToolCall 을 만듭니다.
//   🐍 Python: ToolCall(t, i)                         (python/main.py: ToolCall)
//   ⚙️ C++  : 집합 초기화 {"reverser", "hi"}            (cpp/main.cpp: calls)
//   🦀 Rust : From 을 구현하면 `ToolCall::from(("a","b"))` 와 `.into()` 가 모두 생깁니다.
//   💡 힌트: 매개변수 자리에서 튜플을 바로 분해할 수 있어요: `fn from((tool, input): (&str, &str)) -> Self`
//           &str → String 은 `.to_string()`
// ─────────────────────────────────────────────
impl From<(&str, &str)> for ToolCall {
    fn from((tool, input): (&str, &str)) -> Self {
        todo!("TODO(6): 튜플에서 ToolCall 만들기")
    }
}

// ─────────────────────────────────────────────
// 레지스트리: 서로 다른 타입을 한 Vec 에 담으려면 Box<dyn Tool> 이 필요합니다.
//   (Calculator 와 Reverser 는 크기가 달라서 Vec<Tool> 은 불가능 → 힙에 두고 포인터(Box)로 들고 있어요)
//   ⚙️ C++: std::vector<std::unique_ptr<Tool>>
// ─────────────────────────────────────────────
struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    fn new() -> Self {
        ToolRegistry { tools: Vec::new() }
    }

    // ─────────────────────────────────────────────
    // TODO(7): 도구를 등록하세요 (소유권이 레지스트리로 넘어옵니다).
    //   🐍 Python: self.tools.append(tool)                 (python/main.py: ToolRegistry.register)
    //   ⚙️ C++  : tools.push_back(std::move(tool))          (cpp/main.cpp: ToolRegistry::add)
    //   🦀 Rust : Vec::push
    //   ❓ 왜?: Box 를 값으로 받으므로 호출한 쪽은 더 이상 그 도구를 쓸 수 없어요 (C++ 의 std::move 와 같은데 이후 사용은 컴파일 에러).
    // ─────────────────────────────────────────────
    fn register(&mut self, tool: Box<dyn Tool>) {
        todo!("TODO(7): 도구 등록")
    }

    // ─────────────────────────────────────────────
    // TODO(8): 등록된 도구 이름들을 등록 순서대로 `Vec<&str>` 로 돌려주세요.
    //   💡 힌트: iter().map(|t| t.name()).collect()
    //   ❓ 왜?: 반환 타입의 &str 은 self 를 빌려 온 것이라 레지스트리보다 오래 못 살아요 (ch09 에서 자세히!).
    // ─────────────────────────────────────────────
    fn names(&self) -> Vec<&str> {
        todo!("TODO(8): 이름 목록")
    }

    // ─────────────────────────────────────────────
    // TODO(9): 이름으로 도구를 찾아 실행하세요. 없으면 Err("unknown tool: <이름>").
    //   🐍 Python: for tool in self.tools: if tool.name() == name: return tool.run(input)
    //   ⚙️ C++  : t->name() == name  →  t->run(input)    (virtual 호출 = vtable 을 통한 동적 디스패치)
    //   🦀 Rust : `tool.run(input)` 한 줄이 vtable 을 통해 실제 타입의 run 으로 갑니다.
    //   💡 힌트: iter().find(..) 는 Option 을 줘요 → .ok_or_else(|| format!(..))? 로 Result 로 바꾸세요.
    // ─────────────────────────────────────────────
    fn call(&self, name: &str, input: &str) -> Result<String, String> {
        todo!("TODO(9): 이름으로 찾아 실행")
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

// ─────────────────────────────────────────────
// TODO(10): 정적 디스패치 — 제네릭 `T: Tool` 로 받아서 `reverser("abc") -> ok: cba` 형태 문자열을 만드세요.
//   형식: `{name}("{input}") -> {show(결과)}`
//   🐍 Python: (항상 동적)                                   (python/main.py: run_any)
//   ⚙️ C++  : template <std::derived_from<Tool> T>           (cpp/main.cpp: run_static)
//   🦀 Rust : 컴파일러가 T 마다 함수를 따로 만들어 줍니다(단형화). vtable 이 없어서 인라인도 가능해요.
//   ❓ 왜?: 속도가 중요하고 타입이 컴파일 때 정해지면 정적, 서로 다른 타입을 한 컬렉션에 섞으려면 동적.
// ─────────────────────────────────────────────
fn run_static<T: Tool>(tool: &T, input: &str) -> String {
    todo!("TODO(10): 정적 디스패치")
}

// ─────────────────────────────────────────────
// TODO(11): 동적 디스패치 — `&dyn Tool` 로 받아서 TODO(10) 과 똑같은 형식의 문자열을 만드세요.
//   ⚙️ C++ : const Tool& tool (virtual)                      (cpp/main.cpp: run_dyn)
//   🦀 Rust : `&dyn Tool` 은 "데이터 포인터 + vtable 포인터" 두 칸짜리 뚱뚱한 포인터(fat pointer)입니다.
//   💡 본문은 TODO(10) 과 같아요. 시그니처만 다릅니다!
// ─────────────────────────────────────────────
fn run_dyn(tool: &dyn Tool, input: &str) -> String {
    todo!("TODO(11): 동적 디스패치")
}

// ─────────────────────────────────────────────
// TODO(12): 제네릭 함수 `largest` — 슬라이스에서 가장 큰 값을 `Option<T>` 로 돌려주세요 (빈 슬라이스면 None).
//   🐍 Python: max(items, default=None)                       (python/main.py: largest)
//   ⚙️ C++  : template <typename T> requires std::totally_ordered<T> && std::copyable<T>   (cpp/main.cpp: largest)
//   🦀 Rust : `T: PartialOrd + Copy` — "비교할 수 있고 복사할 수 있는 타입이면 뭐든 OK"
//   💡 힌트: items.iter().copied() 로 값을 복사해 가며 fold 하거나, for 루프로 `if x > best` 비교.
//           `let first = iter.next()?;` 의 `?` 는 Option 에도 쓸 수 있어요 (None 이면 바로 None 반환).
//   ❓ 왜?: f64 는 NaN 때문에 `Ord` 가 아니라 `PartialOrd` 만 구현해요. 그래서 bound 도 PartialOrd 입니다.
// ─────────────────────────────────────────────
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    todo!("TODO(12): 가장 큰 값 찾기")
}

// ─────────────────────────────────────────────
// 🔬 실험 1: bound 를 빼먹으면?  (E0369)
//   아래 함수의 주석을 풀고 `cargo build -p ch08_traits_generics` 를 해 보세요.
//   ❓ 왜 `T` 끼리 `>` 비교가 안 될까요? C++ 템플릿은 "써 보고" 에러가 나지만(늦은 검사),
//      Rust 제네릭은 선언만 보고 검사합니다. 에러 메시지가 어떤 bound 를 제안하나요?
//
// fn largest_no_bound<T>(items: &[T]) -> &T {
//     let mut best = &items[0];
//     for x in items {
//         if x > best {
//             best = x;
//         }
//     }
//     best
// }
//
// 🔬 실험 2: 객체 안전(object safety) — `dyn` 으로 못 쓰는 트레이트  (E0038)
//   `Self` 를 반환하는 메서드나 제네릭 메서드는 vtable 한 칸에 담을 수 없어요.
//
// trait Cloner {
//     fn duplicate(&self) -> Self;
// }
// fn use_it(c: &dyn Cloner) {}
//
// 🔬 실험 3: 필수 메서드를 빼먹으면?  (E0046)
//   TODO(4) 의 `impl Tool for Reverser` 에서 `fn description` 을 잠시 지워 보세요.
// ─────────────────────────────────────────────

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
