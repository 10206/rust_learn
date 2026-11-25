// Ch08 C++20 참조 구현: 에이전트 도구 레지스트리 (동기).
//
// Rust trait + Box<dyn Tool>  <->  C++ 추상 클래스 + unique_ptr<Tool> (virtual, vtable)
// Rust 제네릭 <T: Bound>      <->  C++ template + concept
#include <concepts>
#include <format>
#include <iostream>
#include <memory>
#include <optional>
#include <string>
#include <vector>

// Rust 의 Result<String, String> 대용 (C++23 의 std::expected 가 있다면 그걸 쓸 수 있어요).
struct ToolResult {
    bool ok;
    std::string text;
};

static ToolResult ok(std::string s) { return {true, std::move(s)}; }
static ToolResult err(std::string s) { return {false, std::move(s)}; }
static std::string show(const ToolResult& r) { return (r.ok ? "ok: " : "error: ") + r.text; }

// ── 추상 클래스 = Rust 의 trait ────────────────
// virtual 함수가 하나라도 있으면 객체마다 vtable 포인터가 숨어 들어갑니다.
class Tool {
public:
    virtual ~Tool() = default;  // 기반 클래스 소멸자를 virtual 로 - 안 하면 UB. Rust 는 이런 함정이 없어요.
    virtual std::string name() const = 0;
    virtual std::string description() const = 0;
    virtual ToolResult run(const std::string& input) const = 0;

    // 기본 메서드 (Rust: trait 안의 기본 구현)
    virtual std::string describe() const { return name() + ": " + description(); }
};

static std::vector<std::string> split_ws(const std::string& s) {
    std::vector<std::string> out;
    std::string cur;
    for (char c : s) {
        if (c == ' ' || c == '\t' || c == '\n') {
            if (!cur.empty()) { out.push_back(cur); cur.clear(); }
        } else {
            cur += c;
        }
    }
    if (!cur.empty()) out.push_back(cur);
    return out;
}

static std::optional<long long> parse_int(const std::string& s) {
    try {
        size_t pos = 0;
        long long v = std::stoll(s, &pos);
        if (pos != s.size()) return std::nullopt;
        return v;
    } catch (...) {
        return std::nullopt;
    }
}

class Calculator : public Tool {
public:
    std::string name() const override { return "calculator"; }
    std::string description() const override { return "evaluates 'a op b' on integers (+ - * /)"; }
    ToolResult run(const std::string& input) const override {
        auto invalid = [&] { return err("invalid expression: " + input); };
        auto parts = split_ws(input);
        if (parts.size() != 3) return invalid();
        auto a = parse_int(parts[0]);
        auto b = parse_int(parts[2]);
        if (!a || !b) return invalid();
        const std::string& op = parts[1];
        if (op == "+") return ok(std::to_string(*a + *b));
        if (op == "-") return ok(std::to_string(*a - *b));
        if (op == "*") return ok(std::to_string(*a * *b));
        if (op == "/") {
            if (*b == 0) return err("division by zero");
            return ok(std::to_string(*a / *b));
        }
        return invalid();
    }
};

class WordCounter : public Tool {
public:
    std::string name() const override { return "word_counter"; }
    std::string description() const override { return "counts whitespace-separated words"; }
    ToolResult run(const std::string& input) const override {
        auto n = split_ws(input).size();
        if (n == 0) return err("empty input");
        return ok(std::to_string(n));
    }
};

class Reverser : public Tool {
public:
    std::string name() const override { return "reverser"; }
    std::string description() const override { return "reverses the characters of the input"; }
    ToolResult run(const std::string& input) const override {
        // 주의: 바이트 단위 뒤집기라 ASCII 만 Rust 의 chars().rev() 와 같아요.
        return ok(std::string(input.rbegin(), input.rend()));
    }
};

struct ToolCall {
    std::string tool;
    std::string input;
    // Rust: impl Display
    std::string to_string() const { return std::format("{}(\"{}\")", tool, input); }
};

class ToolRegistry {
public:
    void add(std::unique_ptr<Tool> tool) { tools.push_back(std::move(tool)); }  // Rust: register
    std::vector<std::string> names() const {
        std::vector<std::string> out;
        for (const auto& t : tools) out.push_back(t->name());
        return out;
    }
    ToolResult call(const std::string& name, const std::string& input) const {
        for (const auto& t : tools)
            if (t->name() == name) return t->run(input);  // virtual 호출 = 동적 디스패치
        return err("unknown tool: " + name);
    }
    std::vector<std::unique_ptr<Tool>> tools;  // Rust: Vec<Box<dyn Tool>>
};

// 정적 디스패치: 템플릿. concept 로 "Tool 의 자식이어야 함"을 선언 (Rust 의 T: Tool).
template <std::derived_from<Tool> T>
std::string run_static(const T& tool, const std::string& input) {
    return std::format("{}(\"{}\") -> {}", tool.name(), input, show(tool.run(input)));
}

// 동적 디스패치: 기반 클래스 참조 (Rust 의 &dyn Tool).
std::string run_dyn(const Tool& tool, const std::string& input) {
    return std::format("{}(\"{}\") -> {}", tool.name(), input, show(tool.run(input)));
}

// Rust: fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T>
template <typename T>
    requires std::totally_ordered<T> && std::copyable<T>
std::optional<T> largest(const std::vector<T>& items) {
    if (items.empty()) return std::nullopt;
    T best = items[0];
    for (std::size_t i = 1; i < items.size(); ++i)
        if (items[i] > best) best = items[i];
    return best;
}

static std::string join(const std::vector<std::string>& v, const std::string& sep) {
    std::string out;
    for (std::size_t i = 0; i < v.size(); ++i) out += (i ? sep : "") + v[i];
    return out;
}

int main() {
    ToolRegistry registry;
    registry.add(std::make_unique<Calculator>());
    registry.add(std::make_unique<WordCounter>());
    registry.add(std::make_unique<Reverser>());

    std::cout << "== registry ==\n";
    std::cout << "tools: " << join(registry.names(), ", ") << "\n";
    for (const auto& t : registry.tools) std::cout << t->describe() << "\n";

    std::cout << "== calls ==\n";
    const std::vector<ToolCall> calls = {
        {"calculator", "12 * 3"},
        {"word_counter", "the quick brown fox"},
        {"reverser", "hello"},
        {"calculator", "7 / 0"},
        {"calculator", "abc"},
        {"word_counter", ""},
        {"translator", "hi"},
    };
    for (const auto& call : calls)
        std::cout << call.to_string() << " -> " << show(registry.call(call.tool, call.input)) << "\n";

    std::cout << "== dispatch ==\n";
    std::cout << "static : " << run_static(Reverser{}, "abc") << "\n";
    std::cout << "dynamic: " << run_dyn(Reverser{}, "abc") << "\n";

    std::cout << "== generics ==\n";
    std::cout << "largest ints: " << *largest(std::vector<int>{3, 42, 7}) << "\n";
    std::cout << std::format("largest floats: {:.2f}\n", *largest(std::vector<double>{1.5, 3.5, 2.25}));
    std::cout << "largest chars: " << *largest(std::vector<char>{'a', 'z', 'm'}) << "\n";
    auto empty = largest(std::vector<int>{});
    std::cout << "largest empty: " << (empty ? std::to_string(*empty) : std::string("none")) << "\n";
}
