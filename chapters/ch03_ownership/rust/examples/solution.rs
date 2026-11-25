//! Ch03. 작업 파이프라인 - 정답
//!
//! 실행:  cargo run -q -p ch03_ownership --example solution
//! 테스트: cargo test -p ch03_ownership --example solution

use std::cell::Cell;

// ───────── 데이터 타입 ─────────

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
impl Drop for Task {
    fn drop(&mut self) {
        println!("drop: {}", self.name);
        DROPS.with(|d| d.set(d.get() + 1));
    }
}

impl Task {
    fn new(name: &str, priority: u8) -> Task {
        Task {
            name: name.to_string(), // &str(빌린 글자) -> String(소유하는 글자): 힙에 복사해서 새 소유자를 만듭니다
            priority,
            steps: 0,
        }
    }
}

// ───────── 파이프라인 단계 ─────────

/// `task: Task` -> 소유권이 이 함수로 "이동" 해 들어옵니다. push 하면 Vec 이 새 소유자.
fn submit(inbox: &mut Vec<Task>, task: Task) {
    inbox.push(task);
}

/// 맨 앞 작업을 꺼내 "소유권째" 호출자에게 돌려줍니다. 비었으면 None.
fn take_next(inbox: &mut Vec<Task>) -> Option<Task> {
    if inbox.is_empty() {
        None
    } else {
        Some(inbox.remove(0)) // remove 는 요소를 Vec 밖으로 "이동" 시켜 돌려줍니다
    }
}

/// 소유권이 들어왔다가 (수정된 채) 다시 나갑니다. 복사는 한 번도 없어요.
fn process(mut task: Task) -> Task {
    task.steps += 1;
    task
}

fn finish(done: &mut Vec<Task>, task: Task) {
    done.push(task);
}

/// 소유권을 받고 아무에게도 넘기지 않으면 함수 끝 `}` 에서 task 가 drop 됩니다.
fn cancel(task: Task) {
    println!("cancel: {}", task.name);
} // <- 여기서 drop

/// (복사본, 원본) 을 돌려줍니다. clone 은 힙의 문자열까지 새로 복사하는 "비싼" 연산이라 명시적으로 써야 해요.
fn backup(task: Task) -> (Task, Task) {
    let copy = task.clone();
    (copy, task)
}

/// inbox 의 소유권을 통째로 받아서, 처리 결과를 담은 새 Vec 의 소유권을 돌려줍니다.
fn run_all(mut inbox: Vec<Task>) -> Vec<Task> {
    let mut done = Vec::new();
    while let Some(task) = take_next(&mut inbox) {
        println!("start: {}", task.name);
        let task = process(task); // shadowing: 같은 이름으로 소유권을 다시 받음
        let priority = task.priority; // u8 은 Copy 라서 복사됩니다 (task 는 그대로 사용 가능)
        if priority == 0 {
            cancel(task); // task 이동 -> cancel 안에서 drop
        } else {
            println!("finish: {} (steps={})", task.name, task.steps);
            finish(&mut done, task);
        }
    }
    done
}

/// Task 는 Drop 을 구현해서 `task.name` 을 밖으로 "이동" 시킬 수 없어요(E0509). 그래서 clone 합니다.
/// `for task in done` 은 done 의 소유권을 소비하고, 한 바퀴가 끝날 때마다 task 가 drop 됩니다.
fn drain_names(done: Vec<Task>) -> Vec<String> {
    let mut names = Vec::new();
    for task in done {
        names.push(task.name.clone());
    }
    names
}

fn copy_vs_clone_demo() {
    let a = 5;
    let b = a; // i32 는 Copy: 비트 복사 후 a 도 계속 유효
    println!("a={a} b={b}");
    let s1 = String::from("hello");
    let s2 = s1.clone(); // String 은 Copy 가 아님: 깊은 복사가 필요하면 명시적으로 clone
    println!("s1={s1} s2={s2}");
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
