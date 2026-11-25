// Ch13. 동시성 - C++20 참조 구현.
//   std::thread / std::mutex / std::shared_mutex / std::condition_variable 로 구현합니다.
//
// C++ 와 Rust 의 결정적 차이:
//   C++ : 뮤텍스 없이 공유 변수를 여러 스레드가 고치면 컴파일되고, 실행하면 데이터 레이스 = 미정의 동작(UB).
//         (thread sanitizer 로 "실행 중에" 잡을 수 있을 뿐)
//   Rust: 같은 코드는 "컴파일 에러" (Send / Sync 트레이트 검사). 그래서 데이터 레이스가 없습니다.
// C++ 에는 표준 채널(channel)이 없어서 mutex + condition_variable 로 작은 Channel 을 직접 만듭니다.
#include <algorithm>
#include <condition_variable>
#include <format>
#include <iostream>
#include <mutex>
#include <optional>
#include <queue>
#include <shared_mutex>
#include <string>
#include <thread>
#include <utility>
#include <vector>

bool is_prime(unsigned long n) {
    if (n < 2) return false;
    for (unsigned long i = 2; i * i <= n; ++i)
        if (n % i == 0) return false;
    return true;
}

std::size_t count_primes(unsigned long lo, unsigned long hi) {
    std::size_t c = 0;
    for (unsigned long n = lo; n < hi; ++n)
        if (is_prime(n)) ++c;
    return c;
}

// Rust 의 mpsc::channel 에 해당하는 최소 구현 (여러 생산자, 소비자는 여럿이어도 OK)
template <typename T>
class Channel {
public:
    void send(T v) {
        {
            std::lock_guard lk(m_);
            q_.push(std::move(v));
        }
        cv_.notify_one();
    }
    void close() {  // Rust: 모든 Sender 가 drop 되면 자동으로 닫힘
        {
            std::lock_guard lk(m_);
            closed_ = true;
        }
        cv_.notify_all();
    }
    std::optional<T> recv() {  // 닫혀서 비면 nullopt  (Rust: Err(RecvError))
        std::unique_lock lk(m_);
        cv_.wait(lk, [&] { return !q_.empty() || closed_; });
        if (q_.empty()) return std::nullopt;
        T v = std::move(q_.front());
        q_.pop();
        return v;
    }

private:
    std::mutex m_;
    std::condition_variable cv_;
    std::queue<T> q_;
    bool closed_ = false;
};

// ───────────── 1. 스레드 생성 + join ─────────────
std::vector<unsigned long> parallel_squares(const std::vector<unsigned long>& inputs) {
    std::vector<unsigned long> results(inputs.size());
    std::vector<std::thread> ts;
    for (std::size_t i = 0; i < inputs.size(); ++i)
        ts.emplace_back([&results, &inputs, i] { results[i] = inputs[i] * inputs[i]; });  // 각 스레드는 자기 칸만 씀
    for (auto& t : ts) t.join();
    return results;
}

// ───────────── 2. 빌려 쓰는 스레드 (Rust: thread::scope) ─────────────
// C++ 는 참조 캡처 후 join 을 "잊지 않는 것"이 전적으로 프로그래머 책임입니다. (jthread 는 소멸자에서 join)
std::vector<unsigned long> scoped_chunk_sums(const std::vector<unsigned long>& data, std::size_t chunks) {
    std::size_t size = (data.size() + chunks - 1) / chunks;
    std::size_t n = (data.size() + size - 1) / size;
    std::vector<unsigned long> sums(n, 0);
    {
        std::vector<std::jthread> ts;
        for (std::size_t i = 0; i < n; ++i) {
            ts.emplace_back([&, i] {
                std::size_t b = i * size, e = std::min(data.size(), b + size);
                for (std::size_t k = b; k < e; ++k) sums[i] += data[k];
            });
        }
    }  // jthread 소멸 = join
    return sums;
}

// ───────────── 3. 공유 카운터 + mutex ─────────────
std::size_t counter_with_mutex(std::size_t threads, std::size_t per_thread) {
    std::size_t counter = 0;
    std::mutex m;
    {
        std::vector<std::jthread> ts;
        for (std::size_t t = 0; t < threads; ++t)
            ts.emplace_back([&] {
                for (std::size_t i = 0; i < per_thread; ++i) {
                    std::lock_guard lk(m);  // Rust: m.lock().unwrap()  (값이 Mutex 안에 들어 있어서 잊을 수가 없음)
                    ++counter;
                }
            });
    }
    return counter;
}

// ───────────── 4. 채널 ─────────────
std::vector<std::string> collect_messages(std::size_t producers, std::size_t per) {
    Channel<std::string> ch;
    {
        std::vector<std::jthread> ts;
        for (std::size_t p = 0; p < producers; ++p)
            ts.emplace_back([&ch, p, per] {
                for (std::size_t m = 0; m < per; ++m) ch.send(std::format("producer {} / msg {}", p, m));
            });
    }
    ch.close();
    std::vector<std::string> out;
    while (auto v = ch.recv()) out.push_back(std::move(*v));
    std::ranges::sort(out);
    return out;
}

// ───────────── 5. 읽기/쓰기 락 ─────────────
std::pair<std::vector<unsigned>, std::vector<unsigned>> rwlock_demo() {
    std::vector<unsigned> data;
    std::shared_mutex rw;
    {
        std::vector<std::jthread> ws;
        for (unsigned i = 0; i < 4; ++i)
            ws.emplace_back([&, i] {
                std::unique_lock lk(rw);  // 쓰기 락 (배타)  = Rust write()
                data.push_back(i * 10);
            });
    }
    std::vector<unsigned> sums(3, 0);
    {
        std::vector<std::jthread> rs;
        for (std::size_t i = 0; i < 3; ++i)
            rs.emplace_back([&, i] {
                std::shared_lock lk(rw);  // 읽기 락 (공유) = Rust read()
                for (unsigned v : data) sums[i] += v;
            });
    }
    std::ranges::sort(data);
    return {data, sums};
}

// ───────────── 6. 워커 풀 ─────────────
struct Job {
    std::size_t id;
    unsigned long lo, hi;
};

std::vector<std::pair<std::size_t, std::size_t>> worker_pool(const std::vector<Job>& jobs, std::size_t workers) {
    Channel<Job> job_ch;
    Channel<std::pair<std::size_t, std::size_t>> result_ch;
    for (const auto& j : jobs) job_ch.send(j);
    job_ch.close();
    {
        std::vector<std::jthread> ts;
        for (std::size_t w = 0; w < workers; ++w)
            ts.emplace_back([&] {
                while (auto job = job_ch.recv()) result_ch.send({job->id, count_primes(job->lo, job->hi)});
            });
    }
    result_ch.close();
    std::vector<std::pair<std::size_t, std::size_t>> results;
    while (auto r = result_ch.recv()) results.push_back(*r);
    std::ranges::sort(results);
    return results;
}

template <typename T>
std::string join_list(const std::vector<T>& v) {
    std::string s = "[";
    for (std::size_t i = 0; i < v.size(); ++i) {
        if (i) s += ", ";
        s += std::to_string(v[i]);
    }
    return s + "]";
}

int main() {
    std::cout << "== 1. spawn + join ==\n";
    std::cout << "squares: " << join_list(parallel_squares({1, 2, 3, 4, 5, 6, 7, 8})) << "\n";

    std::cout << "== 2. scoped threads ==\n";
    std::vector<unsigned long> data;
    for (unsigned long i = 1; i <= 100; ++i) data.push_back(i);
    auto sums = scoped_chunk_sums(data, 4);
    unsigned long total = 0;
    for (auto s : sums) total += s;
    std::cout << "chunk sums: " << join_list(sums) << "\n";
    std::cout << "total: " << total << "\n";

    std::cout << "== 3. Arc<Mutex> ==\n";
    std::cout << "counter: " << counter_with_mutex(8, 1000) << " (8 threads x 1000)\n";

    std::cout << "== 4. channel ==\n";
    for (const auto& m : collect_messages(3, 2)) std::cout << "  " << m << "\n";

    std::cout << "== 5. RwLock ==\n";
    auto [written, seen] = rwlock_demo();
    std::cout << "written: " << join_list(written) << "\n";
    std::cout << "readers saw: " << join_list(seen) << "\n";

    std::cout << "== 6. worker pool ==\n";
    std::vector<Job> jobs;
    for (std::size_t i = 0; i < 10; ++i) jobs.push_back({i, i * 10000UL, (i + 1) * 10000UL});
    auto results = worker_pool(jobs, 4);
    std::size_t grand = 0;
    for (auto [jid, n] : results) {
        std::cout << std::format("  job {}: [{}, {}) -> {} primes\n", jid, jobs[jid].lo, jobs[jid].hi, n);
        grand += n;
    }
    std::cout << "total primes below 100000: " << grand << "\n";
}
