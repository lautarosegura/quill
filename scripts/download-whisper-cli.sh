#!/usr/bin/env bash
# Downloads pre-built whisper-cli binaries from ggerganov/whisper.cpp releases
# and places them into src-tauri/binaries/ with Tauri-sidecar naming:
#   whisper-cli-<target-triple>[.exe]
#
# Supported host platforms for this script: macOS (Intel + Apple Silicon), Linux.
# For Windows, use the .ps1 variant.

set -euo pipefail

# Pin a specific whisper.cpp release for reproducibility.
# Update this when you want to bump the version.
WHISPER_VERSION="v1.7.6"

REPO="ggerganov/whisper.cpp"
BIN_DIR="$(dirname "$0")/../src-tauri/binaries"
mkdir -p "$BIN_DIR"

echo "Downloading whisper.cpp ${WHISPER_VERSION} prebuilts..."

# Release asset naming for whisper.cpp varies per version. This mapping pulls
# the right asset per target. If the upstream renames assets, update here.
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    ASSET="whisper-bin-macos-arm64.zip"
    TARGET_TRIPLE="aarch64-apple-darwin"
    ;;
  Darwin-x86_64)
    ASSET="whisper-bin-macos-x64.zip"
    TARGET_TRIPLE="x86_64-apple-darwin"
    ;;
  Linux-x86_64)
    ASSET="whisper-bin-linux-x64.zip"
    TARGET_TRIPLE="x86_64-unknown-linux-gnu"
    ;;
  *)
    echo "Unsupported host: $(uname -s)-$(uname -m)" >&2
    echo "Build whisper-cli from source and copy into src-tauri/binaries/" >&2
    exit 1
    ;;
esac

TMP=$(mktemp -d)
URL="https://github.com/${REPO}/releases/download/${WHISPER_VERSION}/${ASSET}"

echo "Fetching ${URL}"
curl -fL "${URL}" -o "${TMP}/${ASSET}"
unzip -o "${TMP}/${ASSET}" -d "${TMP}"

# Find whisper-cli (the transcription tool — NOT main which is a stub).
# Check common layouts across whisper.cpp versions.
SRC=""
for candidate in \
    "${TMP}/whisper-cli" \
    "${TMP}/bin/whisper-cli" \
    "${TMP}/build/bin/whisper-cli" \
    "${TMP}/Release/whisper-cli"; do
  if [[ -f "${candidate}" ]]; then SRC="${candidate}"; break; fi
done
if [[ -z "${SRC}" ]]; then
  echo "Could not find whisper-cli binary in archive." >&2
  echo "Archive contents:" >&2
  find "${TMP}" -type f >&2
  exit 1
fi

DEST="${BIN_DIR}/whisper-cli-${TARGET_TRIPLE}"
cp "${SRC}" "${DEST}"
chmod +x "${DEST}"
echo "Installed ${DEST}"

# On macOS the whisper-cli bundle typically ships as a single statically-linked
# binary. On Linux it may ship with *.so files — copy them alongside if present.
BIN_SRC_DIR="$(dirname "${SRC}")"
for lib in "${BIN_SRC_DIR}"/*.so "${BIN_SRC_DIR}"/*.dylib; do
  if [[ -f "${lib}" ]]; then
    cp "${lib}" "${BIN_DIR}/"
    echo "Installed ${BIN_DIR}/$(basename "${lib}")"
  fi
done

rm -rf "${TMP}"
echo "Done."
