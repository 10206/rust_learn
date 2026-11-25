// Ch15. Serde 와 JSON - C++20 참조 구현 (nlohmann::ordered_json).
//
// nlohmann::json 은 기본적으로 키를 알파벳 순으로 정렬합니다. Rust/Python 과 키 순서를 맞추려고
// "넣은 순서"를 유지하는 ordered_json 을 씁니다.
// Rust 의 Serde 는 derive 한 줄로 만들어 주는 변환 코드를, C++ 에서는 직접 써야 합니다.
#include <nlohmann/json.hpp>

#include <algorithm>
#include <cstddef>
#include <format>
#include <fstream>
#include <iostream>
#include <map>
#include <optional>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <variant>
#include <vector>

using json = nlohmann::ordered_json;

// ── 타입 정의 ─────────────────────────────────────────────
struct TextBlock {
    std::string text;
};
struct ThinkingBlock {
    std::string thinking;
    std::string signature;
};
struct ToolUseBlock {
    std::string id;
    std::string name;
    json input;
};
struct ToolResultBlock {
    std::string tool_use_id;
    std::string content;
    std::optional<bool> is_error;
};
// Unknown 블록 = 원본 json 그대로 (Rust: ContentBlock::Unknown(Value))
using ContentBlock = std::variant<TextBlock, ThinkingBlock, ToolUseBlock, ToolResultBlock, json>;

struct Message {
    std::string role;                                     // "user" | "assistant"
    std::variant<std::string, std::vector<ContentBlock>> content;  // Rust: untagged enum MessageContent
};

struct Tool {
    std::string name;
    std::string description;
    json input_schema;
};

struct Request {
    std::string model;
    int max_tokens;
    std::optional<std::string> system;
    std::vector<Tool> tools;
    std::vector<Message> messages;
    std::vector<std::string> stop_sequences;
};

struct Summary {
    std::size_t message_count;
    std::size_t tool_calls;
    std::size_t tool_errors;
    std::map<std::string, std::size_t> tools_used;
    std::vector<std::string> unknown_blocks;
};

// 데이터가 기대한 모양이 아닐 때 던지는 예외 (Rust: serde_json::error::Category::Data)
struct DataError : std::runtime_error {
    using std::runtime_error::runtime_error;
};

// ── JSON -> 타입 (역직렬화) ────────────────────────────────

// 키가 있고 타입이 맞는지 검사해서 돌려줍니다. (Serde 가 자동으로 해 주는 일)
const json& need(const json& obj, const char* key) {
    if (!obj.is_object() || !obj.contains(key)) throw DataError(std::string("missing field `") + key + "`");
    return obj.at(key);
}
std::string need_string(const json& obj, const char* key) {
    const json& v = need(obj, key);
    if (!v.is_string()) throw DataError(std::string("invalid type for `") + key + "`");
    return v.get<std::string>();
}

// TODO(3): 아는 블록이면 타입으로, 아니면(또는 모양이 안 맞으면) 원본 그대로 보관.
ContentBlock block_from_json(const json& raw) {
    try {
        if (raw.is_object() && raw.contains("type") && raw.at("type").is_string()) {
            const std::string kind = raw.at("type").get<std::string>();
            if (kind == "text") return TextBlock{need_string(raw, "text")};
            if (kind == "thinking") return ThinkingBlock{need_string(raw, "thinking"), need_string(raw, "signature")};
            if (kind == "tool_use") return ToolUseBlock{need_string(raw, "id"), need_string(raw, "name"), need(raw, "input")};
            if (kind == "tool_result") {
                std::optional<bool> is_error;
                if (raw.contains("is_error") && !raw.at("is_error").is_null()) {
                    if (!raw.at("is_error").is_boolean()) throw DataError("invalid type for `is_error`");
                    is_error = raw.at("is_error").get<bool>();
                }
                return ToolResultBlock{need_string(raw, "tool_use_id"), need_string(raw, "content"), is_error};
            }
        }
    } catch (const DataError&) {
        // 모양이 안 맞으면 Unknown 으로 폴백 (Rust: untagged 가 다음 variant 를 시도)
    }
    return raw;
}

Message message_from_json(const json& raw) {
    std::string role = need_string(raw, "role");
    if (role != "user" && role != "assistant") throw DataError("unknown variant `" + role + "`");
    const json& content = need(raw, "content");
    if (content.is_string()) return Message{role, content.get<std::string>()};
    if (content.is_array()) {
        std::vector<ContentBlock> blocks;
        for (const auto& b : content) blocks.push_back(block_from_json(b));
        return Message{role, std::move(blocks)};
    }
    throw DataError("data did not match any variant of untagged enum MessageContent");
}

Request request_from_json(const json& raw) {
    Request req;
    req.model = need_string(raw, "model");
    const json& mt = need(raw, "max_tokens");
    if (!mt.is_number_integer()) throw DataError("invalid type for `max_tokens`");
    req.max_tokens = mt.get<int>();
    if (raw.contains("system") && !raw.at("system").is_null()) req.system = need_string(raw, "system");
    for (const auto& t : need(raw, "tools"))
        req.tools.push_back(Tool{need_string(t, "name"), need_string(t, "description"), need(t, "input_schema")});
    for (const auto& m : need(raw, "messages")) req.messages.push_back(message_from_json(m));
    if (raw.contains("stop_sequences")) req.stop_sequences = raw.at("stop_sequences").get<std::vector<std::string>>();
    return req;
}

// TODO(5)
Request parse_request(const std::string& text) { return request_from_json(json::parse(text)); }
Message parse_message(const std::string& text) { return message_from_json(json::parse(text)); }

// ── 타입 -> JSON (직렬화) ──────────────────────────────────

json block_to_json(const ContentBlock& block) {
    // ordered_json 은 넣은 순서를 지키므로 Rust 의 필드 선언 순서와 똑같이 넣어 줍니다.
    return std::visit(
        [](const auto& b) -> json {
            using T = std::decay_t<decltype(b)>;
            if constexpr (std::is_same_v<T, TextBlock>) {
                return json{{"type", "text"}, {"text", b.text}};
            } else if constexpr (std::is_same_v<T, ThinkingBlock>) {
                return json{{"type", "thinking"}, {"thinking", b.thinking}, {"signature", b.signature}};
            } else if constexpr (std::is_same_v<T, ToolUseBlock>) {
                return json{{"type", "tool_use"}, {"id", b.id}, {"name", b.name}, {"input", b.input}};
            } else if constexpr (std::is_same_v<T, ToolResultBlock>) {
                json out{{"type", "tool_result"}, {"tool_use_id", b.tool_use_id}, {"content", b.content}};
                if (b.is_error) out["is_error"] = *b.is_error;  // Rust: skip_serializing_if = "Option::is_none"
                return out;
            } else {
                return b;  // Unknown: 그대로
            }
        },
        block);
}

json message_to_json(const Message& msg) {
    json out;
    out["role"] = msg.role;
    if (const auto* s = std::get_if<std::string>(&msg.content)) {
        out["content"] = *s;
    } else {
        json arr = json::array();
        for (const auto& b : std::get<std::vector<ContentBlock>>(msg.content)) arr.push_back(block_to_json(b));
        out["content"] = std::move(arr);
    }
    return out;
}

json request_to_json(const Request& req) {
    json out;
    out["model"] = req.model;
    out["max_tokens"] = req.max_tokens;
    if (req.system) out["system"] = *req.system;
    out["tools"] = json::array();
    for (const auto& t : req.tools)
        out["tools"].push_back(json{{"name", t.name}, {"description", t.description}, {"input_schema", t.input_schema}});
    out["messages"] = json::array();
    for (const auto& m : req.messages) out["messages"].push_back(message_to_json(m));
    if (!req.stop_sequences.empty()) out["stop_sequences"] = req.stop_sequences;  // Rust: skip_serializing_if = "Vec::is_empty"
    return out;
}

json summary_to_json(const Summary& s) {
    // Rust: #[serde(rename_all = "camelCase")]
    json used = json::object();
    for (const auto& [k, v] : s.tools_used) used[k] = v;
    json unknown = json::array();
    for (const auto& u : s.unknown_blocks) unknown.push_back(u);
    json out;
    out["messageCount"] = s.message_count;
    out["toolCalls"] = s.tool_calls;
    out["toolErrors"] = s.tool_errors;
    out["toolsUsed"] = std::move(used);
    out["unknownBlocks"] = std::move(unknown);
    return out;
}

// dump(2): 2칸 들여쓰기. ordered_json 은 기본이 UTF-8 그대로 출력(ensure_ascii=false)입니다.
std::string to_pretty(const json& j) { return j.dump(2); }
std::string to_compact(const json& j) { return j.dump(); }

// ── 분석 ───────────────────────────────────────────────────

// TODO(6): 블록의 type 이름. 모르는 블록은 원본의 "type" 값.
std::string block_kind(const ContentBlock& block) {
    return std::visit(
        [](const auto& b) -> std::string {
            using T = std::decay_t<decltype(b)>;
            if constexpr (std::is_same_v<T, TextBlock>) return "text";
            else if constexpr (std::is_same_v<T, ThinkingBlock>) return "thinking";
            else if constexpr (std::is_same_v<T, ToolUseBlock>) return "tool_use";
            else if constexpr (std::is_same_v<T, ToolResultBlock>) return "tool_result";
            else return (b.is_object() && b.contains("type") && b.at("type").is_string()) ? b.at("type").template get<std::string>() : "?";
        },
        block);
}

// 문자열 content 에는 블록이 없으므로 빈 벡터를 돌려줍니다.
const std::vector<ContentBlock>& blocks_of(const Message& msg) {
    static const std::vector<ContentBlock> empty;
    const auto* blocks = std::get_if<std::vector<ContentBlock>>(&msg.content);
    return blocks ? *blocks : empty;
}

// 문자열 content 는 text 블록 하나로 칩니다.
std::vector<std::string> message_kinds(const Message& msg) {
    if (std::holds_alternative<std::string>(msg.content)) return {"text"};
    std::vector<std::string> kinds;
    for (const auto& b : blocks_of(msg)) kinds.push_back(block_kind(b));
    return kinds;
}

std::map<std::string, std::size_t> count_blocks(const Request& req) {
    std::map<std::string, std::size_t> counts;  // std::map 은 키 정렬 (Rust 의 BTreeMap)
    for (const auto& m : req.messages)
        for (const auto& k : message_kinds(m)) ++counts[k];
    return counts;
}

// TODO(7)
std::map<std::string, std::size_t> tools_used(const Request& req) {
    std::map<std::string, std::size_t> counts;
    for (const auto& m : req.messages)
        for (const auto& b : blocks_of(m))
            if (const auto* use = std::get_if<ToolUseBlock>(&b)) ++counts[use->name];
    return counts;
}

// TODO(8): tool_use 와 tool_result 를 id 로 짝지어 한 줄씩.
std::vector<std::string> tool_call_report(const Request& req) {
    std::map<std::string, const ToolUseBlock*> calls;
    std::vector<std::string> lines;
    for (const auto& m : req.messages) {
        for (const auto& b : blocks_of(m)) {
            if (const auto* use = std::get_if<ToolUseBlock>(&b)) {
                calls[use->id] = use;
            } else if (const auto* res = std::get_if<ToolResultBlock>(&b)) {
                auto it = calls.find(res->tool_use_id);
                std::string head = it != calls.end() ? it->second->name + " " + to_compact(it->second->input)
                                                     : "? " + res->tool_use_id;
                std::string tail = res->is_error.value_or(false) ? "ERROR: " + res->content : res->content;
                lines.push_back(head + " -> " + tail);
            }
        }
    }
    return lines;
}

// TODO(9): 처음 3개 메시지 + 요약 요청 text 블록을 담은 새 요청.
Request build_followup(const Request& req) {
    Request out;
    out.model = req.model;
    out.max_tokens = 256;
    out.system = "Answer in one short sentence.";
    out.tools = req.tools;
    out.messages.assign(req.messages.begin(), req.messages.begin() + 3);  // 복사 (Rust: .cloned())
    if (auto* blocks = std::get_if<std::vector<ContentBlock>>(&out.messages.back().content)) {
        blocks->push_back(TextBlock{"Please summarize the results above in one sentence."});
    }
    return out;
}

// TODO(10)
Summary summarize(const Request& req) {
    Summary s;
    s.message_count = req.messages.size();
    s.tools_used = tools_used(req);
    s.tool_calls = 0;
    for (const auto& [name, n] : s.tools_used) s.tool_calls += n;
    s.tool_errors = 0;
    std::set<std::string> unknown;  // 정렬 + 중복 제거
    for (const auto& m : req.messages) {
        for (const auto& b : blocks_of(m)) {
            if (const auto* res = std::get_if<ToolResultBlock>(&b)) {
                if (res->is_error.value_or(false)) ++s.tool_errors;
            } else if (std::holds_alternative<json>(b)) {
                unknown.insert(block_kind(b));
            }
        }
    }
    s.unknown_blocks.assign(unknown.begin(), unknown.end());
    return s;
}

// TODO(11): 메시지 JSON 하나를 파싱해 "ok" / "syntax" / "data" 로 분류.
std::string classify_error(const std::string& text) {
    try {
        parse_message(text);
        return "ok";
    } catch (const json::parse_error&) {
        return "syntax";
    } catch (const DataError&) {
        return "data";
    }
}

// TODO(12): 타입으로 읽었다가 다시 JSON 으로 써도 원본과 똑같은가?
bool round_trip_lossless(const std::string& text) {
    try {
        return request_to_json(parse_request(text)) == json::parse(text);
    } catch (const std::exception&) {
        return false;
    }
}

int main() {
    std::ifstream file("data/conversation.json");
    std::stringstream buf;
    buf << file.rdbuf();
    const std::string text = buf.str();
    const Request req = parse_request(text);

    std::cout << "== request ==\n";
    std::cout << "model: " << req.model << "\n";
    std::cout << "max_tokens: " << req.max_tokens << "\n";
    std::string names;
    for (const auto& t : req.tools) names += (names.empty() ? "" : ", ") + t.name;
    std::cout << "tools: " << names << "\n";
    std::cout << "messages: " << req.messages.size() << "\n";

    std::cout << "== messages ==\n";
    for (std::size_t i = 0; i < req.messages.size(); ++i) {
        std::string kinds;
        for (const auto& k : message_kinds(req.messages[i])) kinds += (kinds.empty() ? "" : ", ") + k;
        std::cout << std::format("[{}] {}: {}\n", i, req.messages[i].role, kinds);
    }

    std::cout << "== block counts ==\n";
    for (const auto& [kind, n] : count_blocks(req)) std::cout << kind << ": " << n << "\n";

    std::cout << "== tools used ==\n";
    for (const auto& [name, n] : tools_used(req)) std::cout << name << ": " << n << "\n";

    std::cout << "== tool calls ==\n";
    for (const auto& line : tool_call_report(req)) std::cout << line << "\n";

    std::cout << "== bad input ==\n";
    const std::vector<std::string> bad = {
        R"({"role": "user", "content": )",
        R"({"role": "robot", "content": "hi"})",
        R"({"role": "user"})",
        R"({"role": "user", "content": [{"type": "text"}]})",
        R"({"role": "user", "content": 42})",
    };
    for (std::size_t i = 0; i < bad.size(); ++i) std::cout << (i + 1) << ": " << classify_error(bad[i]) << "\n";

    std::cout << "== round trip ==\n";
    std::cout << "lossless: " << (round_trip_lossless(text) ? "yes" : "no") << "\n";

    std::cout << "== follow-up request ==\n";
    std::cout << to_pretty(request_to_json(build_followup(req))) << "\n";

    std::cout << "== summary (original) ==\n";
    std::cout << to_pretty(summary_to_json(summarize(req))) << "\n";

    std::cout << "== summary (follow-up) ==\n";
    std::cout << to_pretty(summary_to_json(summarize(build_followup(req)))) << "\n";
    return 0;
}
