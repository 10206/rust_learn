"""Ch14. async/await 와 Tokio - Python(asyncio) 참조 구현.

Rust 버전(rust/examples/solution.rs)과 함수 이름이 1:1 로 대응됩니다.
Python 의 코루틴(coroutine) 객체도 Rust 의 Future 처럼 "만들기만 해서는 실행되지 않습니다".
"""

import asyncio
import time
from dataclasses import dataclass


@dataclass(frozen=True)
class Call:
    prompt: str
    latency_ms: int


class CallError(Exception):
    """Rust 의 enum CallError { Timeout(u64), Unavailable(u32) } 에 해당합니다."""


class Timeout(CallError):
    def __init__(self, limit_ms: int):
        super().__init__(f"timed out after {limit_ms}ms")


class Unavailable(CallError):
    def __init__(self, attempt: int):
        super().__init__(f"service unavailable (attempt {attempt})")


# TODO(1): 가짜 LLM 호출. latency 만큼 "비동기로" 기다린 뒤 답을 돌려줍니다.
async def fake_llm_call(call: Call) -> str:
    await asyncio.sleep(call.latency_ms / 1000)  # time.sleep 이면 이벤트 루프 전체가 멈춥니다!
    return f"answer to '{call.prompt}'"


# TODO(2): 하나씩 차례로 await (순차 실행). 총 시간 = 지연 시간의 "합".
async def run_sequential(calls: list[Call]) -> list[str]:
    results = []
    for call in calls:
        results.append(await fake_llm_call(call))
    return results


# TODO(3): 모두 동시에 실행. 총 시간 = 지연 시간의 "최댓값". 결과는 입력 순서 유지.
# Rust 에서는 JoinSet 으로 spawn 한 뒤 인덱스로 정렬합니다.
async def run_concurrent(calls: list[Call]) -> list[str]:
    return list(await asyncio.gather(*(fake_llm_call(c) for c in calls)))


# TODO(4): 두 호출을 동시에 실행해 튜플로 돌려줍니다 (Rust: tokio::join!).
async def join_pair(a: Call, b: Call) -> tuple[str, str]:
    ra, rb = await asyncio.gather(fake_llm_call(a), fake_llm_call(b))
    return ra, rb


# TODO(5): 제한 시간 안에 못 끝나면 Timeout. wait_for 는 시간 초과 시 작업을 "취소"합니다.
async def call_with_timeout(call: Call, limit_ms: int) -> str:
    try:
        return await asyncio.wait_for(fake_llm_call(call), limit_ms / 1000)
    except TimeoutError:
        raise Timeout(limit_ms) from None


async def fake_flaky(prompt: str, attempt: int) -> str:
    """처음 두 번은 실패하고 세 번째 시도부터 성공하는 불안정한 API."""
    await asyncio.sleep(0.010)
    if attempt < 3:
        raise Unavailable(attempt)
    return f"answer to '{prompt}' (attempt {attempt})"


# TODO(6): 실패하면 지수 백오프(base, 2*base, 4*base ...)로 최대 max_attempts 번 재시도.
async def retry(max_attempts: int, base_ms: int, op) -> str:
    attempt = 1
    while True:
        try:
            return await op(attempt)
        except CallError as e:
            print(f"  attempt {attempt} failed: {e}")
            if attempt >= max_attempts:
                raise
            await asyncio.sleep(base_ms * 2 ** (attempt - 1) / 1000)
            attempt += 1


# TODO(7): 먼저 끝나는 쪽의 결과를 돌려주고 진 쪽은 취소합니다 (Rust: tokio::select!).
async def race(a: Call, b: Call) -> str:
    tasks = [asyncio.create_task(fake_llm_call(a)), asyncio.create_task(fake_llm_call(b))]
    done, pending = await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
    for t in pending:
        t.cancel()  # 진 쪽 취소 (Rust 는 Future 를 drop 하면 자동 취소)
    await asyncio.gather(*pending, return_exceptions=True)
    return done.pop().result()


# TODO(8): CPU 를 오래 쓰는 동기 함수는 별도 스레드로 (Rust: spawn_blocking).
def sum_of_squares(n: int) -> int:
    return sum(i * i for i in range(1, n + 1))


async def blocking_sum(n: int) -> int:
    return await asyncio.to_thread(sum_of_squares, n)


async def main() -> None:
    print("== 1. futures are lazy ==")
    lazy_call = Call("lazy", 10)

    async def lazy_body() -> str:
        print("  [lazy] body is running")
        return await fake_llm_call(lazy_call)

    fut = lazy_body()  # 코루틴 객체만 만들어짐: 아직 아무것도 실행되지 않음
    print("future created, nothing has run yet")
    print("now awaiting the future")
    print(f"result: {await fut}")

    calls = [
        Call("summarize", 100),
        Call("translate", 200),
        Call("classify", 300),
        Call("extract", 150),
        Call("rewrite", 150),
    ]

    print("== 2. sequential ==")
    t0 = time.perf_counter()
    seq = await run_sequential(calls)
    seq_ms = (time.perf_counter() - t0) * 1000
    for i, r in enumerate(seq):
        print(f"[{i}] {r}")

    print("== 3. concurrent ==")
    t0 = time.perf_counter()
    conc = await run_concurrent(calls)
    conc_ms = (time.perf_counter() - t0) * 1000
    for i, r in enumerate(conc):
        print(f"[{i}] {r}")
    # 순차 합계 900ms vs 동시 최댓값 300ms: 실제 시간은 출력하지 않고 큰 마진으로만 비교합니다.
    print(f"concurrent faster: {'yes' if conc_ms * 2 < seq_ms else 'no'}")

    print("== 4. join! ==")
    ra, rb = await join_pair(Call("a", 100), Call("b", 200))
    print(f"pair: {ra} | {rb}")

    print("== 5. timeout ==")
    for call, limit in [(Call("slow", 500), 150), (Call("quick", 50), 150)]:
        try:
            ok = await call_with_timeout(call, limit)
            print(f"{call.prompt} ({call.latency_ms}ms, limit {limit}ms): ok: {ok}")
        except CallError as e:
            print(f"{call.prompt} ({call.latency_ms}ms, limit {limit}ms): error: {e}")

    print("== 6. retry with backoff ==")
    for max_attempts in (5, 2):
        try:
            ok = await retry(max_attempts, 10, lambda n: fake_flaky("flaky", n))
            print(f"retry (max {max_attempts}): ok: {ok}")
        except CallError as e:
            print(f"retry (max {max_attempts}): error: {e}")

    print("== 7. select (race) ==")
    print(f"winner: {await race(Call('fast_model', 50), Call('slow_model', 300))}")

    print("== 8. blocking work ==")
    n = 1_000_000
    print(f"sum of squares 1..={n}: {await blocking_sum(n)}")


if __name__ == "__main__":
    asyncio.run(main())
