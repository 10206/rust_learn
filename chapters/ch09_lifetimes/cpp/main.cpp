// Ch09 C++20 참조 구현: 제로카피 로그 파서.
//
// std::string_view 는 Rust 의 &str 과 같은 "(포인터, 길이)" 쌍입니다. 복사 없이 빠르지만,
// **원본 문자열이 먼저 사라져도 컴파일러가 막아 주지 않습니다.** (= 댕글링, UB)
// Rust 의 라이프타임은 바로 이 구멍을 컴파일 시점에 막아 주는 장치예요.
#include <algorithm>
#include <fstream>
#include <iostream>
#include <map>
#include <optional>
#include <sstream>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

struct LogEntry {
    std::string_view level;    // Rust: &'a str  -> 원본 text 를 가리킴
    std::string_view module;
    std::string_view message;
};

static const std::vector<std::string_view> LEVELS = {"DEBUG", "INFO", "WARN", "ERROR"};

// Rust: impl Display for LogEntry
static std::string to_string(const LogEntry& e) {
    return "[" + std::string(e.level) + "] " + std::string(e.module) + ": " + std::string(e.message);
}

// --- 댕글링 버그 예시 (주석을 풀면 컴파일은 되지만 UB!) ---------------------------
// std::string_view dangling() {
//     std::string s = "hello world";
//     return std::string_view(s).substr(0, 5);  // s 는 함수가 끝나면 파괴됨 -> 쓰레기를 가리킴
// }
// 같은 코드를 Rust 로 쓰면 E0515 (cannot return value referencing local variable) 로 컴파일이 안 됩니다.
// ----------------------------------------------------------------------------------

// line 이 가리키는 메모리가 살아 있는 동안만 결과가 유효합니다 (이 약속은 주석으로만 존재!).
static std::optional<LogEntry> parse_line(std::string_view line) {
    std::string_view parts[3];
    std::string_view rest = line;
    for (auto& p : parts) {  // splitn(4, ' ') 에 해당: 앞의 3조각만 잘라 내고 나머지는 rest
        auto pos = rest.find(' ');
        if (pos == std::string_view::npos) return std::nullopt;
        p = rest.substr(0, pos);
        rest.remove_prefix(pos + 1);
    }
    std::string_view level = parts[2];
    if (std::find(LEVELS.begin(), LEVELS.end(), level) == LEVELS.end()) return std::nullopt;
    auto sep = rest.find(": ");
    if (sep == std::string_view::npos) return std::nullopt;
    std::string_view module = rest.substr(0, sep);
    std::string_view message = rest.substr(sep + 2);
    if (module.empty() || module.find(' ') != std::string_view::npos || message.empty()) return std::nullopt;
    return LogEntry{level, module, message};
}

static bool is_blank(std::string_view s) {
    return s.find_first_not_of(" \t\r\n") == std::string_view::npos;
}

// text 는 호출한 쪽이 계속 들고 있어야 합니다.
static std::pair<std::vector<LogEntry>, int> parse_log(std::string_view text) {
    std::vector<LogEntry> entries;
    int skipped = 0;
    while (!text.empty()) {
        auto nl = text.find('\n');
        std::string_view line = text.substr(0, nl);
        text = (nl == std::string_view::npos) ? std::string_view{} : text.substr(nl + 1);
        if (is_blank(line)) continue;
        if (auto e = parse_line(line)) entries.push_back(*e);
        else ++skipped;
    }
    return {entries, skipped};
}

static std::vector<LogEntry> filter_by_level(const std::vector<LogEntry>& entries, std::string_view level) {
    std::vector<LogEntry> out;
    for (const auto& e : entries)
        if (e.level == level) out.push_back(e);
    return out;
}

static std::optional<std::string_view> longest_message(const std::vector<LogEntry>& entries) {
    if (entries.empty()) return std::nullopt;
    auto it = std::max_element(entries.begin(), entries.end(),
                               [](const LogEntry& a, const LogEntry& b) { return a.message.size() < b.message.size(); });
    return it->message;
}

static std::map<std::string_view, int> count_by_module(const std::vector<LogEntry>& entries) {
    std::map<std::string_view, int> counts;  // std::map 은 키 순서로 정렬 = Rust BTreeMap
    for (const auto& e : entries) ++counts[e.module];
    return counts;
}

static std::map<std::string_view, int> count_by_level(const std::vector<LogEntry>& entries) {
    std::map<std::string_view, int> counts;
    for (const auto& e : entries) ++counts[e.level];
    return counts;
}

static std::string_view longer(std::string_view a, std::string_view b) { return b.size() > a.size() ? b : a; }

int main() {
    std::ifstream in("data/app.log");
    std::stringstream buf;
    buf << in.rdbuf();
    const std::string text = buf.str();  // entries 가 이 text 를 가리킵니다 (먼저 파괴하면 UB)

    auto [entries, skipped] = parse_log(text);
    std::cout << "parsed " << entries.size() << " entries, skipped " << skipped << " malformed lines\n";

    for (std::string_view level : {"ERROR", "WARN"}) {
        std::cout << "== " << level << " entries ==\n";
        for (const auto& e : filter_by_level(entries, level)) std::cout << to_string(e) << "\n";
    }

    std::cout << "== longest message ==\n";
    if (auto m = longest_message(entries)) std::cout << *m << " (" << m->size() << " bytes)\n";

    std::cout << "== per module ==\n";
    for (const auto& [module, n] : count_by_module(entries)) std::cout << module << ": " << n << "\n";

    std::cout << "== per level ==\n";
    for (const auto& [level, n] : count_by_level(entries)) std::cout << level << ": " << n << "\n";

    std::cout << "== longer ==\n";
    std::cout << "longer(\"abc\", \"de\") = " << longer("abc", "de") << "\n";
}
