// Ch16. HTTP 클라이언트 & 실전 에러 처리 (C++20 참조 구현)
// libcurl 로 HTTP 를 보내고 nlohmann::json 으로 JSON 을 다룹니다.
// 빌드: g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -lcurl
#include <curl/curl.h>

#include <chrono>
#include <cstdlib>
#include <format>
#include <functional>
#include <memory>
#include <stdexcept>
#include <iostream>
#include <nlohmann/json.hpp>
#include <string>
#include <thread>
#include <vector>

using json = nlohmann::json;

// ───────────────────────── 에러 타입 (Rust: thiserror enum ApiError) ─────────────────────────
// C++ 에서는 예외 클래스 계층으로, Rust 에서는 enum 한 개로 표현합니다.
enum class ErrorKind { NotFound, Http, Timeout, Network, Decode };

struct ApiError : std::runtime_error {
    ErrorKind kind;
    long status = 0;  // Http 일 때
    ApiError(ErrorKind k, const std::string& msg, long st = 0) : std::runtime_error(msg), kind(k), status(st) {}
    bool retryable() const { return kind == ErrorKind::Timeout || (kind == ErrorKind::Http && status >= 500); }
};

struct Todo {
    int id;
    std::string title;
    bool done;
};

double backoff_delay_ms(int attempt, double base_ms = 50.0) {
    return base_ms * (1 << (attempt - 1));  // 1 -> base, 2 -> 2*base, 3 -> 4*base
}

// ───────────────────────── libcurl 도우미 ─────────────────────────
static size_t write_cb(char* ptr, size_t size, size_t nmemb, void* userdata) {
    static_cast<std::string*>(userdata)->append(ptr, size * nmemb);
    return size * nmemb;
}

class TodoClient {
public:
    TodoClient(std::string base_url, long timeout_ms) : base_url_(std::move(base_url)), timeout_ms_(timeout_ms) {
        while (!base_url_.empty() && base_url_.back() == '/') base_url_.pop_back();
    }

    void reset() { request("POST", "/reset", json::object()); }

    std::vector<Todo> list_todos() {
        std::vector<Todo> out;
        for (const auto& o : request("GET", "/todos")) out.push_back(to_todo(o));
        return out;
    }

    Todo create_todo(const std::string& title) { return to_todo(request("POST", "/todos", json{{"title", title}})); }

    Todo get_todo(int id) {
        try {
            return to_todo(request("GET", "/todos/" + std::to_string(id)));
        } catch (const ApiError& e) {
            if (e.kind == ErrorKind::Http && e.status == 404)
                throw ApiError(ErrorKind::NotFound, std::format("todo {} not found", id));  // 404 는 의미 있는 에러로 번역
            throw;
        }
    }

    json get_flaky() { return request("GET", "/flaky"); }
    json get_slow(int ms) { return request("GET", "/slow?ms=" + std::to_string(ms)); }

private:
    static Todo to_todo(const json& o) {
        try {
            return Todo{o.at("id").get<int>(), o.at("title").get<std::string>(), o.at("done").get<bool>()};
        } catch (const json::exception& e) {
            throw ApiError(ErrorKind::Decode, std::string("decode error: ") + e.what());
        }
    }

    json request(const std::string& method, const std::string& path, const json& body = nullptr) {
        // RAII: 이 핸들은 함수가 끝나면 자동으로 정리됩니다 (Rust 의 Drop 과 같은 발상).
        std::unique_ptr<CURL, decltype(&curl_easy_cleanup)> curl(curl_easy_init(), curl_easy_cleanup);
        std::string response;
        std::string payload = body.is_null() ? "" : body.dump();
        curl_slist* headers = curl_slist_append(nullptr, "Content-Type: application/json");

        curl_easy_setopt(curl.get(), CURLOPT_URL, (base_url_ + path).c_str());
        curl_easy_setopt(curl.get(), CURLOPT_CUSTOMREQUEST, method.c_str());
        curl_easy_setopt(curl.get(), CURLOPT_HTTPHEADER, headers);
        if (!body.is_null()) curl_easy_setopt(curl.get(), CURLOPT_POSTFIELDS, payload.c_str());
        curl_easy_setopt(curl.get(), CURLOPT_TIMEOUT_MS, timeout_ms_);
        curl_easy_setopt(curl.get(), CURLOPT_NOSIGNAL, 1L);
        curl_easy_setopt(curl.get(), CURLOPT_WRITEFUNCTION, write_cb);
        curl_easy_setopt(curl.get(), CURLOPT_WRITEDATA, &response);

        CURLcode rc = curl_easy_perform(curl.get());
        curl_slist_free_all(headers);
        if (rc == CURLE_OPERATION_TIMEDOUT) throw ApiError(ErrorKind::Timeout, "request timed out");
        if (rc != CURLE_OK) throw ApiError(ErrorKind::Network, std::string("network error: ") + curl_easy_strerror(rc));

        long status = 0;
        curl_easy_getinfo(curl.get(), CURLINFO_RESPONSE_CODE, &status);
        if (status < 200 || status >= 300) throw ApiError(ErrorKind::Http, std::format("HTTP {}", status), status);

        json parsed = json::parse(response, nullptr, /*allow_exceptions=*/false);
        if (parsed.is_discarded()) throw ApiError(ErrorKind::Decode, "decode error: invalid JSON");
        return parsed;
    }

    std::string base_url_;
    long timeout_ms_;
};

// op(attempt) 를 최대 max_attempts 번 시도합니다. 재시도 가능한 에러일 때만 쉬었다가 다시 합니다.
template <typename F>
auto retry_with_backoff(int max_attempts, double base_ms, F op) {
    for (int attempt = 1;; ++attempt) {
        try {
            return op(attempt);
        } catch (const ApiError& e) {
            if (!e.retryable() || attempt >= max_attempts) throw;
            std::this_thread::sleep_for(std::chrono::duration<double, std::milli>(backoff_delay_ms(attempt, base_ms)));
        }
    }
}

int main() {
    curl_global_init(CURL_GLOBAL_DEFAULT);
    const char* env = std::getenv("API_BASE_URL");
    std::string base_url = env ? env : "http://127.0.0.1:8787";
    TodoClient client(base_url, 2000);

    std::cout << "== reset ==\n";
    client.reset();
    std::cout << "reset ok\n";

    std::cout << "== create ==\n";
    for (const char* title : {"Learn Rust", "Write tests", "Ship it"}) {
        Todo t = client.create_todo(title);
        std::cout << std::format("created #{} \"{}\" done={}\n", t.id, t.title, t.done ? "true" : "false");
    }

    std::cout << "== list ==\n";
    for (const Todo& t : client.list_todos()) std::cout << std::format("#{} [{}] {}\n", t.id, t.done ? 'x' : ' ', t.title);

    std::cout << "== get ==\n";
    {
        Todo t = client.get_todo(2);
        std::cout << std::format("#{} \"{}\"\n", t.id, t.title);
    }

    std::cout << "== not found ==\n";
    try {
        client.get_todo(999);
    } catch (const ApiError& e) {
        std::cout << "error: failed to fetch todo 999: " << e.what() << "\n";
        std::cout << "  is NotFound? " << (e.kind == ErrorKind::NotFound ? "yes" : "no") << "\n";
    }

    std::cout << "== invalid body ==\n";
    try {
        client.create_todo("");
    } catch (const ApiError& e) {
        std::cout << "error: " << e.what() << "\n";
    }

    std::cout << "== flaky ==\n";
    retry_with_backoff(5, 50.0, [&](int attempt) {
        try {
            json r = client.get_flaky();
            std::cout << std::format("attempt {}: ok (server saw {} calls)\n", attempt, r.at("attempts").get<int>());
            return r;
        } catch (const ApiError& e) {
            std::cout << std::format("attempt {}: {}\n", attempt, e.what());
            throw;
        }
    });

    std::cout << "== timeout ==\n";
    try {
        TodoClient(base_url, 200).get_slow(1000);
    } catch (const ApiError& e) {
        std::cout << "error: " << e.what() << "\n";
    }
    curl_global_cleanup();
}
