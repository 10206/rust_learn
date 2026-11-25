// Ch03. 작업 파이프라인 - C++20 참조 구현.
//
// ★ 이 파일의 핵심 비교 (Rust 의 move 와 C++ 의 std::move 는 전혀 다른 것!)
//
//  C++ : `std::move(x)` 는 이동시키지 않아요. x 를 "rvalue 로 취급해도 좋다"는 캐스트일 뿐이고,
//        실제 이동은 move 생성자/대입 연산자가 하는 일입니다 (직접 작성해야 할 수도 있음).
//        이동이 끝난 x(moved-from)는 "유효하지만 값은 불특정" 상태로 계속 살아 있고,
//        **소멸자도 여전히 실행됩니다.** 그래서 아래 ~Task() 는 "빈 이름이면 아무것도 안 함" 검사를 해야 해요.
//        (검사를 빼먹으면 `drop: ` 이 이동 횟수만큼 더 찍힙니다.)
//        그리고 컴파일러는 이동 뒤의 x 사용을 막지 않습니다. 쓰면 버그지만 컴파일은 돼요.
//
//  Rust: 기본이 move. 이동은 비트 복사 + "원본을 컴파일 타임에 무효화" 입니다.
//        이동한 변수는 쓸 수 없고(E0382), 소멸자도 실행되지 않아서 "빈 객체" 검사 자체가 필요 없습니다.
//        소유자가 항상 정확히 한 명이라 소멸자(Drop)도 정확히 한 번만 호출돼요.
#include <format>
#include <iostream>
#include <optional>
#include <string>
#include <utility>
#include <vector>

class Task {
public:
    std::string name;
    int priority;
    int steps;

    Task(std::string n, int p) : name(std::move(n)), priority(p), steps(0) {}

    Task(const Task&) = default;             // Rust: #[derive(Clone)] (복사는 C++ 에서는 기본, Rust 에서는 .clone() 명시)
    Task& operator=(const Task&) = default;

    // move 생성자: 이름을 훔치고, 원본을 "빈 상태"로 만들어 둡니다 (moved-from 상태를 우리가 정의).
    Task(Task&& other) noexcept
        : name(std::move(other.name)), priority(other.priority), steps(other.steps) {
        other.name.clear();  // std::string 의 moved-from 은 "유효하지만 불특정" 이라 명시적으로 비웁니다.
    }
    Task& operator=(Task&& other) noexcept {
        if (this != &other) {
            name = std::move(other.name);
            priority = other.priority;
            steps = other.steps;
            other.name.clear();
        }
        return *this;
    }

    ~Task() {
        // moved-from 객체의 소멸자도 실행되므로, "살아 있는 객체인지" 직접 확인해야 합니다.
        if (!name.empty()) {
            std::cout << "drop: " << name << '\n';
        }
    }
};

// ---------- 파이프라인 단계 ----------
// 값 매개변수(Task task) + 호출부의 std::move  ==  Rust 의 `task: Task` (소유권을 넘김)
void submit(std::vector<Task>& inbox, Task task) { inbox.push_back(std::move(task)); }

std::optional<Task> take_next(std::vector<Task>& inbox) {
    if (inbox.empty()) return std::nullopt;
    Task front = std::move(inbox.front());
    inbox.erase(inbox.begin());
    return front;
}

Task process(Task task) {
    task.steps += 1;
    return task;  // 지역 변수 반환은 자동으로 move (또는 NRVO)
}

void finish(std::vector<Task>& done, Task task) { done.push_back(std::move(task)); }

void cancel(Task task) {
    std::cout << "cancel: " << task.name << '\n';
}  // <- 여기서 task 의 소멸자 실행 (Rust 의 drop 과 같은 타이밍)

std::pair<Task, Task> backup(Task task) {
    Task copy = task;  // 복사: C++ 에서는 `=` 만 쓰면 깊은 복사가 일어납니다! (Rust 는 .clone() 을 써야만)
    return {std::move(copy), std::move(task)};
}

std::vector<Task> run_all(std::vector<Task> inbox) {
    std::vector<Task> done;
    while (auto next = take_next(inbox)) {
        std::cout << "start: " << next->name << '\n';
        Task task = process(std::move(*next));
        if (task.priority == 0) {
            cancel(std::move(task));
        } else {
            std::cout << std::format("finish: {} (steps={})\n", task.name, task.steps);
            finish(done, std::move(task));
        }
    }
    return done;
}

std::vector<std::string> drain_names(std::vector<Task> done) {
    std::vector<std::string> names;
    for (Task& t : done) {
        names.push_back(t.name);
        Task consumed = std::move(t);  // 이 반복이 끝날 때 consumed 가 소멸 == Rust 의 `for task in done`
    }
    return names;
}

void drop_now(Task) {}  // Rust 의 std::mem::drop: 소유권만 받고 아무것도 안 하면, 함수가 끝날 때 소멸

void copy_vs_clone_demo() {
    int a = 5;
    int b = a;
    std::cout << std::format("a={} b={}\n", a, b);
    std::string s1 = "hello";
    std::string s2 = s1;  // C++ 은 이 한 줄이 힙 메모리 복사! Rust 라면 s1 이 move 되어 이후 사용 불가
    std::cout << std::format("s1={} s2={}\n", s1, s2);
}

int main() {
    std::cout << "== 1. submit ==\n";
    std::vector<Task> inbox;
    for (auto [name, prio] : {std::pair<const char*, int>{"write", 2}, {"build", 1}, {"test", 3},
                              {"cleanup", 0}, {"deploy", 1}}) {
        submit(inbox, Task(name, prio));
    }
    std::cout << "inbox size: " << inbox.size() << '\n';

    std::cout << "== 2. run pipeline ==\n";
    std::vector<Task> done = run_all(std::move(inbox));
    std::cout << "done count: " << done.size() << '\n';

    std::cout << "== 3. drain names ==\n";
    std::vector<std::string> names = drain_names(std::move(done));
    std::string joined;
    for (std::size_t i = 0; i < names.size(); ++i) {
        if (i > 0) joined += ",";
        joined += names[i];
    }
    std::cout << "names: " << joined << '\n';

    std::cout << "== 4. backup ==\n";
    auto [copy, original] = backup(Task("report", 5));
    std::cout << std::format("copy={} original={}\n", copy.name, original.name);
    drop_now(std::move(copy));
    std::cout << "after explicit drop\n";

    std::cout << "== 5. scope ==\n";
    {
        Task inner_a("inner-a", 1);
        Task inner_b("inner-b", 1);
        std::cout << "leaving scope\n";
    }  // 선언의 역순으로 소멸: inner-b, inner-a  (Rust 와 같은 규칙)

    std::cout << "== 6. copy vs clone ==\n";
    copy_vs_clone_demo();

    std::cout << "== 7. end of main ==\n";
    return 0;  // original 소멸 -> "drop: report"
}
