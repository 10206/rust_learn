// Ch02. 콜라츠 통계 - C++20 참조 구현.
// Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
//
// C++ 와 Rust 의 큰 차이 (이 파일을 읽으며 비교해 보세요)
//  - 부호 없는 정수(uint8_t 등)는 C++ 에서도 "감기는" 것이 정의된 동작입니다. 부호 있는 정수 오버플로는 UB.
//    Rust 는 debug 에서 둘 다 패닉, release 에서 둘 다 감깁니다 (둘 다 UB 는 아님).
//  - C++ 는 정수 <-> 실수 암묵 변환이 되지만, Rust 는 항상 `as` 로 명시해야 합니다.
#include <cmath>
#include <cstdint>
#include <format>
#include <iostream>
#include <optional>
#include <string>
#include <utility>
#include <vector>

// ---------- 콜라츠 기본 ----------
std::uint64_t collatz_next(std::uint64_t n) {
    // Rust 에서는 if/else 자체가 값을 가지는 식입니다. C++ 는 삼항 연산자를 씁니다.
    return n % 2 == 0 ? n / 2 : 3 * n + 1;
}

std::uint32_t collatz_steps(std::uint64_t n) {
    std::uint32_t steps = 0;
    while (n != 1) {
        n = collatz_next(n);
        ++steps;
    }
    return steps;
}

std::uint64_t collatz_peak(std::uint64_t n) {
    std::uint64_t peak = n;
    while (n != 1) {
        n = collatz_next(n);
        if (n > peak) peak = n;
    }
    return peak;
}

// Rust: match steps { 0 => ..., 1..=10 => ..., 11..=50 => ..., _ => ... }  (빠뜨리면 컴파일 에러)
const char* classify_steps(std::uint32_t steps) {
    if (steps == 0) return "start";
    if (steps <= 10) return "short";
    if (steps <= 50) return "medium";
    return "long";
}

std::pair<std::uint64_t, std::uint32_t> longest_chain(std::uint64_t limit) {
    std::pair<std::uint64_t, std::uint32_t> best{1, 0};
    for (std::uint64_t n = 1; n <= limit; ++n) {
        std::uint32_t s = collatz_steps(n);
        if (s > best.second) best = {n, s};  // 같은 길이면 먼저 나온 n 유지
    }
    return best;
}

std::pair<std::uint64_t, std::uint64_t> max_peak(std::uint64_t limit) {
    std::pair<std::uint64_t, std::uint64_t> best{1, 1};
    for (std::uint64_t n = 1; n <= limit; ++n) {
        std::uint64_t p = collatz_peak(n);
        if (p > best.second) best = {n, p};
    }
    return best;
}

// ---------- 오버플로 ----------
// Rust 는 u8::checked_mul / checked_add 가 표준으로 있습니다. C++ 는 직접 검사하거나 컴파일러 내장 함수가 필요해요.
// 여기서는 더 넓은 타입(uint64_t)으로 계산한 뒤 최댓값과 비교합니다.
std::optional<std::uint64_t> checked_next(std::uint64_t n, std::uint64_t max_value) {
    if (n % 2 == 0) return n / 2;
    std::uint64_t v = 3 * n + 1;
    if (v > max_value) return std::nullopt;
    return v;
}

std::optional<std::uint32_t> overflow_step(std::uint64_t start, std::uint64_t max_value) {
    std::uint64_t n = start;
    std::uint32_t steps = 0;
    while (n != 1) {
        auto nxt = checked_next(n, max_value);
        if (!nxt) return steps;
        n = *nxt;
        ++steps;
    }
    return std::nullopt;
}

std::optional<std::pair<std::uint64_t, std::uint64_t>> find_first_exceeding(std::uint64_t limit,
                                                                           std::uint64_t ceiling) {
    // C++ 에는 labeled break 가 없어서 goto, 플래그, 또는 함수 return 으로 탈출합니다.
    for (std::uint64_t n = 1; n <= limit; ++n) {
        std::uint64_t x = n;
        while (true) {
            if (x > ceiling) return std::make_pair(n, x);
            if (x == 1) break;
            x = collatz_next(x);
        }
    }
    return std::nullopt;
}

// ---------- as 변환 ----------
std::uint8_t to_u8(std::int32_t x) { return static_cast<std::uint8_t>(x); }       // 하위 8비트만 남음
std::int32_t trunc_to_i32(double x) { return static_cast<std::int32_t>(x); }       // 소수부 버림
// 주의: 범위를 벗어난 double -> int 변환은 C++ 에서 UB, Rust 의 `as` 는 최솟값/최댓값으로 "포화"합니다.

// ---------- 출력 ----------
std::string fmt_opt(std::optional<std::uint64_t> v) {
    return v ? std::format("Some({})", *v) : std::string("None");
}

void basics_demo() {
    std::cout << "== basics ==\n";
    int x = 5;
    std::cout << "shadowing: " << x;
    x = x * 2;  // C++ 의 안쪽 스코프 가림(shadowing)과 비슷하지만, Rust 는 같은 스코프에서도 새 변수를 만듭니다.
    std::cout << " -> " << x;
    std::string xs = std::to_string(x) + " apples";  // Rust 는 같은 이름 x 로 타입까지 바꿀 수 있음
    std::cout << " -> " << xs << '\n';

    auto point = std::make_pair(3, 4);
    auto [px, py] = point;
    std::cout << std::format("tuple: ({}, {}) -> x={} y={}\n", point.first, point.second, px, py);
    const int arr[3] = {10, 20, 30};
    std::cout << std::format("array: [{}, {}, {}] len={} sum={}\n", arr[0], arr[1], arr[2], 3,
                             arr[0] + arr[1] + arr[2]);

    // C++ 의 정수 나눗셈과 % 는 Rust 와 같습니다 (0 쪽으로 버림, 나머지 부호는 피제수 따름).
    int rem_euclid = ((-7 % 3) + 3) % 3;
    std::cout << std::format("div: 7/2={} -7/2={} -7%3={} rem_euclid={}\n", 7 / 2, -7 / 2, -7 % 3,
                             rem_euclid);
    std::cout << std::format("casts: 300 as u8={} -1 as u8={} 3.99 as i32={} -3.99 as i32={}\n",
                             static_cast<int>(to_u8(300)), static_cast<int>(to_u8(-1)),
                             trunc_to_i32(3.99), trunc_to_i32(-3.99));

    const char* label = collatz_steps(27) > 100 ? "big" : "small";
    int block = [] { int a = 3; return a * a; }();  // Rust: let block = { let a = 3; a * a };
    int counter = 0;
    int loop_value = 0;
    while (true) {  // Rust: let loop_value = loop { counter += 1; if counter == 5 { break counter * 2; } };
        ++counter;
        if (counter == 5) {
            loop_value = counter * 2;
            break;
        }
    }
    std::cout << std::format("exprs: if={} block={} loop={}\n", label, block, loop_value);
}

void overflow_demo() {
    std::cout << "== overflow: u8 add ==\n";
    for (auto [a, b] : {std::pair{250, 5}, std::pair{250, 10}}) {
        int total = a + b;
        std::optional<std::uint64_t> checked;
        if (total <= 255) checked = static_cast<std::uint64_t>(total);
        int wrapping = static_cast<std::uint8_t>(total);
        int saturating = total < 255 ? total : 255;
        std::cout << std::format("{} + {}: checked={} wrapping={} saturating={}\n", a, b, fmt_opt(checked),
                                 wrapping, saturating);
    }

    std::cout << "== overflow: collatz ==\n";
    int count = 0;
    int first_s = 0;
    std::uint32_t first_st = 0;
    for (std::uint64_t s = 1; s <= 255; ++s) {
        auto st = overflow_step(s, 255);
        if (st) {
            if (count == 0) {
                first_s = static_cast<int>(s);
                first_st = *st;
            }
            ++count;
        }
    }
    std::cout << std::format("u8 starts 1..=255 that overflow: {} (first: {} at step {})\n", count, first_s,
                             first_st);
    for (std::uint64_t start : {27ULL, 113383ULL, 159487ULL}) {
        auto step = overflow_step(start, 4294967295ULL);
        if (!step) {
            std::cout << std::format("u32 start={}: no overflow\n", start);
        } else {
            std::cout << std::format("u32 start={}: overflow at step {}\n", start, *step);
        }
    }
}

int main() {
    basics_demo();

    std::cout << "== collatz 1..=30 ==\n";
    std::cout << std::format("{:>3} {:>5} {:>6} kind\n", "n", "steps", "peak");
    for (std::uint64_t n = 1; n <= 30; ++n) {
        std::uint32_t s = collatz_steps(n);
        std::cout << std::format("{:>3} {:>5} {:>6} {}\n", n, s, collatz_peak(n), classify_steps(s));
    }
    auto [ln, ls] = longest_chain(30);
    std::cout << std::format("longest chain: n={} steps={}\n", ln, ls);
    auto [pn, pp] = max_peak(30);
    std::cout << std::format("highest peak: n={} peak={}\n", pn, pp);

    overflow_demo();

    std::cout << "== labeled break ==\n";
    for (auto [limit, ceiling] : {std::pair<std::uint64_t, std::uint64_t>{100, 1000}, {30, 10000}}) {
        auto found = find_first_exceeding(limit, ceiling);
        if (!found) {
            std::cout << std::format("first n<={} whose chain exceeds {}: none\n", limit, ceiling);
        } else {
            std::cout << std::format("first n<={} whose chain exceeds {}: n={} value={}\n", limit, ceiling,
                                     found->first, found->second);
        }
    }
    return 0;
}
