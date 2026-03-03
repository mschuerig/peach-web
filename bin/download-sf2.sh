#!/bin/bash
#
# download-sf2.sh — Download the GeneralUser GS SF2 SoundFont for Peach.
#
# Run this script once before your first build:
#   ./bin/download-sf2.sh
#
# The SF2 file is placed at .cache/GeneralUser-GS.sf2 in the project root.
# Trunk copies this file into the dist/ bundle at build time.
#
# Dependencies: curl, shasum (stock macOS)

set -euo pipefail

# --- Configuration -----------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFIG_FILE="${SCRIPT_DIR}/sf2-sources.conf"
CACHE_DIR="${PROJECT_ROOT}/.cache"

if [ ! -f "${CONFIG_FILE}" ]; then
    echo "error: SF2 config file not found: ${CONFIG_FILE}" >&2
    echo "note: Ensure bin/sf2-sources.conf exists in the project." >&2
    exit 1
fi

# Parse key=value config (ignoring comments and blank lines)
DOWNLOAD_URL=$(grep '^url=' "${CONFIG_FILE}" | cut -d'=' -f2-)
SF2_FILENAME=$(grep '^filename=' "${CONFIG_FILE}" | cut -d'=' -f2-)
EXPECTED_SHA256=$(grep '^sha256=' "${CONFIG_FILE}" | cut -d'=' -f2-)

if [ -z "${DOWNLOAD_URL}" ] || [ -z "${SF2_FILENAME}" ] || [ -z "${EXPECTED_SHA256}" ]; then
    echo "error: Missing required fields in ${CONFIG_FILE}" >&2
    echo "note: Required fields: url, filename, sha256" >&2
    exit 1
fi

CACHED_SF2="${CACHE_DIR}/${SF2_FILENAME}"
TEMP_FILE="${CACHE_DIR}/${SF2_FILENAME}.download"

# --- Cleanup trap ------------------------------------------------------------

cleanup() {
    rm -f "${TEMP_FILE}"
}
trap cleanup EXIT

# --- Helper functions --------------------------------------------------------

verify_checksum() {
    local file="$1"
    local actual
    actual=$(shasum -a 256 "${file}" | awk '{print $1}')
    if [ "${actual}" = "${EXPECTED_SHA256}" ]; then
        return 0
    else
        echo "warning: Checksum mismatch for ${file}" >&2
        echo "  expected: ${EXPECTED_SHA256}" >&2
        echo "  actual:   ${actual}" >&2
        return 1
    fi
}

download_sf2() {
    echo "Downloading ${SF2_FILENAME}..."
    if ! curl -L -f --retry 3 --silent --show-error -o "${TEMP_FILE}" "${DOWNLOAD_URL}"; then
        echo "error: Download failed. Check your network connection." >&2
        echo "note: URL: ${DOWNLOAD_URL}" >&2
        echo "note: You can manually place the file at ${CACHED_SF2}" >&2
        exit 1
    fi

    # Verify download is not an HTML error page
    if file "${TEMP_FILE}" | grep -qi "HTML"; then
        rm -f "${TEMP_FILE}"
        echo "error: Download returned an HTML page instead of an SF2 file." >&2
        echo "note: The download URL may have changed. Check bin/sf2-sources.conf." >&2
        exit 1
    fi

    mv "${TEMP_FILE}" "${CACHED_SF2}"
    echo "Download complete."
}

# --- Main --------------------------------------------------------------------

mkdir -p "${CACHE_DIR}"

# Check if cached file exists and has correct checksum
if [ -f "${CACHED_SF2}" ]; then
    if verify_checksum "${CACHED_SF2}"; then
        echo "SF2 file is up to date: ${CACHED_SF2}"
        exit 0
    else
        echo "Cached file has incorrect checksum. Re-downloading..."
        rm -f "${CACHED_SF2}"
    fi
else
    echo "No cached SF2 found. Downloading..."
fi

download_sf2

if ! verify_checksum "${CACHED_SF2}"; then
    rm -f "${CACHED_SF2}"
    echo "error: Downloaded file has incorrect checksum." >&2
    echo "note: The expected checksum in bin/sf2-sources.conf may need updating." >&2
    exit 1
fi

echo "SF2 setup complete: ${CACHED_SF2}"
