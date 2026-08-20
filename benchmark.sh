#!/usr/bin/env bash
set -euo pipefail

BIN="${BIN:-./target/release/rustbin}"
PORT="${PORT:-8080}"
VUS="${VUS:-500}"
DURATION="${DURATION:-60s}"
FILE_SIZE="${FILE_SIZE:-16384}"

LIMITS="${LIMITS:-32 64 128 256 512}"
RESULTS_DIR="${RESULTS_DIR:-benchmark-results}"

mkdir -p "$RESULTS_DIR"

if ss -ltn 2>/dev/null | grep -q ":${PORT} "; then
    echo "ERROR: port ${PORT} is already in use."
    echo "Stop the existing server or choose another PORT."
    exit 1
fi

for LIMIT in $LIMITS; do
    echo
    echo "Upload limit:   ${LIMIT}"
    echo "Download limit: ${LIMIT}"
    echo "k6 VUs:         ${VUS}"
    echo "Duration:       ${DURATION}"
    echo "File size:      ${FILE_SIZE} bytes"

    SERVER_LOG="${RESULTS_DIR}/server_${LIMIT}.log"
    K6_LOG="${RESULTS_DIR}/k6_${LIMIT}.log"
    K6_JSON="${RESULTS_DIR}/k6_${LIMIT}.json"
    RESOURCE_LOG="${RESULTS_DIR}/resources_${LIMIT}.csv"

    PORT="$PORT"     UPLOAD_LIMIT="$LIMIT"     DOWNLOAD_LIMIT="$LIMIT"     "$BIN" >"$SERVER_LOG" 2>&1 &

    SERVER_PID=$!
    RESOURCE_PID=""
    K6_STATUS=0

    cleanup() {
        if [[ -n "${K6_PID:-}" ]] && kill -0 "$K6_PID" 2>/dev/null; then
            kill "$K6_PID" 2>/dev/null || true
        fi
        if [[ -n "${RESOURCE_PID:-}" ]] && kill -0 "$RESOURCE_PID" 2>/dev/null; then
            kill "$RESOURCE_PID" 2>/dev/null || true
        fi
        if kill -0 "$SERVER_PID" 2>/dev/null; then
            kill "$SERVER_PID" 2>/dev/null || true
        fi
        wait "$SERVER_PID" 2>/dev/null || true
    }

    trap cleanup EXIT INT TERM

    READY=0
    for _ in $(seq 1 50); do
        if curl -fsS "http://127.0.0.1:${PORT}/robots.txt" >/dev/null 2>&1; then
            READY=1
            break
        fi

        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo "ERROR: rustbin exited during startup."
            cat "$SERVER_LOG"
            exit 1
        fi
        sleep 0.1
    done

    if [[ "$READY" -ne 1 ]]; then
        echo "ERROR: rustbin did not become ready."
        cat "$SERVER_LOG"
        exit 1
    fi

    echo "Rustbin PID: ${SERVER_PID}"

    echo "timestamp,cpu_percent,rss_kb" >"$RESOURCE_LOG"

    (
        while kill -0 "$SERVER_PID" 2>/dev/null; do
            timestamp="$(date +%s)"
            cpu="$(ps -p "$SERVER_PID" -o %cpu= | awk '{print $1}')"
            rss="$(ps -p "$SERVER_PID" -o rss= | awk '{print $1}')"

            if [[ -n "$cpu" && -n "$rss" ]]; then
                echo "${timestamp},${cpu},${rss}" >>"$RESOURCE_LOG"
            fi
            sleep 1
        done
    ) &
    RESOURCE_PID=$!

    set +e
    BASE_URL="http://127.0.0.1:${PORT}"     VUS="$VUS"     DURATION="$DURATION"     FILE_SIZE="$FILE_SIZE"     k6 run         --summary-export="$K6_JSON"         benchmark.js         >"$K6_LOG" 2>&1
    K6_STATUS=$?
    set -e

    if [[ -n "$RESOURCE_PID" ]] && kill -0 "$RESOURCE_PID" 2>/dev/null; then
        kill "$RESOURCE_PID" 2>/dev/null || true
        wait "$RESOURCE_PID" 2>/dev/null || true
    fi

    if [[ "$K6_STATUS" -ne 0 ]]; then
        echo
        echo "ERROR: k6 failed for server limit ${LIMIT}"
        cat "$K6_LOG"
        exit "$K6_STATUS"
    fi

    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true

    echo
    echo "Completed limit=${LIMIT}"
    echo "  k6 summary: ${K6_JSON}"
    echo "  k6 log:     ${K6_LOG}"
    echo "  CPU/RSS:    ${RESOURCE_LOG}"
    echo "  server log: ${SERVER_LOG}"

    trap - EXIT INT TERM
done

echo
echo "All benchmark runs completed."
