"""Ch13. 동시성 - Python 참조 구현.

Python 메모: CPython 에는 GIL(Global Interpreter Lock)이 있어서 한 번에 파이썬 바이트코드를
실행하는 스레드는 하나뿐입니다. 그래서
  - I/O 대기(네트워크, 파일)에는 threading 이 유효하지만
  - CPU 계산을 여러 코어로 나누려면 multiprocessing 이 필요합니다 (Rust 는 스레드로 진짜 병렬).
그래도 `counter += 1` 같은 read-modify-write 는 원자적이지 않으므로 Lock 이 필요합니다.
출력은 스레드 스케줄링과 무관하게 항상 같도록 결과를 "정렬/인덱스 순서"로 모읍니다.
"""

from __future__ import annotations

import queue
import threading
from dataclasses import dataclass


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    i = 2
    while i * i <= n:
        if n % i == 0:
            return False
        i += 1
    return True


def count_primes(lo: int, hi: int) -> int:
    """[lo, hi) 구간의 소수 개수."""
    return sum(1 for n in range(lo, hi) if is_prime(n))


# ───────────── 1. 스레드 생성 + join ─────────────
def parallel_squares(inputs: list[int]) -> list[int]:
    results = [0] * len(inputs)  # 각 스레드가 자기 칸에만 씁니다 (Rust: JoinHandle 이 값을 돌려줌)

    def work(i: int, x: int) -> None:
        results[i] = x * x

    threads = [threading.Thread(target=work, args=(i, x)) for i, x in enumerate(inputs)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return results


# ───────────── 2. 빌려 쓰는 스레드 (Rust: thread::scope) ─────────────
def scoped_chunk_sums(data: list[int], chunks: int) -> list[int]:
    size = -(-len(data) // chunks)  # 올림 나눗셈
    parts = [data[i : i + size] for i in range(0, len(data), size)]
    sums = [0] * len(parts)

    def work(i: int) -> None:
        sums[i] = sum(parts[i])

    threads = [threading.Thread(target=work, args=(i,)) for i in range(len(parts))]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return sums


# ───────────── 3. 공유 카운터 + Lock (Rust: Arc<Mutex<usize>>) ─────────────
def counter_with_mutex(threads: int, per_thread: int) -> int:
    counter = 0
    lock = threading.Lock()

    def work() -> None:
        nonlocal counter
        for _ in range(per_thread):
            with lock:  # Rust: let mut g = m.lock().unwrap(); *g += 1;
                counter += 1

    ts = [threading.Thread(target=work) for _ in range(threads)]
    for t in ts:
        t.start()
    for t in ts:
        t.join()
    return counter


# ───────────── 4. 채널 (Rust: mpsc) ─────────────
def collect_messages(producers: int, per: int) -> list[str]:
    q: queue.Queue[str] = queue.Queue()

    def produce(pid: int) -> None:
        for m in range(per):
            q.put(f"producer {pid} / msg {m}")

    ts = [threading.Thread(target=produce, args=(p,)) for p in range(producers)]
    for t in ts:
        t.start()
    for t in ts:
        t.join()
    out: list[str] = []
    while not q.empty():
        out.append(q.get())
    out.sort()  # 도착 순서는 비결정적 -> 정렬해서 출력
    return out


# ───────────── 5. 읽기/쓰기 락 (Rust: RwLock) ─────────────
def rwlock_demo() -> tuple[list[int], list[int]]:
    # 표준 라이브러리에는 RWLock 이 없어서 Lock 으로 대신합니다. (개념은 같음)
    data: list[int] = []
    lock = threading.Lock()

    def writer(i: int) -> None:
        with lock:
            data.append(i * 10)

    ws = [threading.Thread(target=writer, args=(i,)) for i in range(4)]
    for t in ws:
        t.start()
    for t in ws:
        t.join()

    sums = [0] * 3

    def reader(i: int) -> None:
        with lock:
            sums[i] = sum(data)

    rs = [threading.Thread(target=reader, args=(i,)) for i in range(3)]
    for t in rs:
        t.start()
    for t in rs:
        t.join()
    return sorted(data), sums


# ───────────── 6. 워커 풀 ─────────────
@dataclass
class Job:
    id: int
    lo: int
    hi: int


def worker_pool(jobs: list[Job], workers: int) -> list[tuple[int, int]]:
    job_q: queue.Queue[Job | None] = queue.Queue()
    result_q: queue.Queue[tuple[int, int]] = queue.Queue()
    for j in jobs:
        job_q.put(j)
    for _ in range(workers):
        job_q.put(None)  # 종료 신호 (Rust: 송신자를 drop 하면 recv 가 Err)

    def worker() -> None:
        while (job := job_q.get()) is not None:
            result_q.put((job.id, count_primes(job.lo, job.hi)))

    ts = [threading.Thread(target=worker) for _ in range(workers)]
    for t in ts:
        t.start()
    for t in ts:
        t.join()
    results = []
    while not result_q.empty():
        results.append(result_q.get())
    results.sort()
    return results


def main() -> None:
    print("== 1. spawn + join ==")
    print(f"squares: {parallel_squares([1, 2, 3, 4, 5, 6, 7, 8])}")

    print("== 2. scoped threads ==")
    data = list(range(1, 101))
    sums = scoped_chunk_sums(data, 4)
    print(f"chunk sums: {sums}")
    print(f"total: {sum(sums)}")

    print("== 3. Arc<Mutex> ==")
    print(f"counter: {counter_with_mutex(8, 1000)} (8 threads x 1000)")

    print("== 4. channel ==")
    for m in collect_messages(3, 2):
        print(f"  {m}")

    print("== 5. RwLock ==")
    written, seen = rwlock_demo()
    print(f"written: {written}")
    print(f"readers saw: {seen}")

    print("== 6. worker pool ==")
    jobs = [Job(i, i * 10000, (i + 1) * 10000) for i in range(10)]
    results = worker_pool(jobs, 4)
    for jid, n in results:
        job = jobs[jid]
        print(f"  job {jid}: [{job.lo}, {job.hi}) -> {n} primes")
    print(f"total primes below 100000: {sum(n for _, n in results)}")


if __name__ == "__main__":
    main()
