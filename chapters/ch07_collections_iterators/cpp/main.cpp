// Ch07 성적 데이터 분석 (C++20 참조 구현).
// C++20 ranges(views::filter/transform) 가 Rust 이터레이터 체인과 가장 비슷하지만,
// collect 에 해당하는 `ranges::to` 는 C++23 이라 여기서는 for 루프/push_back 으로 모읍니다.
#include <algorithm>
#include <cctype>
#include <format>
#include <fstream>
#include <functional>
#include <iostream>
#include <map>
#include <optional>
#include <ranges>
#include <set>
#include <sstream>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

struct Record {
    std::string student;
    std::string subject;
    int score;
};

std::string trim(const std::string& s) {
    auto b = s.find_first_not_of(" \t\r\n");
    if (b == std::string::npos) return "";
    auto e = s.find_last_not_of(" \t\r\n");
    return s.substr(b, e - b + 1);
}

std::optional<Record> parse_record(const std::string& line) {
    std::vector<std::string> parts;
    std::stringstream ss(line);
    std::string p;
    while (std::getline(ss, p, ',')) parts.push_back(trim(p));
    if (!line.empty() && line.back() == ',') parts.push_back("");  // getline 은 끝의 빈 필드를 놓칩니다
    if (parts.size() != 3) return std::nullopt;
    const std::string& score = parts[2];
    bool digits = !score.empty() && std::all_of(score.begin(), score.end(), [](unsigned char c) { return std::isdigit(c); });
    if (parts[0].empty() || parts[1].empty() || !digits) return std::nullopt;
    return Record{parts[0], parts[1], std::stoi(score)};
}

std::pair<std::vector<Record>, int> parse_records(const std::string& text) {
    std::vector<std::string> lines;
    std::istringstream in(text);
    std::string l;
    bool first = true;
    while (std::getline(in, l)) {
        if (first) { first = false; continue; }  // 헤더 건너뛰기
        if (!trim(l).empty()) lines.push_back(l);
    }
    std::vector<Record> records;
    for (const auto& line : lines)
        if (auto r = parse_record(line)) records.push_back(*r);
    return {records, static_cast<int>(lines.size() - records.size())};
}

std::map<std::string, double> student_averages(const std::vector<Record>& records) {
    std::map<std::string, std::vector<int>> scores;  // std::map 은 키 순서대로 정렬됩니다
    for (const auto& r : records) scores[r.student].push_back(r.score);  // operator[] 가 Rust 의 entry().or_default()
    std::map<std::string, double> out;
    for (const auto& [name, v] : scores) {
        int sum = 0;
        for (int s : v) sum += s;
        out[name] = static_cast<double>(sum) / static_cast<double>(v.size());
    }
    return out;
}

std::map<std::string, std::pair<std::string, int>> subject_tops(const std::vector<Record>& records) {
    std::map<std::string, std::pair<std::string, int>> tops;
    for (const auto& r : records) {
        auto it = tops.find(r.subject);
        if (it == tops.end()) tops[r.subject] = {r.student, r.score};
        else if (r.score > it->second.second) it->second = {r.student, r.score};
    }
    return tops;
}

char grade_of(int score) {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
}

std::unordered_map<char, int> grade_histogram(const std::vector<int>& scores) {
    std::unordered_map<char, int> hist;
    for (int s : scores) ++hist[grade_of(s)];
    return hist;
}

// 람다 [bonus] : 값 캡처. Rust 의 `move |s| ...` 와 같은 효과입니다.
// ([&bonus] 로 참조 캡처하면 함수가 끝난 뒤 dangling — 컴파일러가 막아 주지 않습니다!)
std::function<int(int)> make_curve(int bonus) {
    return [bonus](int s) { return std::min(s + bonus, 100); };
}

void apply_in_place(std::vector<int>& scores, const std::function<int(int)>& f) {
    for (int& s : scores) s = f(s);
}

std::pair<int, int> count_passing(const std::vector<int>& scores) {
    int calls = 0;
    auto passes = [&calls](int s) { ++calls; return s >= 60; };  // 참조 캡처로 calls 를 수정 -> Rust 의 FnMut
    int passing = static_cast<int>(std::ranges::count_if(scores, passes));
    return {passing, calls};
}

std::vector<std::pair<std::string, double>> ranking(const std::map<std::string, double>& averages) {
    std::vector<std::pair<std::string, double>> v(averages.begin(), averages.end());
    std::sort(v.begin(), v.end(), [](const auto& a, const auto& b) {
        if (a.second != b.second) return a.second > b.second;
        return a.first < b.first;
    });
    return v;
}

std::vector<std::string> distinct_subjects(const std::vector<Record>& records) {
    std::set<std::string> s;  // std::set 은 정렬 + 중복 제거
    for (const auto& r : records) s.insert(r.subject);
    return {s.begin(), s.end()};
}

void print_histogram(const std::string& title, const std::vector<int>& scores) {
    std::cout << title << "\n";
    auto hist = grade_histogram(scores);
    for (char g : std::string("ABCDF")) {
        int n = hist.count(g) ? hist[g] : 0;
        std::cout << std::format("{}: {} ({})\n", g, std::string(static_cast<size_t>(n), '#'), n);
    }
}

int main() {
    std::ifstream f("data/scores.csv");
    std::stringstream buf;
    buf << f.rdbuf();
    auto [records, skipped] = parse_records(buf.str());
    std::cout << std::format("records: {} (skipped {})\n", records.size(), skipped);
    std::string joined;
    for (const auto& s : distinct_subjects(records)) joined += (joined.empty() ? "" : ", ") + s;
    std::cout << "subjects: " << joined << "\n";

    std::cout << "== average per student ==\n";
    auto averages = student_averages(records);
    for (const auto& [name, avg] : averages) std::cout << std::format("{:<6}: {:.2f}\n", name, avg);

    std::cout << "== top per subject ==\n";
    for (const auto& [subject, best] : subject_tops(records))
        std::cout << std::format("{:<8}: {} ({})\n", subject, best.first, best.second);

    // views::transform 은 Rust 의 .iter().map(...) 과 같은 "게으른" 뷰입니다. 벡터로 옮기는 단계가 collect().
    auto score_view = records | std::views::transform([](const Record& r) { return r.score; });
    std::vector<int> scores(score_view.begin(), score_view.end());
    print_histogram("== grade histogram ==", scores);

    auto curve = make_curve(5);
    apply_in_place(scores, curve);
    print_histogram("== after +5 curve ==", scores);

    std::vector<int> raw(score_view.begin(), score_view.end());
    auto [passing, calls] = count_passing(raw);
    std::cout << "== passing ==\n";
    std::cout << std::format("passing: {} of {} (predicate called {} times)\n", passing, raw.size(), calls);

    std::cout << "== ranking ==\n";
    int i = 0;
    for (const auto& [name, avg] : ranking(averages)) std::cout << std::format("{}. {:<6} {:.2f}\n", ++i, name, avg);
}
