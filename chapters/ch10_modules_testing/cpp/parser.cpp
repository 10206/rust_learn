#include "parser.hpp"

#include <cctype>

namespace ledger {

namespace {  // 익명 namespace = 이 .cpp 안에서만 보이는 비공개 함수 (Rust 의 pub 없는 fn)

std::vector<std::string> split(const std::string& s, char sep) {
    std::vector<std::string> out;
    std::string cur;
    for (char c : s) {
        if (c == sep) {
            out.push_back(cur);
            cur.clear();
        } else {
            cur += c;
        }
    }
    out.push_back(cur);
    return out;
}

bool all_digits(const std::string& s) {
    if (s.empty()) return false;
    for (char c : s)
        if (!std::isdigit(static_cast<unsigned char>(c))) return false;
    return true;
}

struct Date {
    int y, m, d;
};

std::optional<Date> parse_date(const std::string& s) {
    auto p = split(s, '-');
    if (p.size() != 3 || p[0].size() != 4) return std::nullopt;
    for (const auto& x : p)
        if (!all_digits(x)) return std::nullopt;
    Date d{std::stoi(p[0]), std::stoi(p[1]), std::stoi(p[2])};
    if (d.m >= 1 && d.m <= 12 && d.d >= 1 && d.d <= 31) return d;
    return std::nullopt;
}

std::optional<std::int64_t> parse_int(const std::string& s) {
    std::string body = (!s.empty() && s[0] == '-') ? s.substr(1) : s;
    if (!all_digits(body)) return std::nullopt;
    return std::stoll(s);
}

}  // namespace

std::string ParseError::to_string() const {
    std::string prefix = "line " + std::to_string(line) + ": ";
    switch (kind) {
        case Kind::FieldCount: return prefix + "expected 4 fields, found " + std::to_string(found);
        case Kind::BadDate: return prefix + "invalid date '" + text + "'";
        case Kind::UnknownCategory: return prefix + "unknown category '" + text + "'";
        case Kind::BadAmount: return prefix + "invalid amount '" + text + "'";
    }
    return prefix;
}

ParseResult parse_line(int line_no, const std::string& line) {
    auto f = split(line, ',');
    auto fail = [&](ParseError::Kind k, std::string text, int found = 0) {
        return ParseResult{std::nullopt, ParseError{k, line_no, std::move(text), found}};
    };
    if (f.size() != 4) return fail(ParseError::Kind::FieldCount, "", static_cast<int>(f.size()));
    auto date = parse_date(f[0]);
    if (!date) return fail(ParseError::Kind::BadDate, f[0]);
    auto cat = parse_category(f[1]);
    if (!cat) return fail(ParseError::Kind::UnknownCategory, f[1]);
    auto amount = parse_int(f[2]);
    if (!amount) return fail(ParseError::Kind::BadAmount, f[2]);
    return ParseResult{Transaction{date->y, date->m, date->d, *cat, *amount, f[3]}, std::nullopt};
}

std::pair<std::vector<Transaction>, std::vector<ParseError>> parse_ledger(const std::string& text) {
    std::vector<Transaction> ok;
    std::vector<ParseError> errors;
    int line_no = 0;
    std::size_t pos = 0;
    while (pos < text.size()) {
        auto nl = text.find('\n', pos);
        std::string line = text.substr(pos, nl == std::string::npos ? std::string::npos : nl - pos);
        pos = nl == std::string::npos ? text.size() : nl + 1;
        ++line_no;
        if (line.find_first_not_of(" \t\r") == std::string::npos || line.rfind("date,", 0) == 0) continue;
        auto r = parse_line(line_no, line);
        if (r.ok) ok.push_back(*r.ok);
        else errors.push_back(*r.err);
    }
    return {ok, errors};
}

}  // namespace ledger
