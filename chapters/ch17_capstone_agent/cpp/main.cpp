// Ch17. 캡스톤: 미니 AI 에이전트 (C++20 참조 구현)
// libcurl + nlohmann::json 으로 Claude Messages API 를 "손으로" 호출합니다. (Rust 버전과 같은 구조)
//
// 환경 변수
//   LLM_BASE_URL       기본값 http://127.0.0.1:8787 (목 서버).  실제 API: https://api.anthropic.com
//   ANTHROPIC_API_KEY  목 서버에서는 생략 가능 ("mock-key"), 실제 API 에서는 필수
//   LLM_MODEL          기본값 claude-opus-5-5
//   AGENT_TASK         기본 과제 대신 다른 과제를 주고 싶을 때
#include <curl/curl.h>

#include <cctype>
#include <cstdlib>
#include <format>
#include <fstream>
#include <iostream>
#include <memory>
#include <nlohmann/json.hpp>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

using json = nlohmann::json;

constexpr int MAX_ITERATIONS = 10;
constexpr int MAX_TOKENS = 16000;
constexpr size_t PREVIEW_LEN = 70;
const std::string DATA_DIR = "data";
const char* DEFAULT_TASK =
    "Read the note 'notes.txt', count its words, compute (12+30)*2, "
    "try to read '../secret.txt', and check the weather in Seoul. Then summarize.";
const char* SYSTEM_PROMPT = "You are a concise assistant. Use the provided tools when they help.";

// ───────────────────────── 설정 ─────────────────────────
struct Config {
    std::string base_url, api_key, model;
    bool is_real() const { return base_url.find("api.anthropic.com") != std::string::npos; }
};

Config config_from_env() {
    auto get = [](const char* k) -> const char* { return std::getenv(k); };
    Config cfg;
    cfg.base_url = get("LLM_BASE_URL") ? get("LLM_BASE_URL") : "http://127.0.0.1:8787";
    cfg.model = get("LLM_MODEL") ? get("LLM_MODEL") : "claude-opus-5-5";
    if (get("ANTHROPIC_API_KEY") && *get("ANTHROPIC_API_KEY")) {
        cfg.api_key = get("ANTHROPIC_API_KEY");
    } else if (!cfg.is_real()) {
        cfg.api_key = "mock-key";  // 목 서버일 때만 가짜 키 허용
    } else {
        throw std::runtime_error("ANTHROPIC_API_KEY is required for the real API");
    }
    return cfg;
}

// ───────────────────────── 도구 ─────────────────────────
// 도구 실패는 예외로 던지고, 레지스트리가 잡아서 "is_error 결과"로 바꿉니다.
struct ToolError : std::runtime_error {
    using std::runtime_error::runtime_error;
};

std::string required_str(const json& input, const char* key) {
    if (!input.is_object() || !input.contains(key) || !input[key].is_string())
        throw ToolError(std::string("missing string field `") + key + "`");
    return input[key].get<std::string>();
}

class Tool {
public:
    virtual ~Tool() = default;
    virtual std::string name() const = 0;
    virtual std::string description() const = 0;
    virtual json input_schema() const = 0;
    virtual std::string run(const json& input) const = 0;
    json definition() const {
        return {{"name", name()}, {"description", description()}, {"input_schema", input_schema()}};
    }
};

// + - * / 와 괄호, 소수를 지원하는 재귀 하강 계산기
class ExprParser {
public:
    explicit ExprParser(const std::string& text) {
        for (char c : text)
            if (c != ' ') s_.push_back(c);
    }
    double parse() {
        double v = expr();
        if (pos_ != s_.size()) throw ToolError(std::string("invalid expression: unexpected '") + s_[pos_] + "'");
        return v;
    }

private:
    char peek() const { return pos_ < s_.size() ? s_[pos_] : '\0'; }
    double expr() {
        double v = term();
        while (peek() == '+' || peek() == '-') {
            char op = s_[pos_++];
            double r = term();
            v = op == '+' ? v + r : v - r;
        }
        return v;
    }
    double term() {
        double v = factor();
        while (peek() == '*' || peek() == '/') {
            char op = s_[pos_++];
            double r = factor();
            if (op == '*') {
                v *= r;
            } else {
                if (r == 0) throw ToolError("division by zero");
                v /= r;
            }
        }
        return v;
    }
    double factor() {
        char c = peek();
        if (c == '-') {
            ++pos_;
            return -factor();
        }
        if (c == '(') {
            ++pos_;
            double v = expr();
            if (peek() != ')') throw ToolError("invalid expression: expected ')'");
            ++pos_;
            return v;
        }
        size_t start = pos_;
        while (std::isdigit(static_cast<unsigned char>(peek())) || peek() == '.') ++pos_;
        if (start == pos_)
            throw ToolError(c ? std::string("invalid expression: unexpected '") + c + "'" : "invalid expression: unexpected end");
        return std::stod(s_.substr(start, pos_ - start));
    }
    std::string s_;
    size_t pos_ = 0;
};

std::string format_number(double x) {
    if (x == static_cast<double>(static_cast<long long>(x)) && x > -1e15 && x < 1e15) return std::to_string(static_cast<long long>(x));
    return std::format("{:.2f}", x);
}

class Calculator : public Tool {
public:
    std::string name() const override { return "calculator"; }
    std::string description() const override { return "Evaluate an arithmetic expression with + - * / and parentheses."; }
    json input_schema() const override {
        return {{"type", "object"},
                {"properties", {{"expression", {{"type", "string"}, {"description", "e.g. (12+30)*2"}}}}},
                {"required", {"expression"}}};
    }
    std::string run(const json& input) const override { return format_number(ExprParser(required_str(input, "expression")).parse()); }
};

class WordCount : public Tool {
public:
    std::string name() const override { return "word_count"; }
    std::string description() const override { return "Count the whitespace-separated words in a text."; }
    json input_schema() const override {
        return {{"type", "object"}, {"properties", {{"text", {{"type", "string"}}}}}, {"required", {"text"}}};
    }
    std::string run(const json& input) const override {
        std::istringstream in(required_str(input, "text"));
        std::string w;
        int n = 0;
        while (in >> w) ++n;  // >> 는 공백 기준으로 끊어 읽습니다
        return std::to_string(n);
    }
};

// data/ 안의 '파일 이름 하나'만 허용. ../ 같은 경로 탈출은 거부합니다.
std::string safe_note_path(const std::string& name) {
    if (name.empty() || name == "." || name == ".." || name.find('/') != std::string::npos || name.find('\\') != std::string::npos)
        throw ToolError("path not allowed: " + name);
    return DATA_DIR + "/" + name;
}

class ReadNote : public Tool {
public:
    std::string name() const override { return "read_note"; }
    std::string description() const override { return "Read a text note from the data/ directory by file name."; }
    json input_schema() const override {
        return {{"type", "object"},
                {"properties", {{"name", {{"type", "string"}, {"description", "file name, e.g. notes.txt"}}}}},
                {"required", {"name"}}};
    }
    std::string run(const json& input) const override {
        std::string name = required_str(input, "name");
        std::ifstream f(safe_note_path(name));
        if (!f) throw ToolError("note not found: " + name);
        std::stringstream ss;
        ss << f.rdbuf();
        return ss.str();
    }
};

class GetWeather : public Tool {
public:
    std::string name() const override { return "get_weather"; }
    std::string description() const override { return "Get the (fixed, fake) current weather for a city."; }
    json input_schema() const override {
        return {{"type", "object"}, {"properties", {{"city", {{"type", "string"}}}}}, {"required", {"city"}}};
    }
    std::string run(const json& input) const override {
        std::string city = required_str(input, "city");
        if (city == "Seoul") return "18C, clear";
        if (city == "Busan") return "21C, cloudy";
        throw ToolError("no weather data for " + city);
    }
};

struct ToolOutcome {
    std::string content;
    bool is_error;
};

class ToolRegistry {
public:
    void add(std::unique_ptr<Tool> t) { tools_.push_back(std::move(t)); }
    json definitions() const {
        json arr = json::array();
        for (const auto& t : tools_) arr.push_back(t->definition());
        return arr;
    }
    ToolOutcome execute(const std::string& name, const json& input) const {
        for (const auto& t : tools_) {
            if (t->name() != name) continue;
            try {
                return {t->run(input), false};
            } catch (const ToolError& e) {
                return {e.what(), true};
            }
        }
        return {"unknown tool: " + name, true};
    }

private:
    std::vector<std::unique_ptr<Tool>> tools_;
};

// ───────────────────────── HTTP (libcurl) ─────────────────────────
static size_t write_cb(char* ptr, size_t size, size_t nmemb, void* userdata) {
    static_cast<std::string*>(userdata)->append(ptr, size * nmemb);
    return size * nmemb;
}

struct ApiError : std::runtime_error {
    using std::runtime_error::runtime_error;
};

class LlmClient {
public:
    explicit LlmClient(Config cfg) : cfg_(std::move(cfg)) {}

    json create_message(const json& body) const {
        std::unique_ptr<CURL, decltype(&curl_easy_cleanup)> curl(curl_easy_init(), curl_easy_cleanup);
        std::string response, payload = body.dump();
        curl_slist* headers = nullptr;
        headers = curl_slist_append(headers, "content-type: application/json");
        headers = curl_slist_append(headers, ("x-api-key: " + cfg_.api_key).c_str());
        headers = curl_slist_append(headers, "anthropic-version: 2026-10-21");
        headers = curl_slist_append(headers, "anthropic-beta: server-side-fallback-2026-07-01");

        curl_easy_setopt(curl.get(), CURLOPT_URL, (cfg_.base_url + "/v1/messages").c_str());
        curl_easy_setopt(curl.get(), CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl.get(), CURLOPT_POSTFIELDS, payload.c_str());
        curl_easy_setopt(curl.get(), CURLOPT_TIMEOUT, 600L);  // thinking 이 길 수 있으니 넉넉하게
        curl_easy_setopt(curl.get(), CURLOPT_NOSIGNAL, 1L);
        curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_cb);
        curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, &response);
        CURLcode rc = curl_easy_perform(curl.get());
        curl_slist_free_all(headers);
        if (rc != CURLE_OK) throw ApiError(std::string("network error: ") + curl_easy_strerror(rc));

        long status = 0;
        curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &status);
        json parsed = json::parse(response, nullptr, false);
        if (status != 200) {
            std::string msg = "(no message)";
            if (parsed.is_object() && parsed.contains("error") && parsed["error"].contains("message"))
                msg = parsed["error"]["message"].get<std::string>();
            throw ApiError(std::format("API error {}: {}", status, msg));
        }
        if (parsed.is_discarded()) throw ApiError("decode error: invalid JSON");
        return parsed;
    }

private:
    Config cfg_;
};

// ───────────────────────── 출력 도우미 ─────────────────────────
std::string preview(std::string s) {
    for (size_t p = 0; (p = s.find('\n', p)) != std::string::npos; p += 2) s.replace(p, 1, "\\n");
    return s.size() <= PREVIEW_LEN ? s : s.substr(0, PREVIEW_LEN - 3) + "...";
}

// ───────────────────────── 에이전트 루프 ─────────────────────────
struct AgentError : std::runtime_error {
    using std::runtime_error::runtime_error;
};

struct AgentOutcome {
    std::string final_text;
    int turns, tool_calls, tool_errors;
};

AgentOutcome run_agent(const LlmClient& llm, const Config& cfg, const ToolRegistry& registry, const std::string& task) {
    json messages = json::array({{{"role", "user"}, {"content", task}}});
    json tools = registry.definitions();
    int tool_calls = 0, tool_errors = 0;

    std::cout << "task: " << task << "\n";
    for (int turn = 1; turn <= MAX_ITERATIONS; ++turn) {
        json request = {{"model", cfg.model},
                        {"max_tokens", MAX_TOKENS},
                        {"system", SYSTEM_PROMPT},
                        {"tools", tools},
                        {"messages", messages},
                        {"output_config", {{"effort", "medium"}}},
                        {"fallbacks", "default"}};
        json response = llm.create_message(request);
        std::string stop = response.value("stop_reason", "");
        std::cout << "--- turn " << turn << " (stop_reason: " << stop << ") ---\n";

        // refusal 은 content 를 읽기 전에 먼저 확인합니다.
        if (stop == "refusal") throw AgentError("the model refused the request");

        const json& content = response.at("content");
        std::string text;
        for (const auto& block : content) {
            std::string type = block.value("type", "");
            if (type == "thinking") {
                std::cout << "thinking: [block kept]\n";
            } else if (type == "text") {
                text += block.value("text", "");
                if (stop != "end_turn") std::cout << "assistant: " << block.value("text", "") << "\n";
            } else if (type == "tool_use") {
                std::cout << "tool_use: " << block["name"].get<std::string>() << " " << preview(block["input"].dump()) << "\n";
            }
        }

        if (stop == "end_turn") return {text, turn, tool_calls, tool_errors};
        if (stop == "max_tokens") throw AgentError("the answer was cut off (max_tokens)");

        // assistant 의 content 는 thinking 블록까지 '그대로' 되돌려 보냅니다 (고치거나 빼지 않기!).
        messages.push_back({{"role", "assistant"}, {"content", content}});
        if (stop == "pause_turn") continue;
        if (stop != "tool_use") throw AgentError("unexpected stop_reason: " + stop);

        // 한 번의 assistant 턴에 나온 tool_result 는 user 메시지 '하나'에 모두 담습니다.
        json results = json::array();
        for (const auto& block : content) {
            if (block.value("type", "") != "tool_use") continue;
            ToolOutcome out = registry.execute(block["name"].get<std::string>(), block["input"]);
            ++tool_calls;
            if (out.is_error) ++tool_errors;
            std::cout << "result: " << block["name"].get<std::string>() << " " << (out.is_error ? "ERROR" : "ok") << " "
                      << preview(out.content) << "\n";
            json item = {{"type", "tool_result"}, {"tool_use_id", block["id"]}, {"content", out.content}};
            if (out.is_error) item["is_error"] = true;
            results.push_back(item);
        }
        messages.push_back({{"role", "user"}, {"content", results}});
    }
    throw AgentError(std::format("agent did not finish within {} iterations", MAX_ITERATIONS));
}

int main() {
    curl_global_init(CURL_GLOBAL_DEFAULT);
    int rc = 0;
    try {
        Config cfg = config_from_env();
        LlmClient llm(cfg);
        ToolRegistry registry;
        registry.add(std::make_unique<Calculator>());
        registry.add(std::make_unique<WordCount>());
        registry.add(std::make_unique<ReadNote>());
        registry.add(std::make_unique<GetWeather>());

        const char* env_task = std::getenv("AGENT_TASK");
        AgentOutcome out = run_agent(llm, cfg, registry, env_task ? env_task : DEFAULT_TASK);
        std::cout << "--- final answer ---\n" << out.final_text << "\n";
        std::cout << std::format("--- done: {} turns, {} tool calls, {} tool errors ---\n", out.turns, out.tool_calls, out.tool_errors);
    } catch (const std::exception& e) {
        std::cerr << "agent error: " << e.what() << "\n";
        rc = 1;
    }
    curl_global_cleanup();
    return rc;
}
