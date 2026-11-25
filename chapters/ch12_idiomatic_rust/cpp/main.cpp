// Ch12. Rust다운 코드(관용구) - C++20 참조 구현.
// 주제: AgentConfig 빌더 + 검증 + 파싱 + 리포트. (Python 버전과 같은 출력)
//
// Rust 와 비교 포인트
//   - Builder 는 `Builder&` 를 돌려주는 체인 (Rust 는 self 를 값으로 받아 값으로 돌려줌 -> 소유권 이동)
//   - 오류 반환: C++20 에는 std::expected 가 없어서(C++23) 작은 Result 구조체를 직접 만듭니다.
//     Rust 는 Result<T, E> 가 표준이고 `?` 연산자로 전파합니다.
#include <algorithm>
#include <charconv>
#include <format>
#include <iostream>
#include <optional>
#include <set>
#include <string>
#include <string_view>
#include <vector>

struct AgentConfig {
    std::string name;
    std::string model = "default-model";
    unsigned max_tokens = 1024;
    double temperature = 1.0;
    std::vector<std::string> tools;
    std::optional<std::string> system_prompt;
    unsigned retries = 2;
    unsigned timeout_secs = 30;
};

// Rust 의 Result<AgentConfig, ConfigError> 에 해당
struct Built {
    std::optional<AgentConfig> value;
    std::string error;
};

std::string join_tools(const std::vector<std::string>& tools) {
    std::string out;
    for (std::size_t i = 0; i < tools.size(); ++i) {
        if (i > 0) out += ", ";
        out += tools[i];
    }
    return out;
}

// Rust: impl fmt::Display for AgentConfig
std::string to_string(const AgentConfig& c) {
    std::string prompt = c.system_prompt ? std::format("{} chars", c.system_prompt->size()) : "none";
    return std::format("{} | model={} | tokens={} | temp={:.2f} | retries={} | timeout={}s | tools=[{}] | prompt={}",
                       c.name, c.model, c.max_tokens, c.temperature, c.retries, c.timeout_secs,
                       join_tools(c.tools), prompt);
}

std::string trim(std::string_view s) {
    while (!s.empty() && s.front() == ' ') s.remove_prefix(1);
    while (!s.empty() && s.back() == ' ') s.remove_suffix(1);
    return std::string(s);
}

class AgentConfigBuilder {
public:
    explicit AgentConfigBuilder(std::string name) { cfg_.name = std::move(name); }
    AgentConfigBuilder& name(std::string v) { cfg_.name = std::move(v); return *this; }
    AgentConfigBuilder& model(std::string v) { cfg_.model = std::move(v); return *this; }
    AgentConfigBuilder& max_tokens(unsigned v) { cfg_.max_tokens = v; return *this; }
    AgentConfigBuilder& temperature(double v) { cfg_.temperature = v; return *this; }
    AgentConfigBuilder& tool(std::string v) { cfg_.tools.push_back(std::move(v)); return *this; }
    AgentConfigBuilder& tools(std::vector<std::string> vs) {
        for (auto& v : vs) cfg_.tools.push_back(std::move(v));
        return *this;
    }
    AgentConfigBuilder& system_prompt(std::string v) { cfg_.system_prompt = std::move(v); return *this; }
    AgentConfigBuilder& retries(unsigned v) { cfg_.retries = v; return *this; }
    AgentConfigBuilder& timeout_secs(unsigned v) { cfg_.timeout_secs = v; return *this; }

    Built build() const {
        const AgentConfig& c = cfg_;
        if (trim(c.name).empty()) return {std::nullopt, "missing required field: name"};
        if (trim(c.model).empty()) return {std::nullopt, "missing required field: model"};
        if (c.max_tokens < 1 || c.max_tokens > 8192)
            return {std::nullopt, std::format("max_tokens {} out of range 1..=8192", c.max_tokens)};
        if (!(c.temperature >= 0.0 && c.temperature <= 2.0))
            return {std::nullopt, std::format("temperature {:.1f} out of range 0.0..=2.0", c.temperature)};
        if (c.retries > 5) return {std::nullopt, std::format("retries {} exceeds limit 5", c.retries)};
        std::set<std::string> seen;
        for (const auto& t : c.tools) {
            if (!seen.insert(t).second) return {std::nullopt, "duplicate tool: " + t};
        }
        return {c, ""};
    }

private:
    AgentConfig cfg_;
};

template <typename T>
bool parse_num(std::string_view s, T& out) {
    auto [ptr, ec] = std::from_chars(s.data(), s.data() + s.size(), out);
    return ec == std::errc{} && ptr == s.data() + s.size();
}

// Rust: impl TryFrom<&str> for AgentConfig
Built parse_config(std::string_view spec) {
    AgentConfigBuilder b("");
    std::size_t pos = 0;
    while (pos <= spec.size()) {
        std::size_t end = spec.find(';', pos);
        if (end == std::string_view::npos) end = spec.size();
        std::string pair = trim(spec.substr(pos, end - pos));
        pos = end + 1;
        if (pair.empty()) continue;
        auto eq = pair.find('=');
        if (eq == std::string::npos) return {std::nullopt, "malformed pair: '" + pair + "'"};
        std::string key = trim(pair.substr(0, eq));
        std::string value = trim(pair.substr(eq + 1));
        auto invalid = [&](const char* k) { return Built{std::nullopt, std::format("invalid value for {}: '{}'", k, value)}; };
        if (key == "name") {
            b.name(value);
        } else if (key == "model") {
            b.model(value);
        } else if (key == "max_tokens") {
            unsigned v;
            if (!parse_num(value, v)) return invalid("max_tokens");
            b.max_tokens(v);
        } else if (key == "temperature") {
            // from_chars(double) 는 g++ 13 에서 지원됩니다.
            double v;
            if (!parse_num(value, v)) return invalid("temperature");
            b.temperature(v);
        } else if (key == "retries") {
            unsigned v;
            if (!parse_num(value, v)) return invalid("retries");
            b.retries(v);
        } else if (key == "timeout") {
            unsigned v;
            if (!parse_num(value, v)) return invalid("timeout");
            b.timeout_secs(v);
        } else if (key == "tools") {
            std::vector<std::string> ts;
            std::size_t p = 0;
            while (p <= value.size()) {
                std::size_t e = value.find(',', p);
                if (e == std::string::npos) e = value.size();
                std::string t = trim(std::string_view(value).substr(p, e - p));
                if (!t.empty()) ts.push_back(t);
                p = e + 1;
            }
            b.tools(std::move(ts));
        } else if (key == "prompt") {
            b.system_prompt(value);
        } else {
            return {std::nullopt, "unknown key: " + key};
        }
    }
    return b.build();
}

// ───────────── 작은 유틸리티 ─────────────
std::size_t count_long_names(const std::vector<std::string>& names, std::size_t min_len) {
    return static_cast<std::size_t>(std::ranges::count_if(names, [&](const std::string& n) { return n.size() >= min_len; }));
}

std::optional<std::size_t> find_tool(const std::vector<std::string>& tools, std::string_view name) {
    auto it = std::ranges::find(tools, name);
    if (it == tools.end()) return std::nullopt;
    return static_cast<std::size_t>(it - tools.begin());
}

std::size_t prompt_len(const AgentConfig& c) { return c.system_prompt ? c.system_prompt->size() : 0; }

std::optional<std::string> first_tool_upper(const AgentConfig& c) {
    if (c.tools.empty()) return std::nullopt;
    std::string s = c.tools.front();
    std::ranges::transform(s, s.begin(), [](unsigned char ch) { return static_cast<char>(std::toupper(ch)); });
    return s;
}

const char* describe_temperature(double t) {
    if (t < 0.3) return "precise";
    if (t < 0.8) return "balanced";
    return "creative";
}

std::string report(std::vector<AgentConfig> configs) {  // 값으로 받아서(복사) 정렬합니다
    std::string out = std::format("{:<8}|{:>7}|{:>6}\n", "name", "tokens", "temp");
    unsigned total = 0;
    for (const auto& c : configs) total += c.max_tokens;
    const AgentConfig* largest = &configs.front();
    for (const auto& c : configs)
        if (c.max_tokens > largest->max_tokens) largest = &c;
    const std::string largest_name = largest->name;
    const unsigned largest_tokens = largest->max_tokens;
    std::ranges::sort(configs, {}, &AgentConfig::name);
    for (const auto& c : configs) out += std::format("{:<8}|{:>7}|{:>6.2f}\n", c.name, c.max_tokens, c.temperature);
    out += std::format("total tokens: {}\n", total);
    out += std::format("largest: {} ({})", largest_name, largest_tokens);
    return out;
}

template <typename T>
std::string opt(const std::optional<T>& v) {
    if (!v) return "none";
    if constexpr (std::is_same_v<T, std::string>) return *v;
    else return std::to_string(*v);
}

AgentConfig must(const Built& b) { return *b.value; }

int main() {
    std::cout << "== 1. builder ==\n";
    auto helper = must(AgentConfigBuilder("helper")
                           .model("claude-sonnet").max_tokens(1024).temperature(0.7)
                           .tool("search").tool("calc").system_prompt("You are a helpful assistant.").build());
    auto writer = must(AgentConfigBuilder("writer")
                           .model("claude-opus").max_tokens(4096).temperature(0.3).retries(1).timeout_secs(60).build());
    auto coder = must(AgentConfigBuilder("coder").tool("shell").system_prompt("Write code.").build());
    for (const auto& c : {helper, writer, coder}) std::cout << to_string(c) << "\n";

    std::cout << "== 2. validation ==\n";
    struct Case { const char* label; Built built; };
    std::vector<Case> cases;
    cases.push_back({"empty name", AgentConfigBuilder("").build()});
    cases.push_back({"hot temperature", AgentConfigBuilder("a").temperature(3.5).build()});
    cases.push_back({"zero tokens", AgentConfigBuilder("a").max_tokens(0).build()});
    cases.push_back({"too many retries", AgentConfigBuilder("a").retries(9).build()});
    cases.push_back({"duplicate tool", AgentConfigBuilder("a").tool("search").tool("calc").tool("search").build()});
    for (const auto& c : cases) {
        if (c.built.value) std::cout << "  " << c.label << ": ok\n";
        else std::cout << "  " << c.label << ": error: " << c.built.error << "\n";
    }

    std::cout << "== 3. parse ==\n";
    const std::vector<std::string> specs = {
        "name=writer;model=claude-opus;max_tokens=2048;temperature=0.3;tools=search,calc;prompt=Be brief",
        "name=scribe",
        "name=x; max_tokens=abc",
        "model=m1",
        "name=x;color=red",
        "name=x;oops",
        "name=x;temperature=9",
    };
    for (std::size_t i = 0; i < specs.size(); ++i) {
        Built r = parse_config(specs[i]);
        if (r.value) std::cout << std::format("  [{}] ok: {}\n", i + 1, to_string(*r.value));
        else std::cout << std::format("  [{}] error: {}\n", i + 1, r.error);
    }

    std::cout << "== 4. utilities ==\n";
    const std::vector<std::string> tools = {"search", "calc", "shell", "web_browser_long"};
    std::cout << "join_tools: " << join_tools(tools) << "\n";
    std::cout << "count_long_names(>=5): " << count_long_names(tools, 5) << "\n";
    std::cout << "find_tool shell: " << opt(find_tool(tools, "shell")) << "\n";
    std::cout << "find_tool python: " << opt(find_tool(tools, "python")) << "\n";
    std::cout << "prompt_len helper: " << prompt_len(helper) << "\n";
    std::cout << "prompt_len writer: " << prompt_len(writer) << "\n";
    std::cout << "first_tool_upper helper: " << opt(first_tool_upper(helper)) << "\n";
    std::cout << "first_tool_upper writer: " << opt(first_tool_upper(writer)) << "\n";
    for (double t : {0.0, 0.5, 1.5}) std::cout << std::format("temperature {:.1f}: {}\n", t, describe_temperature(t));

    std::cout << "== 5. report ==\n";
    std::cout << report({helper, writer, coder}) << "\n";
}
