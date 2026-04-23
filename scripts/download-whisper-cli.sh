#!/usr/bin/env bash
# Obtains the whisper-cli sidecar for the host platform and places it at
# src-tauri/binaries/ with Tauri-sidecar naming (<name>-<target-triple>).
#
# Fetch strategy is per-platform because upstream ships wildly different
# assets per tag:
#   * Linux   → BUILD FROM SOURCE. whisper.cpp v1.7.6 releases ship no Linux
#               binary at all; the only *-x64.zip assets in the tag are
#               Windows. We clone the repo at the pinned tag and cmake a
#               statically-linked whisper-cli in a tempdir.
#   * macOS   → not shipped either (v1.7.6 only has the xcframework). Build
#               it yourself and drop into src-tauri/binaries/. Left as a
#               manual step until the darwin port lands in v0.3+.
#   * Windows → NOT handled here — use scripts/download-whisper-cli.ps1,
#               which grabs whisper-bin-x64.zip.
#
# Rationale for static linking on Linux (-DBUILD_SHARED_LIBS=OFF):
# BUILD_SHARED_LIBS=ON produces libwhisper.so + libggml*.so alongside the
# binary and requires you to ship them with a correct RPATH. Tauri's .deb
# bundler splits externalBin and resources across /usr/lib/<app>/ subdirs,
# so a simple `$ORIGIN` rpath wouldn't resolve. A single static binary
# sidesteps all of that for ~10 MB of extra size.

set -euo pipefail

WHISPER_VERSION="v1.7.6"
# whisper.cpp moved from ggerganov/whisper.cpp to ggml-org/whisper.cpp.
# The old URL redirects but we use the canonical form.
REPO="ggml-org/whisper.cpp"
BIN_DIR="$(dirname "$0")/../src-tauri/binaries"
mkdir -p "$BIN_DIR"

HOST="$(uname -s)-$(uname -m)"

case "$HOST" in
  Linux-x86_64)
    TARGET_TRIPLE="x86_64-unknown-linux-gnu"

    echo "Building whisper.cpp ${WHISPER_VERSION} from source (no upstream Linux binary available)."

    for tool in git cmake make cc; do
      if ! command -v "$tool" >/dev/null 2>&1; then
        echo "ERROR: missing required build tool: $tool" >&2
        echo "On Ubuntu/Debian: sudo apt install build-essential cmake git" >&2
        echo "On Fedora:        sudo dnf install gcc-c++ make cmake git" >&2
        echo "On Arch:          sudo pacman -S base-devel cmake git" >&2
        exit 1
      fi
    done

    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    echo "Cloning ${REPO} at tag ${WHISPER_VERSION}..."
    git clone \
      --depth 1 \
      --branch "${WHISPER_VERSION}" \
      "https://github.com/${REPO}.git" \
      "${TMP}/whisper.cpp"

    echo "Configuring cmake (Release, static, generic-CPU)..."
    # GGML_NATIVE=OFF disables -march=native so the resulting binary runs on
    # any x86_64 CPU, not just whatever the build host happened to be.
    cmake \
      -S "${TMP}/whisper.cpp" \
      -B "${TMP}/whisper.cpp/build" \
      -DCMAKE_BUILD_TYPE=Release \
      -DBUILD_SHARED_LIBS=OFF \
      -DGGML_NATIVE=OFF \
      -DWHISPER_BUILD_TESTS=OFF \
      -DWHISPER_BUILD_SERVER=OFF

    echo "Building whisper-cli..."
    cmake --build "${TMP}/whisper.cpp/build" \
      --config Release \
      --target whisper-cli \
      -j "$(nproc)"

    # Locate the built binary — path differs between whisper.cpp versions.
    SRC=""
    for candidate in \
        "${TMP}/whisper.cpp/build/bin/whisper-cli" \
        "${TMP}/whisper.cpp/build/whisper-cli" \
        "${TMP}/whisper.cpp/build/examples/cli/whisper-cli"; do
      if [[ -f "${candidate}" ]]; then SRC="${candidate}"; break; fi
    done
    if [[ -z "${SRC}" ]]; then
      echo "ERROR: could not find built whisper-cli binary." >&2
      echo "Build tree layout:" >&2
      find "${TMP}/whisper.cpp/build" -type f -name "whisper-cli*" >&2 || true
      exit 1
    fi

    DEST="${BIN_DIR}/whisper-cli-${TARGET_TRIPLE}"
    cp "${SRC}" "${DEST}"
    chmod +x "${DEST}"
    echo "Installed ${DEST}"

    # Belt-and-suspenders: if BUILD_SHARED_LIBS=OFF ended up missing some
    # internal sub-lib that still built as shared, copy any stray .so next
    # to the binary so Tauri's resources glob can pick them up.
    while IFS= read -r -d '' lib; do
      cp "${lib}" "${BIN_DIR}/"
      echo "Installed $(basename "${lib}") (unexpected shared lib from static build)"
    done < <(find "${TMP}/whisper.cpp/build" -type f -name "*.so*" -print0 2>/dev/null || true)

    # Sanity-check: the binary actually runs. If libstdc++ / libgomp etc.
    # are too new for the build host to satisfy, this surfaces immediately
    # rather than at user install time.
    if "${DEST}" --help >/dev/null 2>&1; then
      echo "whisper-cli sanity-check passed."
    else
      echo "Warning: ${DEST} did not execute cleanly. Run 'ldd ${DEST}' to inspect dependencies." >&2
    fi

    echo "Done."
    ;;

  Darwin-arm64|Darwin-x86_64)
    if [[ "${HOST}" == "Darwin-arm64" ]]; then
      TARGET_TRIPLE="aarch64-apple-darwin"
    else
      TARGET_TRIPLE="x86_64-apple-darwin"
    fi
    echo "ERROR: macOS is not yet supported for automated setup." >&2
    echo "whisper.cpp ${WHISPER_VERSION} does not ship a macOS CLI binary." >&2
    echo "To enable local development, build whisper-cli from source:" >&2
    echo "  git clone https://github.com/${REPO}" >&2
    echo "  cd whisper.cpp && git checkout ${WHISPER_VERSION}" >&2
    echo "  cmake -B build -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release" >&2
    echo "  cmake --build build --target whisper-cli -j" >&2
    echo "And place the resulting binary at:" >&2
    echo "  ${BIN_DIR}/whisper-cli-${TARGET_TRIPLE}" >&2
    exit 1
    ;;

  *)
    echo "Unsupported host platform: ${HOST}" >&2
    echo "Supported: Linux-x86_64, Darwin-arm64, Darwin-x86_64." >&2
    exit 1
    ;;
esac
