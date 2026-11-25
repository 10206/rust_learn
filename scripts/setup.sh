#!/usr/bin/env bash
# 학습에 필요한 도구를 설치합니다. 여러 번 실행해도 안전해요.
set -u

echo "== rust_learn 환경 설정 =="
OS="$(uname -s)"

if [ "$OS" = "Darwin" ]; then
  echo "macOS 에서는 아래 명령을 직접 실행해 주세요 (자동 설치는 하지 않아요):"
  echo "  xcode-select --install          # clang/g++ (C++20 지원 버전 필요)"
  echo "  brew install nlohmann-json curl python"
  echo "  pip3 install anthropic"
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo "  rustup component add rustfmt clippy"
  exit 0
fi

SUDO=""
if [ "$(id -u)" != "0" ]; then
  if command -v sudo >/dev/null 2>&1; then SUDO="sudo"; else
    echo "root 권한(sudo)이 필요해요. 다시 실행해 주세요." >&2; exit 1
  fi
fi

if command -v apt-get >/dev/null 2>&1; then
  echo "-- apt 패키지 설치 (g++, nlohmann-json3-dev, libcurl4-openssl-dev, python3, pip)"
  $SUDO apt-get update -y || echo "apt-get update 실패 (계속 진행해요)"
  $SUDO apt-get install -y g++ build-essential nlohmann-json3-dev libcurl4-openssl-dev \
    python3 python3-pip curl ca-certificates || echo "일부 apt 패키지 설치에 실패했어요."
else
  echo "apt-get 이 없는 배포판이에요. g++, nlohmann-json, libcurl 개발 패키지, python3, pip 를 직접 설치해 주세요."
fi

echo "-- Python 'anthropic' 패키지"
if python3 -c 'import anthropic' >/dev/null 2>&1; then
  echo "이미 설치되어 있어요."
else
  python3 -m pip install anthropic 2>/dev/null \
    || python3 -m pip install --user anthropic 2>/dev/null \
    || python3 -m pip install --break-system-packages anthropic \
    || echo "anthropic 설치에 실패했어요 (ch17 Python 예제에만 필요해요)."
fi

echo "-- Rust"
if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "rustup 으로 Rust 를 설치해요..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
  echo "새 터미널을 열거나  source \"\$HOME/.cargo/env\"  를 실행하세요."
else
  echo "cargo 가 이미 있어요: $(cargo --version)"
fi
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy
else
  echo "rustup 이 없어서 rustfmt/clippy 컴포넌트는 건너뛰었어요."
fi

echo "완료! 이제 scripts/compare.sh ch01 처럼 실행해 보세요."
