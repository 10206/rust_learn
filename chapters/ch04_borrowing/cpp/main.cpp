// Ch04. 텍스트 분석기 - C++20 참조 구현.
// Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
//
// 비교 포인트
//  - std::string_view / std::span 은 Rust 의 &str / &[T] 와 같은 "뷰"(포인터 + 길이)입니다.
//    하지만 C++ 은 뷰가 원본보다 오래 살아도 컴파일러가 막지 않아요 (dangling, UB).
//    아래 dangling_example() 주석을 보세요. Rust 는 이런 코드를 컴파일 에러로 막습니다.
//  - const T& / T& 는 Rust 의 &T / &mut T 와 비슷하지만, C++ 은 같은 객체에 T& 를 여러 개 만들 수 있어요.
//    Rust 는 "&mut 하나 XOR & 여러 개" 를 컴파일 타임에 강제합니다.
#include <algorithm>
#include <cctype>
#include <cstdint>
#include <format>
#include <iostream>
#include <span>
#include <string>
#include <string_view>
#include <vector>

const std::string TEXT =
    "The quick brown fox jumps over the lazy dog. It was a sunny day! Did you enjoy the afternoon?";
const std::string KOREAN = "안녕하세요 러스트";

// 공백(ASCII)으로 나눠서 string_view 목록으로 돌려줍니다. (Rust: split_whitespace)
std::vector<std::string_view> split_whitespace(std::string_view text) {
    std::vector<std::string_view> out;
    std::size_t i = 0;
    while (i < text.size()) {
        while (i < text.size() && std::isspace(static_cast<unsigned char>(text[i]))) ++i;
        std::size_t start = i;
        while (i < text.size() && !std::isspace(static_cast<unsigned char>(text[i]))) ++i;
        if (i > start) out.push_back(text.substr(start, i - start));
    }
    return out;
}

std::size_t word_count(std::string_view text) { return split_whitespace(text).size(); }

// 반환값 string_view 는 text 의 일부를 가리킵니다 (복사 없음). text 가 먼저 사라지면 dangling!
std::string_view longest_word(std::string_view text) {
    std::string_view best;
    for (std::string_view w : split_whitespace(text)) {
        while (!w.empty() && std::ispunct(static_cast<unsigned char>(w.front()))) w.remove_prefix(1);
        while (!w.empty() && std::ispunct(static_cast<unsigned char>(w.back()))) w.remove_suffix(1);
        if (w.size() > best.size()) best = w;  // 엄격히 클 때만 갱신 -> 동률이면 먼저 나온 단어
    }
    return best;
}

std::string_view first_sentence(std::string_view text) {
    std::size_t pos = text.find_first_of(".!?");
    if (pos == std::string_view::npos) return text;
    return text.substr(0, pos + 1);
}

// std::string& 로 받아 그 자리에서 고칩니다. Rust 의 &mut String 과 같은 역할이지만
// C++ 은 호출부(normalize(s))만 봐서는 s 가 바뀐다는 걸 알 수 없어요. Rust 는 normalize(&mut s) 로 드러납니다.
void normalize(std::string& s) {
    std::string out;
    for (std::string_view w : split_whitespace(s)) {
        if (!out.empty()) out += ' ';
        out += w;
    }
    for (char& c : out) c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
    s = std::move(out);
}

void scale_in_place(std::span<int> nums, int k) {  // Rust: &mut [i32]
    for (int& n : nums) n *= k;
}

int sum_slice(std::span<const int> nums) {  // Rust: &[i32]
    int total = 0;
    for (int n : nums) total += n;
    return total;
}

// ---------- UTF-8 ----------
// std::string 의 size() 는 Rust 의 str::len() 처럼 "바이트 수" 입니다. 글자 수는 직접 세어야 해요.
// UTF-8 에서 0b10xxxxxx 로 시작하는 바이트는 "이어지는 바이트(continuation)" 라서 글자의 시작이 아닙니다.
bool is_char_start(char c) { return (static_cast<unsigned char>(c) & 0xC0) != 0x80; }

std::size_t char_count(std::string_view s) {
    return static_cast<std::size_t>(std::count_if(s.begin(), s.end(), is_char_start));
}

// 앞의 n 글자. n 이 글자 수보다 크면 전체. (Rust: char_indices().nth(n) 으로 경계를 찾음)
std::string_view first_n_chars(std::string_view s, std::size_t n) {
    std::size_t chars = 0;
    for (std::size_t i = 0; i < s.size(); ++i) {
        if (is_char_start(s[i])) {
            if (chars == n) return s.substr(0, i);
            ++chars;
        }
    }
    return s;
}

std::string reverse_chars(std::string_view s) {
    std::vector<std::string_view> chars;
    for (std::size_t i = 0; i < s.size();) {
        std::size_t j = i + 1;
        while (j < s.size() && !is_char_start(s[j])) ++j;
        chars.push_back(s.substr(i, j - i));
        i = j;
    }
    std::string out;
    for (auto it = chars.rbegin(); it != chars.rend(); ++it) out += *it;
    return out;
}

std::string hex_bytes(std::string_view s) {
    std::string out;
    for (std::size_t i = 0; i < s.size(); ++i) {
        if (i > 0) out += ' ';
        out += std::format("{:02x}", static_cast<unsigned>(static_cast<unsigned char>(s[i])));
    }
    return out;
}

// ⚠️ 아래 함수는 (일부러) 주석 처리했어요. C++ 에서 dangling 이 얼마나 조용히 컴파일되는지 보여 주는 예시입니다.
//
// std::string_view dangling_example() {
//     std::string s = "temporary string that is long enough to live on the heap";
//     return s;   // string -> string_view 암묵 변환. view 는 지역 변수 s 의 버퍼를 가리킨다.
// }               // 함수가 끝나면 s 가 소멸 -> 호출한 쪽이 view 를 읽으면 UB (해제된 메모리 읽기)
//
// g++ 13 `-Wall -Wextra` 는 이 코드를 경고 없이 컴파일하고, 실행하면 쓰레기 값이 출력됩니다 (직접 확인했어요).
// Rust 에서 같은 코드는 "E0515: cannot return value referencing local variable" 컴파일 에러입니다.

std::string join(const std::vector<int>& v) {
    std::string out = "[";
    for (std::size_t i = 0; i < v.size(); ++i) {
        if (i > 0) out += ", ";
        out += std::to_string(v[i]);
    }
    return out + "]";
}

int main() {
    std::cout << "== 1. text stats ==\n";
    std::cout << "text: " << TEXT << '\n';
    std::cout << "words: " << word_count(TEXT) << '\n';
    std::cout << "longest: " << longest_word(TEXT) << '\n';
    std::cout << "longest (tie): " << longest_word("one two six") << '\n';
    std::cout << "first sentence: " << first_sentence(TEXT) << '\n';
    std::cout << "first sentence (no punctuation): " << first_sentence("hello there") << '\n';

    std::cout << "== 2. normalize in place ==\n";
    std::string s = "  Hello,   WORLD   from  Rust!  ";
    std::cout << "before: \"" << s << "\"\n";
    normalize(s);
    std::cout << "after: \"" << s << "\"\n";

    std::cout << "== 3. slices of numbers ==\n";
    std::vector<int> nums{1, 2, 3, 4, 5};
    std::cout << "nums: " << join(nums) << '\n';
    std::cout << "sum: " << sum_slice(nums) << '\n';
    scale_in_place(nums, 3);
    std::cout << "after scale x3: " << join(nums) << '\n';
    std::cout << "first three sum: " << sum_slice(std::span<const int>(nums).first(3)) << '\n';
    std::cout << "last two sum: " << sum_slice(std::span<const int>(nums).last(2)) << '\n';

    std::cout << "== 4. UTF-8 ==\n";
    std::cout << "text: " << KOREAN << '\n';
    std::cout << "words: " << word_count(KOREAN) << '\n';
    std::cout << "bytes: " << KOREAN.size() << '\n';
    std::cout << "chars: " << char_count(KOREAN) << '\n';
    std::cout << "first 3 chars: " << first_n_chars(KOREAN, 3) << '\n';
    std::cout << "first byte-slice [0..3]: " << std::string_view(KOREAN).substr(0, 3) << '\n';
    std::cout << "reversed: " << reverse_chars(KOREAN) << '\n';
    std::cout << "bytes of 가: " << hex_bytes("가") << '\n';
    return 0;
}
