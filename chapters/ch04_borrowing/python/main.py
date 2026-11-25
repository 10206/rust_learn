"""Ch04. 텍스트 분석기 - Python 참조 구현.

Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
Python 의 str 은 불변(immutable)이고 슬라이스(s[a:b])는 항상 "새 문자열"을 만듭니다(복사).
Rust 의 &str 슬라이스는 원본의 일부를 가리키는 "뷰"(포인터 + 길이)라서 복사가 없어요.
Python 의 len(str) 은 "글자(코드포인트) 수", Rust 의 str::len() 은 "바이트 수" 입니다. (아래 UTF-8 절)
"""
import sys

# 한글이 stdout 에 정확히 UTF-8 로 나가도록 고정 (Rust / C++ 출력과 바이트 단위로 같아야 해요)
sys.stdout.reconfigure(encoding="utf-8")

TEXT = "The quick brown fox jumps over the lazy dog. It was a sunny day! Did you enjoy the afternoon?"
KOREAN = "안녕하세요 러스트"


def word_count(text: str) -> int:
    return len(text.split())


def longest_word(text: str) -> str:
    """구두점을 뗀 가장 긴 단어. 길이가 같으면 먼저 나온 단어. (Rust 는 원본의 일부를 &str 로 돌려줘요)"""
    punct = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
    best = ""
    for w in text.split():
        w = w.strip(punct)
        if len(w) > len(best):  # 엄격히 클 때만 갱신 -> 동률이면 먼저 나온 단어 유지
            best = w
    return best


def first_sentence(text: str) -> str:
    """처음 나오는 . ! ? 까지(그 문자 포함). 없으면 전체."""
    ends = [i for i in (text.find("."), text.find("!"), text.find("?")) if i != -1]
    if not ends:
        return text
    return text[: min(ends) + 1]


def normalize(s: str) -> str:
    """앞뒤 공백 제거 + 연속 공백을 하나로 + 소문자.
    Python 문자열은 불변이라 '제자리 수정'이 불가능해서 새 문자열을 돌려줍니다.
    (Rust 버전은 &mut String 을 받아 그 자리에서 고칩니다.)"""
    return " ".join(s.split()).lower()


def scale_in_place(nums: list[int], k: int) -> None:
    # Python 리스트는 가변이라 함수 안에서 고치면 호출한 쪽에도 보입니다.
    # (호출부만 봐서는 "이 함수가 내 리스트를 바꾼다"는 사실이 드러나지 않죠. Rust 는 &mut 로 드러납니다.)
    for i in range(len(nums)):
        nums[i] *= k


def sum_slice(nums: list[int]) -> int:
    return sum(nums)


def char_count(s: str) -> int:
    return len(s)


def first_n_chars(s: str, n: int) -> str:
    return s[:n]


def reverse_chars(s: str) -> str:
    return s[::-1]


def hex_bytes(s: str) -> str:
    return " ".join(f"{b:02x}" for b in s.encode("utf-8"))


def main() -> None:
    print("== 1. text stats ==")
    print(f"text: {TEXT}")
    print(f"words: {word_count(TEXT)}")
    print(f"longest: {longest_word(TEXT)}")
    print(f"longest (tie): {longest_word('one two six')}")
    print(f"first sentence: {first_sentence(TEXT)}")
    print(f"first sentence (no punctuation): {first_sentence('hello there')}")

    print("== 2. normalize in place ==")
    s = "  Hello,   WORLD   from  Rust!  "
    print(f'before: "{s}"')
    s = normalize(s)
    print(f'after: "{s}"')

    print("== 3. slices of numbers ==")
    nums = [1, 2, 3, 4, 5]
    print("nums: [" + ", ".join(map(str, nums)) + "]")
    print(f"sum: {sum_slice(nums)}")
    scale_in_place(nums, 3)
    print("after scale x3: [" + ", ".join(map(str, nums)) + "]")
    print(f"first three sum: {sum_slice(nums[:3])}")
    print(f"last two sum: {sum_slice(nums[3:])}")

    print("== 4. UTF-8 ==")
    print(f"text: {KOREAN}")
    print(f"words: {word_count(KOREAN)}")
    print(f"bytes: {len(KOREAN.encode('utf-8'))}")
    print(f"chars: {char_count(KOREAN)}")
    print(f"first 3 chars: {first_n_chars(KOREAN, 3)}")
    # Rust: &text[0..3] 는 "바이트 0~3" -> '안' 한 글자 (한글 한 글자는 UTF-8 로 3바이트)
    print(f"first byte-slice [0..3]: {KOREAN.encode('utf-8')[0:3].decode('utf-8')}")
    print(f"reversed: {reverse_chars(KOREAN)}")
    print(f"bytes of 가: {hex_bytes('가')}")


if __name__ == "__main__":
    main()
