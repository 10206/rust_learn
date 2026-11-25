// Ch10 C++: parser (Rust: parser.rs)
#pragma once
#include <string>
#include <utility>
#include <vector>

#include "model.hpp"

namespace ledger {

// Rust 는 enum 4 변형 + Display. C++ 는 kind + 필드로 단순화했어요.
struct ParseError {
    enum class Kind { FieldCount, BadDate, UnknownCategory, BadAmount } kind;
    int line;
    std::string text;
    int found = 0;

    std::string to_string() const;  // Rust: impl Display
};

// 실패하면 ParseError 를 optional 에 담아 돌려줍니다 (예외 대신 Rust 의 Result 처럼).
struct ParseResult {
    std::optional<Transaction> ok;
    std::optional<ParseError> err;
};

ParseResult parse_line(int line_no, const std::string& line);
std::pair<std::vector<Transaction>, std::vector<ParseError>> parse_ledger(const std::string& text);

}  // namespace ledger
