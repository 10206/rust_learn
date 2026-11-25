// Ch06 설정 파일 파서 (C++20 참조 구현).
// C++ 는 예외(exception)로 실패를 알립니다. 시그니처에 `noexcept` 가 아니면 "던질 수도 있음" 일 뿐
// 어떤 예외인지 알 수 없습니다. (C++23 의 std::expected<T,E> 는 Rust 의 Result 와 비슷합니다.)
#include <algorithm>
#include <cctype>
#include <format>
#include <fstream>
#include <iostream>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

struct Config {
    std::string name;
    int port;
    bool debug;
    int max_retries;
};

// 에러 종류마다 예외 클래스를 만드는 대신, 메시지를 만드는 헬퍼를 두었습니다.
struct ConfigError : std::runtime_error {
    using std::runtime_error::runtime_error;
};

ConfigError missing_equals(int line) { return ConfigError(std::format("line {}: expected 'key = value'", line)); }
ConfigError unknown_key(int line, const std::string& key) { return ConfigError(std::format("line {}: unknown key '{}'", line, key)); }
ConfigError duplicate_key(int line, const std::string& key) { return ConfigError(std::format("line {}: duplicate key '{}'", line, key)); }
ConfigError invalid(int line, const std::string& key, const std::string& reason) {
    return ConfigError(std::format("line {}: invalid value for '{}': {}", line, key, reason));
}
ConfigError missing_key(const std::string& key) { return ConfigError(std::format("missing required key '{}'", key)); }

std::string show(const Config& c) {
    return std::format("name={} port={} debug={} max_retries={}", c.name, c.port, c.debug ? "true" : "false", c.max_retries);
}

const std::vector<std::pair<std::string, std::string>> BAD_INPUTS = {
    {"no equals", "name = x\nport 80\n"},
    {"unknown key", "name = x\nport = 80\ncolour = red\n"},
    {"port too big", "name = x\nport = 70000\n"},
    {"port not number", "name = x\nport = eighty\n"},
    {"negative port", "name = x\nport = -1\n"},
    {"bad bool", "name = x\nport = 80\ndebug = maybe\n"},
    {"retries too big", "name = x\nport = 80\nmax_retries = 300\n"},
    {"missing name", "# only a port\nport = 80\n"},
    {"duplicate", "name = a\nname = b\n"},
    {"empty name", "name =\nport = 1\n"},
};

std::string trim(const std::string& s) {
    auto b = s.find_first_not_of(" \t\r\n");
    if (b == std::string::npos) return "";
    auto e = s.find_last_not_of(" \t\r\n");
    return s.substr(b, e - b + 1);
}

// "없을 수 있음" 은 std::optional 로, 실패는 예외로 표현합니다.
std::optional<std::pair<std::string, std::string>> parse_line(int line_no, const std::string& raw) {
    std::string line = trim(raw);
    if (line.empty() || line[0] == '#') return std::nullopt;
    auto eq = line.find('=');
    if (eq == std::string::npos) throw missing_equals(line_no);
    return std::make_pair(trim(line.substr(0, eq)), trim(line.substr(eq + 1)));
}

long long parse_number(int line, const std::string& key, const std::string& value, long long max) {
    bool digits = !value.empty() && std::all_of(value.begin(), value.end(), [](unsigned char c) { return std::isdigit(c); });
    if (!digits) throw invalid(line, key, "not a valid number");
    std::string range_msg = std::format("out of range (0-{})", max);
    // 18자리를 넘으면 long long 에 안 들어갈 수 있으므로 바로 범위 초과 처리
    if (value.size() > 18) throw invalid(line, key, range_msg);
    long long n = std::stoll(value);
    if (n > max) throw invalid(line, key, range_msg);
    return n;
}

bool parse_bool(int line, const std::string& key, const std::string& value) {
    if (value == "true") return true;
    if (value == "false") return false;
    throw invalid(line, key, "expected true or false");
}

Config parse_config(const std::string& text) {
    std::optional<std::string> name;
    std::optional<int> port, max_retries;
    std::optional<bool> debug;

    std::istringstream in(text);
    std::string raw;
    int line_no = 0;
    while (std::getline(in, raw)) {
        ++line_no;
        auto kv = parse_line(line_no, raw);  // 예외는 자동 전파: Rust 의 `?` 에 해당
        if (!kv) continue;
        const auto& [key, value] = *kv;
        if (key == "name") {
            if (name) throw duplicate_key(line_no, key);
            if (value.empty()) throw invalid(line_no, key, "must not be empty");
            name = value;
        } else if (key == "port") {
            if (port) throw duplicate_key(line_no, key);
            port = static_cast<int>(parse_number(line_no, key, value, 65535));
        } else if (key == "debug") {
            if (debug) throw duplicate_key(line_no, key);
            debug = parse_bool(line_no, key, value);
        } else if (key == "max_retries") {
            if (max_retries) throw duplicate_key(line_no, key);
            max_retries = static_cast<int>(parse_number(line_no, key, value, 255));
        } else {
            throw unknown_key(line_no, key);
        }
    }
    if (!name) throw missing_key("name");
    if (!port) throw missing_key("port");
    return Config{*name, *port, debug.value_or(false), max_retries.value_or(3)};
}

Config load_config(const std::string& path) {
    std::ifstream f(path);
    if (!f) throw ConfigError("cannot read config file");
    std::stringstream ss;
    ss << f.rdbuf();
    return parse_config(ss.str());
}

int port_or_default(const std::string& text) {
    try {
        return parse_config(text).port;
    } catch (const ConfigError&) {
        return 8080;
    }
}

int main() {
    for (const char* path : {"data/app.conf", "data/missing.conf"}) {
        try {
            Config cfg = load_config(path);  // 먼저 평가해야 예외 시 "loaded: " 가 출력되지 않습니다.
            std::cout << "loaded: " << show(cfg) << "\n";
        } catch (const ConfigError& e) {
            std::cout << "error: " << e.what() << "\n";
        }
    }

    std::cout << "-- bad inputs --\n";
    for (const auto& [label, text] : BAD_INPUTS) {
        try {
            Config cfg = parse_config(text);
            std::cout << std::format("{:<16} ok?! {}\n", label, show(cfg));
        } catch (const ConfigError& e) {
            std::cout << std::format("{:<16} error: {}\n", label, e.what());
        }
    }

    std::cout << "-- defaults --\n";
    const std::string minimal = "name = tiny\nport = 9000\n";
    try {
        Config cfg = parse_config(minimal);
        std::cout << "minimal: " << show(cfg) << "\n";
    } catch (const ConfigError& e) {
        std::cout << "error: " << e.what() << "\n";
    }
    std::cout << "port_or_default(good) = " << port_or_default(minimal) << "\n";
    std::cout << "port_or_default(bad)  = " << port_or_default("port = x") << "\n";
}
