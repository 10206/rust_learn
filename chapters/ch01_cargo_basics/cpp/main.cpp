// Ch01. 온도 변환표 - C++20 참조 구현.
// Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
// C++: 컴파일 -> 링크 -> 실행 파일. (g++ 가 컴파일러+링커 역할을 모두 호출합니다)
#include <algorithm>
#include <array>
#include <cstdio>
#include <format>
#include <iostream>
#include <numeric>
#include <string>
#include <tuple>

// Rust: format!("Hello, {}!", name)  -> 출력하지 않고 String 을 만들어 돌려줍니다.
std::string greeting(const std::string& name) {
    return std::format("Hello, {}!", name);
}

double c_to_f(double c) { return c * 9.0 / 5.0 + 32.0; }

double f_to_c(double f) { return (f - 32.0) * 5.0 / 9.0; }

std::string table_line(double c) {
    // 폭 7, 소수점 1자리, 오른쪽 정렬
    return std::format("{:>7.1f} {:>7.1f}", c, c_to_f(c));
}

void print_table() {
    std::cout << std::format("{:>7} {:>7}", "C", "F") << '\n';
    for (int c = -10; c <= 40; c += 10) {  // Rust: (-10..=40).step_by(10)
        std::cout << table_line(static_cast<double>(c)) << '\n';
    }
}

// Rust 는 튜플 (f64, f64, f64) 을 바로 반환합니다. C++ 는 std::tuple (또는 struct).
std::tuple<double, double, double> min_max_avg(const std::array<double, 5>& values) {
    auto [lo, hi] = std::minmax_element(values.begin(), values.end());
    double sum = std::accumulate(values.begin(), values.end(), 0.0);
    return {*lo, *hi, sum / static_cast<double>(values.size())};
}

int main() {
    std::cout << greeting("Cargo") << '\n';
    std::cout << "=== Celsius to Fahrenheit ===\n";
    print_table();

    std::cout << "=== round trip (F -> C) ===\n";
    for (double f : {14.0, 32.0, 98.6, 212.0}) {
        std::cout << std::format("{:>6.1f} F = {:>6.1f} C", f, f_to_c(f)) << '\n';
    }

    std::cout << "=== sample readings ===\n";
    const std::array<double, 5> readings{22.5, 18.0, 30.1, 15.7, 25.3};
    std::string joined;
    for (std::size_t i = 0; i < readings.size(); ++i) {
        if (i > 0) joined += ", ";
        joined += std::format("{:.1f}", readings[i]);
    }
    std::cout << "readings: " << joined << '\n';
    auto [lo, hi, avg] = min_max_avg(readings);
    std::cout << std::format("min: {:.1f}  max: {:.1f}  avg: {:.2f}", lo, hi, avg) << '\n';
    return 0;
}
