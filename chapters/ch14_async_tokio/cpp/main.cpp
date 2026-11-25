// Ch14. async/await 와 Tokio - C++20 참조 구현.
//
// C++20 코루틴(co_await)은 언어 "문법"만 제공하고, 실행기(executor)나 Task 타입은
// 표준 라이브러리에 없어서 직접 만들거나 외부 라이브러리(cppcoro, Boost.Asio 등)가 필요합니다.
// 그래서 여기서는 std::async / std::future / 스레드로 "동시성"을 흉내 냅니다.
// (Rust 의 Tokio 는 스레드 몇 개로 수천 개의 Future 를 돌리지만,
//  std::async(launch::async) 는 호출마다 스레드를 하나씩 씁니다.)
#include <chrono>
#include <cstdint>
#include <format>
#include <future>
#include <iostream>
#include <string>
#include <thread>
#include <utility>
#include <vector>
#include <functional>

using namespace std::chrono_literals;
using Clock = std::chrono::steady_clock;

struct Call {
    std::string prompt;
    int latency_ms;
};

// Rust 의 Result<String, CallError> 대신 쓰는 간단한 결과 타입 (std::expected 는 C++23).
struct Outcome {
    bool ok;
    std::string text;  // ok 이면 답, 아니면 에러 메시지
};

// TODO(1): 가짜 LLM 호출. 스레드를 잠재웁니다 (std::this_thread::sleep_for).
std::string fake_llm_call(const Call& call) {
    std::this_thread::sleep_for(std::chrono::milliseconds(call.latency_ms));
    return std::format("answer to '{}'", call.prompt);
}

// TODO(2): 순차 실행. 총 시간 = 지연 시간의 합.
std::vector<std::string> run_sequential(const std::vector<Call>& calls) {
    std::vector<std::string> results;
    for (const auto& call : calls) results.push_back(fake_llm_call(call));
    return results;
}

// TODO(3): 동시 실행. 호출마다 스레드를 띄우고, 입력 순서대로 get() 합니다.
std::vector<std::string> run_concurrent(const std::vector<Call>& calls) {
    std::vector<std::future<std::string>> futures;
    for (const auto& call : calls) {
        // Rust 의 tokio::spawn 은 'static + Send 를 컴파일러가 검사하지만,
        // C++ 은 call 을 참조로 넘겨도 컴파일됩니다 (수명 문제는 개발자 책임).
        futures.push_back(std::async(std::launch::async, fake_llm_call, call));
    }
    std::vector<std::string> results;
    for (auto& f : futures) results.push_back(f.get());
    return results;
}

// TODO(4): 두 호출을 동시에 (Rust: tokio::join!).
std::pair<std::string, std::string> join_pair(const Call& a, const Call& b) {
    auto fa = std::async(std::launch::async, fake_llm_call, a);
    auto fb = std::async(std::launch::async, fake_llm_call, b);
    return {fa.get(), fb.get()};
}

// TODO(5): 시간 제한. wait_for 로 기다려 보고, 안 끝났으면 timeout.
// 주의: std::async 가 돌려준 future 의 소멸자는 작업이 끝날 때까지 "블록"합니다.
//       즉 C++ 에서는 취소가 없어서 함수가 반환되는 시점이 결국 500ms 뒤가 됩니다.
//       Rust 는 Future 를 drop 하면 그 자리에서 즉시 취소됩니다.
Outcome call_with_timeout(const Call& call, int limit_ms) {
    auto fut = std::async(std::launch::async, fake_llm_call, call);
    if (fut.wait_for(std::chrono::milliseconds(limit_ms)) == std::future_status::timeout) {
        return {false, std::format("timed out after {}ms", limit_ms)};
    }
    return {true, fut.get()};
}

Outcome fake_flaky(const std::string& prompt, int attempt) {
    std::this_thread::sleep_for(10ms);
    if (attempt < 3) return {false, std::format("service unavailable (attempt {})", attempt)};
    return {true, std::format("answer to '{}' (attempt {})", prompt, attempt)};
}

// TODO(6): 지수 백오프 재시도.
Outcome retry(int max_attempts, int base_ms, const std::function<Outcome(int)>& op) {
    int attempt = 1;
    while (true) {
        Outcome r = op(attempt);
        if (r.ok) return r;
        std::cout << std::format("  attempt {} failed: {}\n", attempt, r.text);
        if (attempt >= max_attempts) return r;
        std::this_thread::sleep_for(std::chrono::milliseconds(base_ms * (1 << (attempt - 1))));
        ++attempt;
    }
}

// TODO(7): 먼저 끝나는 쪽 (Rust: tokio::select!). 두 future 를 번갈아 살펴봅니다(폴링).
// 진 쪽도 소멸자에서 끝까지 기다립니다 (취소 불가).
std::string race(const Call& a, const Call& b) {
    auto fa = std::async(std::launch::async, fake_llm_call, a);
    auto fb = std::async(std::launch::async, fake_llm_call, b);
    while (true) {
        if (fa.wait_for(1ms) == std::future_status::ready) return fa.get();
        if (fb.wait_for(1ms) == std::future_status::ready) return fb.get();
    }
}

// TODO(8): CPU 작업. std::async 는 이미 스레드이므로 spawn_blocking 과 비슷한 역할입니다.
std::int64_t sum_of_squares(std::int64_t n) {
    std::int64_t total = 0;
    for (std::int64_t i = 1; i <= n; ++i) total += i * i;
    return total;
}

std::int64_t blocking_sum(std::int64_t n) {
    return std::async(std::launch::async, sum_of_squares, n).get();
}

double ms_since(Clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(Clock::now() - t0).count();
}

int main() {
    std::cout << "== 1. futures are lazy ==\n";
    {
        Call lazy_call{"lazy", 10};
        // launch::deferred = "get() 할 때까지 실행하지 않음". Rust Future 의 lazy 와 가장 닮은 모드입니다.
        auto fut = std::async(std::launch::deferred, [&] {
            std::cout << "  [lazy] body is running\n";
            return fake_llm_call(lazy_call);
        });
        std::cout << "future created, nothing has run yet\n";
        std::cout << "now awaiting the future\n";
        std::string result = fut.get();  // << 체인 안에서 부르면 출력 순서가 꼬일 수 있어 따로 받습니다
        std::cout << "result: " << result << "\n";
    }

    std::vector<Call> calls = {
        {"summarize", 100}, {"translate", 200}, {"classify", 300}, {"extract", 150}, {"rewrite", 150},
    };

    std::cout << "== 2. sequential ==\n";
    auto t0 = Clock::now();
    auto seq = run_sequential(calls);
    double seq_ms = ms_since(t0);
    for (std::size_t i = 0; i < seq.size(); ++i) std::cout << std::format("[{}] {}\n", i, seq[i]);

    std::cout << "== 3. concurrent ==\n";
    t0 = Clock::now();
    auto conc = run_concurrent(calls);
    double conc_ms = ms_since(t0);
    for (std::size_t i = 0; i < conc.size(); ++i) std::cout << std::format("[{}] {}\n", i, conc[i]);
    std::cout << "concurrent faster: " << (conc_ms * 2 < seq_ms ? "yes" : "no") << "\n";

    std::cout << "== 4. join! ==\n";
    auto [ra, rb] = join_pair({"a", 100}, {"b", 200});
    std::cout << std::format("pair: {} | {}\n", ra, rb);

    std::cout << "== 5. timeout ==\n";
    for (auto [call, limit] : std::vector<std::pair<Call, int>>{{{"slow", 500}, 150}, {{"quick", 50}, 150}}) {
        Outcome r = call_with_timeout(call, limit);
        std::cout << std::format("{} ({}ms, limit {}ms): {}: {}\n", call.prompt, call.latency_ms, limit,
                                 r.ok ? "ok" : "error", r.text);
    }

    std::cout << "== 6. retry with backoff ==\n";
    for (int max_attempts : {5, 2}) {
        Outcome r = retry(max_attempts, 10, [](int n) { return fake_flaky("flaky", n); });
        std::cout << std::format("retry (max {}): {}: {}\n", max_attempts, r.ok ? "ok" : "error", r.text);
    }

    std::cout << "== 7. select (race) ==\n";
    std::cout << "winner: " << race({"fast_model", 50}, {"slow_model", 300}) << "\n";

    std::cout << "== 8. blocking work ==\n";
    std::int64_t n = 1'000'000;
    std::cout << std::format("sum of squares 1..={}: {}\n", n, blocking_sum(n));
    return 0;
}
