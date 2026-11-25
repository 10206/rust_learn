//! # Ch03. 소유권과 이동 - 작업 파이프라인
//!
//! ## 목표
//! - "값에는 소유자가 정확히 한 명" 이라는 규칙과, 소유권이 **이동(move)** 하는 순간을 눈으로 확인한다.
//! - `Copy` 와 `Clone` 의 차이, `impl Drop` 이 호출되는 정확한 타이밍(스코프 끝, 이동, `drop()`)을 이해한다.
//! - 소유권을 "받고 -> 돌려주는" 함수 설계에 익숙해진다.
//! - 🔬 실험실에서 `E0382`(이동 후 사용) 같은 컴파일 에러를 일부러 만들어 읽어 본다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`  (소유권 개념이 없어서 `release()` 를 직접 호출해 흉내 냄)
//! - C++   : `../cpp/main.cpp`    (move 생성자 + 소멸자: moved-from 객체의 소멸자도 실행된다는 점에 주목!)
//!
//! ## 진행 방법
//! 1. 먼저 README 의 "이론" 을 읽고, `cargo run -q -p ch03_ownership --example solution` 으로 정답 출력을 훑어봅니다.
//!    (이 챕터는 **출력의 순서** 자체가 학습 내용이라서, 먼저 "어디서 drop 이 찍히는지" 예측해 보는 게 좋아요.)
//! 2. `TODO(1)` ~ `TODO(10)` 을 순서대로 구현하고 `cargo test -p ch03_ownership` 으로 확인합니다.
//! 3. 맨 아래 `experiments()` 의 🔬 실험을 하나씩 열어 컴파일러의 말을 읽어 보세요.
//!
//! ## 완료 기준
//! - `cargo test -p ch03_ownership` 가 모두 통과
//! - `../../scripts/compare.sh ch03` 에서 Python / C++ / Rust 출력이 똑같음 (drop 순서까지!)

// 다 풀고 나면 이 줄을 지워 보세요. (경고가 어떤 걸 알려주는지 읽어 보는 것도 공부예요)
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::cell::Cell;

// ───────── 데이터 타입 (이미 완성되어 있어요) ─────────

/// `String` 을 소유하는 구조체. `String` 은 Copy 가 아니므로 `Task` 도 Copy 가 될 수 없고,
/// 값을 넘기면 "이동(move)" 합니다. `Clone` 은 명시적 깊은 복사를 허용해요.
#[derive(Clone)]
struct Task {
    name: String,
    priority: u8,
    steps: u32,
}

// (테스트 도우미 - 읽지 않아도 돼요) drop 횟수를 세어서 테스트가 "정말 drop 됐는지" 확인합니다.
thread_local! {
    static DROPS: Cell<u32> = const { Cell::new(0) };
}
#[cfg(test)]
fn drop_count() -> u32 {
    DROPS.with(|d| d.get())
}

/// RAII: 소유자가 사라지는 순간 정확히 한 번 호출됩니다 (C++ 소멸자).
/// 출력되는 `drop: <name>` 줄의 "위치"가 곧 소유권이 끝난 지점입니다.
impl Drop for Task {
    fn drop(&mut self) {
        println!("drop: {}", self.name);
        DROPS.with(|d| d.set(d.get() + 1));
    }
}

// ─────────────────────────────────────────────
// TODO(1): `Task::new` - 이름과 우선순위로 Task 를 만드세요. steps 는 0 으로 시작.
//   🐍 Python: Task(name, priority)                              (python/main.py: Task.__init__)
//   ⚙️ C++  : Task(std::string n, int p) : name(std::move(n))…    (cpp/main.cpp: Task::Task)
//   🦀 Rust : `name: &str` 은 "빌린 글자" 이고 `Task.name` 은 `String`(소유하는 글자) 입니다. 소유자를 새로 만들어야 해요.
//   💡 힌트: `name.to_string()` 또는 `String::from(name)`. 필드 이름과 변수 이름이 같으면 `priority,` 처럼 줄여 쓸 수 있어요.
//   ❓ 왜?: `&str` 만 저장하면 원본이 먼저 사라질 때 문제가 됩니다. 구조체가 데이터를 "소유"하려면 `String` 이어야 해요. (4·9장에서 이어서)
// ─────────────────────────────────────────────
impl Task {
    fn new(name: &str, priority: u8) -> Task {
        todo!("TODO(1): String 을 소유하는 Task 만들기")
    }
}

// ───────── 파이프라인 단계 ─────────

// ─────────────────────────────────────────────
// TODO(2): `submit` - task 를 inbox(Vec) 맨 뒤에 넣으세요.
//   🐍 Python: inbox.append(task)                                 (python/main.py: submit)
//   ⚙️ C++  : inbox.push_back(std::move(task))                    (cpp/main.cpp: submit)
//   🦀 Rust : `task: Task` 라고 쓰면 호출하는 쪽에서 소유권이 "이동" 해 들어옵니다. push 가 그 소유권을 Vec 에게 넘겨요.
//   💡 힌트: `inbox.push(task);`   (`&mut Vec<Task>` 는 "이 Vec 를 잠깐 빌려서 고친다" 는 뜻입니다. 4장에서 자세히!)
//   ❓ 왜?: 이 함수가 끝난 뒤 호출한 쪽에서 `task` 를 다시 쓰려 하면 컴파일 에러 -> 이중 소유를 원천 차단합니다.
//   🔬 실험 1·2: 이동 후에 원래 변수를 쓰면 어떤 에러가 날까요?
// ─────────────────────────────────────────────
fn submit(inbox: &mut Vec<Task>, task: Task) {
    todo!("TODO(2): inbox 에 task 넣기")
}

// ─────────────────────────────────────────────
// TODO(3): `take_next` - 맨 앞 작업을 "꺼내서" 소유권째 돌려주세요. 비었으면 None.
//   🐍 Python: inbox.pop(0)                                       (python/main.py: take_next)
//   ⚙️ C++  : Task t = std::move(inbox.front()); inbox.erase(...)  (cpp/main.cpp: take_next)
//   🦀 Rust : `inbox[0]` 로 꺼내려 하면 안 됩니다 (E0507: Vec 안의 값을 이동시킬 수 없음). `remove(0)` 은 "이동해서 꺼내" 줘요.
//   💡 힌트: `if inbox.is_empty() { None } else { Some(inbox.remove(0)) }`
//   ❓ 왜?: `Option<Task>` 는 "있으면 Some(소유권), 없으면 None" 을 타입으로 표현합니다. null 이 없는 Rust 의 방식이에요. (5·6장)
//   🔬 실험 6: `let first = inbox[0];` 은 왜 안 될까요?
// ─────────────────────────────────────────────
fn take_next(inbox: &mut Vec<Task>) -> Option<Task> {
    todo!("TODO(3): 맨 앞 작업 꺼내기")
}

// ─────────────────────────────────────────────
// TODO(4): `process` - 작업을 받아서 steps 를 1 올리고 "다시 돌려주세요". (들어갔다 나오기)
//   🐍 Python: task.steps += 1; return task                       (python/main.py: process)
//   ⚙️ C++  : Task process(Task task) { task.steps += 1; return task; }   (cpp/main.cpp: process)
//   🦀 Rust : 매개변수에 `mut` 를 붙이면 (`mut task: Task`) 소유한 값을 함수 안에서 바꿀 수 있습니다. 마지막 식으로 task 를 반환.
//   💡 힌트: 복사는 일어나지 않아요. 소유권이 들어왔다가 그대로 나갑니다. (clone 금지!)
//   ❓ 왜?: "소유권을 받아서 고치고 돌려주는" 패턴은 빌더/파이프라인에서 자주 보여요. 다음 장의 `&mut` 이 이걸 더 편하게 해 줍니다.
// ─────────────────────────────────────────────
fn process(mut task: Task) -> Task {
    todo!("TODO(4): steps += 1 하고 task 돌려주기")
}

// ─────────────────────────────────────────────
// TODO(5): `finish` - 끝난 작업을 done(Vec)에 넣으세요. (TODO(2) 와 같은 모양)
//   🐍 Python: done.append(task)                                  (python/main.py: finish)
//   🦀 Rust : 소유권이 Vec 에게 이동합니다. Vec 이 살아 있는 동안 task 도 살아 있어요.
// ─────────────────────────────────────────────
fn finish(done: &mut Vec<Task>, task: Task) {
    todo!("TODO(5): done 에 task 넣기")
}

// ─────────────────────────────────────────────
// TODO(6): `cancel` - "cancel: <name>" 을 한 줄 출력하세요. 그게 전부입니다!
//          task 를 아무에게도 넘기지 않았으므로, 함수가 끝나는 `}` 에서 자동으로 drop 됩니다.
//   🐍 Python: print(...); release(task)  (직접 호출)              (python/main.py: cancel)
//   ⚙️ C++  : 함수 끝에서 매개변수의 소멸자가 실행됨               (cpp/main.cpp: cancel)
//   🦀 Rust : `println!("cancel: {}", task.name);`  -> 출력에서 `cancel:` 바로 다음 줄에 `drop:` 이 나오는지 확인하세요.
//   💡 힌트: println! 은 인자를 "빌려서" 읽기만 합니다. 그래서 `task.name` 을 찍어도 이동하지 않아요.
//   ❓ 왜?: 소멸 시점이 코드에 보이지 않지만 항상 `}` 라는 점이 RAII 의 핵심입니다 (파일/락/소켓이 같은 방식으로 정리돼요).
// ─────────────────────────────────────────────
fn cancel(task: Task) {
    todo!("TODO(6): cancel 메시지 출력")
}

// ─────────────────────────────────────────────
// TODO(7): `backup` - (복사본, 원본) 튜플을 돌려주세요. 복사본은 clone 으로 만듭니다.
//   🐍 Python: copy = task.clone(); return copy, task              (python/main.py: backup)
//   ⚙️ C++  : Task copy = task;  (이 한 줄이 깊은 복사!)            (cpp/main.cpp: backup)
//   🦀 Rust : `let copy = task.clone();` 깊은 복사를 "눈에 보이게" 합니다. 그 다음 `(copy, task)` — task 는 마지막에 이동.
//   💡 힌트: 순서가 중요합니다! `(task, task.clone())` 으로 쓰면 task 가 먼저 이동해 버려서 clone 할 수 없어요.
//   ❓ 왜?: C++ 은 `=` 만 써도 비싼 복사가 조용히 일어나지만 Rust 는 `.clone()` 이 코드에 나타나므로 비용이 보입니다.
// ─────────────────────────────────────────────
fn backup(task: Task) -> (Task, Task) {
    todo!("TODO(7): clone 한 복사본과 원본 돌려주기")
}

// ─────────────────────────────────────────────
// TODO(8): `run_all` - inbox 의 작업을 하나씩 꺼내 처리합니다. (TODO(2)~(7) 을 조립)
//   각 작업마다: "start: <name>" 출력 -> process -> priority 가 0 이면 cancel, 아니면
//   "finish: <name> (steps=<n>)" 출력 후 done 에 finish. 끝나면 done 을 반환.
//   🐍 Python: while (task := take_next(inbox)) is not None: ...    (python/main.py: run_all)
//   ⚙️ C++  : while (auto next = take_next(inbox)) { ... }          (cpp/main.cpp: run_all)
//   🦀 Rust : `while let Some(task) = take_next(&mut inbox) { ... }`  — Some 일 때만 반복하며 소유권을 task 로 꺼냅니다.
//   💡 힌트: `let task = process(task);` (shadowing 으로 소유권을 다시 받기) /
//           `task.priority` 는 u8(Copy) 이므로 읽어도 task 는 이동하지 않아요 /
//           `cancel(task)` 또는 `finish(&mut done, task)` 로 소유권을 넘기면 그 뒤로 task 는 못 씁니다 (분기마다 한 번만!).
//   ❓ 왜?: 어떤 분기에서든 task 의 소유권이 정확히 한 곳으로만 가는지 컴파일러가 끝까지 확인해 줘요.
//   🔬 실험 3: `finish` 를 호출한 뒤 `task.name` 을 출력하면?
// ─────────────────────────────────────────────
fn run_all(mut inbox: Vec<Task>) -> Vec<Task> {
    todo!("TODO(8): 파이프라인 돌리기")
}

// ─────────────────────────────────────────────
// TODO(9): `drain_names` - done 을 "소비"하면서 이름들을 Vec<String> 으로 모아 돌려주세요.
//   🐍 Python: for task in done: names.append(task.name); release(task)   (python/main.py: drain_names)
//   ⚙️ C++  : Task consumed = std::move(t);                                 (cpp/main.cpp: drain_names)
//   🦀 Rust : `for task in done { ... }` 은 done 의 소유권을 가져가서 한 바퀴마다 task 를 하나씩 꺼내고,
//            바퀴가 끝나면 그 task 를 drop 합니다 -> `drop:` 줄이 names 출력보다 "먼저" 나오는 이유!
//   💡 힌트: `names.push(task.name)` 은 컴파일 에러(E0509)! Task 가 Drop 을 구현하면 필드를 밖으로 이동시킬 수 없어요.
//           `task.name.clone()` 으로 복사하세요. (🔬 실험 5 에서 에러를 먼저 만나 보세요)
//   ❓ 왜?: 필드를 빼가면 Drop::drop 이 "반쯤 비어 버린 값" 을 보게 되기 때문입니다. 안전을 위해 아예 금지돼요.
// ─────────────────────────────────────────────
fn drain_names(done: Vec<Task>) -> Vec<String> {
    todo!("TODO(9): done 을 소비하며 이름 모으기")
}

// ─────────────────────────────────────────────
// TODO(10): `copy_vs_clone_demo` - 아래 두 줄을 각각 출력하세요.
//   (1) i32:    `let a = 5; let b = a;`            출력 "a=5 b=5"
//   (2) String: s1 = "hello" 를 s2 로 "복사"          출력 "s1=hello s2=hello"
//   🐍 Python: b = a / s2 = s1                                      (python/main.py: copy_vs_clone_demo)
//   ⚙️ C++  : std::string s2 = s1;  (조용히 힙 복사)                  (cpp/main.cpp: copy_vs_clone_demo)
//   🦀 Rust : (1) i32 는 `Copy` 라서 `let b = a;` 후에도 a 가 유효합니다.
//            (2) `let s2 = s1;` 은 move 라서 s1 을 못 씁니다 -> 둘 다 쓰려면 `s1.clone()`.
//   💡 힌트: 스택에만 있는 작은 값(정수·실수·bool·char)은 Copy, 힙을 가리키는 값(String, Vec)은 Clone 만 가능.
//   ❓ 왜?: Copy 는 "비트 복사해도 안전한 타입" 을 뜻합니다. Drop 을 구현하거나 힙을 소유하면 Copy 가 될 수 없어요 (E0204 / E0184).
//   🔬 실험 4·7: `let s2 = s1;` 후 s1 사용, `#[derive(Copy)]` on Task
// ─────────────────────────────────────────────
fn copy_vs_clone_demo() {
    todo!("TODO(10): Copy 와 Clone 비교 출력")
}


fn main() {
    println!("== 1. submit ==");
    let mut inbox = Vec::new();
    for (name, prio) in [("write", 2), ("build", 1), ("test", 3), ("cleanup", 0), ("deploy", 1)] {
        submit(&mut inbox, Task::new(name, prio));
    }
    println!("inbox size: {}", inbox.len());

    println!("== 2. run pipeline ==");
    let done = run_all(inbox); // inbox 는 여기서 이동됨. 이후 inbox 를 쓰면 E0382!
    println!("done count: {}", done.len());

    println!("== 3. drain names ==");
    let names = drain_names(done);
    println!("names: {}", names.join(","));

    println!("== 4. backup ==");
    let (copy, original) = backup(Task::new("report", 5));
    println!("copy={} original={}", copy.name, original.name);
    drop(copy); // std::mem::drop: 소유권을 받아서 바로 버립니다. 일찍 정리하고 싶을 때
    println!("after explicit drop");

    println!("== 5. scope ==");
    {
        let _inner_a = Task::new("inner-a", 1);
        let _inner_b = Task::new("inner-b", 1);
        println!("leaving scope");
    } // 선언의 "역순"으로 drop: inner-b, inner-a

    println!("== 6. copy vs clone ==");
    copy_vs_clone_demo();

    println!("== 7. end of main ==");
    // main 이 끝나면 아직 소유자가 남아 있는 original 이 drop 됩니다 -> "drop: report"
}

// ═════════════════════════════════════════════
// 🔬 실험실: 소유권 에러와 친해지기  (이 챕터에서 가장 중요한 부분!)
//   한 블록씩 주석(`// `)을 풀고 `cargo check -p ch03_ownership` 를 실행해 에러 메시지를 읽으세요.
//   메시지의 세 줄 -- "move occurs because ... does not implement the `Copy` trait" /
//   "value moved here" / "value used here after move" -- 이 이야기의 전부입니다.
//   다 본 뒤에는 다시 주석 처리하세요. (이 함수는 어디서도 호출되지 않아요)
// ═════════════════════════════════════════════
fn experiments() {
    // [실험 1] 이동 후 사용 (가장 기본).  기대 에러: E0382 (borrow of moved value: `s1`)
    //   ❓ 질문: 에러 메시지의 세 부분 중 "move occurs because" 가 설명하는 이유는 무엇인가요?
    //   ❓ 질문: C++ 의 `std::string s2 = std::move(s1);` 후 s1 을 쓰면 컴파일러가 막나요? (아니요!)
    //
    // let s1 = String::from("hello");
    // let s2 = s1;
    // println!("{s1}, {s2}");

    // [실험 2] 함수에 넘긴 뒤 사용.  기대 에러: E0382
    //   ❓ 질문: 컴파일러가 "consider changing this parameter type ... to borrow instead" 라고 제안해요. 4장에서 배울 해법입니다.
    //
    // let t = Task::new("build", 1);
    // cancel(t);
    // println!("{}", t.name);

    // [실험 3] 반복문 안에서 이동 (double move).  기대 에러: E0382 (value moved here, in previous iteration of loop)
    //   ❓ 질문: "in previous iteration of loop" 는 무슨 뜻일까요? 첫 바퀴는 멀쩡한데 왜 에러일까요?
    //
    // let t = Task::new("loop", 1);
    // for _ in 0..2 {
    //     cancel(t);
    // }

    // [실험 4] 같은 값을 두 번 이동 / 조건부 이동.  기대 에러: E0382
    //   ❓ 질문: if 안에서만 이동해도 if 밖에서 쓰면 에러입니다. 컴파일러는 어떻게 "이동했을 수도 있음" 을 알죠?
    //
    // let s = String::from("x");
    // let a = s;
    // let b = s;                       // 두 번째 이동
    // let n = String::from("y");
    // if std::env::args().count() > 5 { drop(n); }
    // println!("{n}");

    // [실험 5] Drop 을 구현한 타입의 필드를 이동.  기대 에러: E0509 (cannot move out of type `Task`, which implements the `Drop` trait)
    //   ❓ 질문: 왜 금지일까요? (drop 이 호출될 때 name 이 이미 빠져나가 있다면?)  고치는 두 방법: 빌리기(&) / clone().
    //
    // let t = Task::new("write", 1);
    // let n = t.name;

    // [실험 6] Vec 안의 값을 인덱스로 이동.  기대 에러: E0507 (cannot move out of index of `Vec<String>`)
    //   ❓ 질문: C++ 의 `std::string s = v[0];` 는 복사입니다. Rust 는 왜 거절하고, 어떤 대안(`&v[0]`, `.clone()`, `remove`)을 제안하나요?
    //
    // let v = vec![String::from("a"), String::from("b")];
    // let first = v[0];

    // [실험 7] Copy 를 억지로 붙이기.  기대 에러: E0204 (the trait `Copy` cannot be implemented for this type)
    //   (참고: Drop 을 구현한 타입에 Copy 를 붙이면 E0184 — Copy 와 Drop 은 함께 쓸 수 없어요)
    //   ❓ 질문: String 필드가 있으면 왜 Copy 가 불가능할까요? (비트 복사하면 같은 힙을 두 객체가 가리켜 이중 해제!)
    //
    // #[derive(Clone, Copy)]
    // struct Bad { name: String }

    // [실험 8] `for x in vec` 후 vec 재사용.  기대 에러: E0382 (`names` moved due to this implicit call to `.into_iter()`)
    //   ❓ 질문: `for n in &names` 로 바꾸면 되는 이유는? (빌리기 -> 4장)
    //
    // let names = vec![String::from("a"), String::from("b")];
    // for n in names { println!("{n}"); }
    // println!("{}", names.len());

    // [실험 9] 명시적으로 drop 메서드 호출.  기대 에러: E0040 (explicit use of destructor method)
    //   ❓ 질문: 일찍 버리고 싶을 땐? -> `drop(t)` (std::mem::drop). C++ 의 `t.~Task()` 와 비교해 보세요.
    //
    // let t = Task::new("x", 1);
    // t.drop();

    // [실험 10] 🧠 drop 순서 예측 (컴파일 에러 아님!).  주석을 풀고 `cargo run -p ch03_ownership` 하기 전에 예측해 보세요.
    //   a, b, c 는 어떤 순서로 drop 될까요?  `let _ = Task::new("temp", 1);` 은 언제 drop 될까요?
    //   (정답은 README 의 "drop 순서 규칙" 에 있어요. 틀려도 괜찮습니다. 직접 눈으로 보는 게 목적!)
    //   실행하려면 main 의 첫 줄에 `experiments();` 를 잠깐 넣으세요.
    //
    // let _a = Task::new("a", 1);
    // let _b = Task::new("b", 1);
    // let _ = Task::new("temp", 1);
    // let _c = Task::new("c", 1);
    // println!("end of experiments");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_of(v: &[Task]) -> Vec<&str> {
        v.iter().map(|t| t.name.as_str()).collect()
    }

    #[test]
    fn new_sets_fields() {
        let t = Task::new("a", 3);
        assert_eq!(t.name, "a");
        assert_eq!(t.priority, 3);
        assert_eq!(t.steps, 0);
    }

    #[test]
    fn submit_and_take_are_fifo() {
        let mut inbox = Vec::new();
        submit(&mut inbox, Task::new("first", 1));
        submit(&mut inbox, Task::new("second", 1));
        assert_eq!(names_of(&inbox), ["first", "second"]);
        let t = take_next(&mut inbox).expect("작업이 있어야 해요");
        assert_eq!(t.name, "first");
        assert_eq!(inbox.len(), 1);
        assert!(take_next(&mut Vec::new()).is_none());
    }

    #[test]
    fn process_returns_same_task_with_one_more_step() {
        let t = process(Task::new("p", 1));
        assert_eq!(t.name, "p");
        assert_eq!(t.steps, 1);
        let t = process(t);
        assert_eq!(t.steps, 2);
    }

    #[test]
    fn cancel_drops_exactly_once() {
        let before = drop_count();
        cancel(Task::new("x", 0));
        assert_eq!(drop_count() - before, 1);
    }

    #[test]
    fn finish_keeps_task_alive() {
        let before = drop_count();
        let mut done = Vec::new();
        finish(&mut done, Task::new("kept", 1));
        assert_eq!(drop_count() - before, 0, "Vec 에 넣은 작업은 아직 살아 있어야 해요");
        assert_eq!(done.len(), 1);
    }

    #[test]
    fn run_all_skips_priority_zero() {
        let mut inbox = Vec::new();
        submit(&mut inbox, Task::new("a", 1));
        submit(&mut inbox, Task::new("zero", 0));
        submit(&mut inbox, Task::new("b", 2));
        let before = drop_count();
        let done = run_all(inbox);
        assert_eq!(names_of(&done), ["a", "b"]);
        assert!(done.iter().all(|t| t.steps == 1));
        assert_eq!(drop_count() - before, 1, "취소된 작업 하나만 drop 되어야 해요");
    }

    #[test]
    fn backup_makes_independent_copy() {
        let (copy, original) = backup(Task::new("r", 5));
        assert_eq!(copy.name, original.name);
        assert_ne!(copy.name.as_ptr(), original.name.as_ptr(), "clone 은 힙 메모리까지 새로 만들어요");
    }

    #[test]
    fn drain_names_consumes_and_drops_all() {
        let done = vec![Task::new("x", 1), Task::new("y", 1), Task::new("z", 1)];
        let before = drop_count();
        let names = drain_names(done);
        assert_eq!(names, ["x", "y", "z"]);
        assert_eq!(drop_count() - before, 3);
    }
}
