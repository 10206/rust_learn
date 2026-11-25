"""Ch03. 작업 파이프라인 - Python 참조 구현.

Python 의 변수는 객체에 붙은 "이름표"입니다. `b = a` 는 같은 객체에 이름표를 하나 더 붙일 뿐이에요.
객체는 이름표가 하나도 안 남으면 (참조 카운트 0) 치워집니다.

Rust / C++ 의 소멸자 타이밍을 똑같이 보여 주려고, 이 파일에서는 `release()` 를 "직접" 호출해서
`drop: <name>` 을 출력합니다. (CPython 의 __del__ 로도 비슷하게 할 수 있지만, 실행 환경에 따라
시점이 달라질 수 있어서 이 코스에서는 명시적으로 부릅니다.)
주석의 "Rust:" 는 그 지점에서 Rust 가 자동으로 해 주는 일을 뜻해요.
"""
from __future__ import annotations


class Task:
    def __init__(self, name: str, priority: int) -> None:
        self.name = name
        self.priority = priority
        self.steps = 0

    def clone(self) -> "Task":
        # Rust: #[derive(Clone)] 가 만들어 주는 깊은 복사. Python 은 일부러 새 객체를 만들어야 해요.
        c = Task(self.name, self.priority)
        c.steps = self.steps
        return c


def release(task: Task) -> None:
    """Rust: 소유자가 스코프를 벗어나면 `impl Drop` 의 drop 이 자동 호출됩니다."""
    print(f"drop: {task.name}")


# ---------- 파이프라인 단계 ----------
def submit(inbox: list[Task], task: Task) -> None:
    inbox.append(task)


def take_next(inbox: list[Task]) -> Task | None:
    if not inbox:
        return None
    return inbox.pop(0)


def process(task: Task) -> Task:
    # Rust: 소유권을 받아서 고친 뒤 다시 돌려줍니다(들어갔다 나오기).
    task.steps += 1
    return task


def finish(done: list[Task], task: Task) -> None:
    done.append(task)


def cancel(task: Task) -> None:
    print(f"cancel: {task.name}")
    release(task)  # Rust: 함수가 끝나는 `}` 에서 task 가 자동으로 drop 됩니다.


def backup(task: Task) -> tuple[Task, Task]:
    copy = task.clone()
    return copy, task


def run_all(inbox: list[Task]) -> list[Task]:
    done: list[Task] = []
    while (task := take_next(inbox)) is not None:
        print(f"start: {task.name}")
        task = process(task)
        if task.priority == 0:
            cancel(task)
        else:
            print(f"finish: {task.name} (steps={task.steps})")
            finish(done, task)
    return done


def drain_names(done: list[Task]) -> list[str]:
    names: list[str] = []
    for task in done:
        names.append(task.name)
        release(task)  # Rust: for 루프의 한 바퀴가 끝날 때마다 task 가 drop 됩니다.
    return names


def copy_vs_clone_demo() -> None:
    a = 5
    b = a
    print(f"a={a} b={b}")
    s1 = "hello"
    s2 = s1  # Python 문자열은 불변이라 같은 객체를 공유해도 안전합니다
    print(f"s1={s1} s2={s2}")


def main() -> None:
    print("== 1. submit ==")
    inbox: list[Task] = []
    for name, prio in [("write", 2), ("build", 1), ("test", 3), ("cleanup", 0), ("deploy", 1)]:
        submit(inbox, Task(name, prio))
    print(f"inbox size: {len(inbox)}")

    print("== 2. run pipeline ==")
    done = run_all(inbox)
    print(f"done count: {len(done)}")

    print("== 3. drain names ==")
    names = drain_names(done)
    print("names: " + ",".join(names))

    print("== 4. backup ==")
    copy, original = backup(Task("report", 5))
    print(f"copy={copy.name} original={original.name}")
    release(copy)  # Rust: drop(copy) -- 명시적으로 일찍 버리기
    print("after explicit drop")

    print("== 5. scope ==")
    inner_a = Task("inner-a", 1)
    inner_b = Task("inner-b", 1)
    print("leaving scope")
    release(inner_b)  # Rust: 선언의 "역순"으로 자동 drop (b 먼저, a 나중)
    release(inner_a)

    print("== 6. copy vs clone ==")
    copy_vs_clone_demo()

    print("== 7. end of main ==")
    release(original)  # Rust: main 이 끝나면서 아직 살아 있는 original 이 drop 됩니다.


if __name__ == "__main__":
    main()
