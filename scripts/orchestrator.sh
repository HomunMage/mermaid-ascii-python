#!/bin/bash
# orchestrator.sh — Continuous agent loop running in tmux
# Spawns worker in the same tmux session, monitors via trigger file

cd "$(dirname "$0")/.." || exit 1
PROJECT_DIR="$(pwd)"
TRIGGER_FILE="${PROJECT_DIR}/_trigger"
LOG_FILE="${PROJECT_DIR}/out/orchestrator.log"
MAX_CYCLES="${1:-50}"
CYCLE=0

# Source Rust
. "$HOME/.cargo/env" 2>/dev/null || true

mkdir -p out

log() {
  echo "$(date '+%H:%M:%S') $1" | tee -a "$LOG_FILE"
}

log "========================================"
log "Orchestrator started. Max cycles: ${MAX_CYCLES}"
log "========================================"

while [ "$CYCLE" -lt "$MAX_CYCLES" ]; do
  CYCLE=$((CYCLE + 1))
  log ""
  log "--- Cycle ${CYCLE}/${MAX_CYCLES} ---"

  # Clean trigger file
  rm -f "$TRIGGER_FILE"

  # Run worker directly (we're already in tmux)
  log "Starting worker..."
  bash "${PROJECT_DIR}/scripts/worker.sh"
  log "Worker exited."

  # Check trigger result
  if [ -f "$TRIGGER_FILE" ]; then
    RESULT=$(cat "$TRIGGER_FILE")
    log "Worker result: ${RESULT}"

    case "$RESULT" in
      DONE)
        log "Cycle complete. Starting next cycle..."
        ;;
      BLOCKED)
        log "Worker BLOCKED. Waiting 30s then retrying..."
        sleep 30
        ;;
      ALL_COMPLETE)
        log "ALL PHASES COMPLETE!"
        log "Project is done. Check out/ for results."
        exit 0
        ;;
      *)
        log "Unknown result: ${RESULT}. Continuing..."
        ;;
    esac
  else
    log "No trigger file. Worker may have hit an issue. Retrying in 10s..."
    sleep 10
  fi

  # Brief pause between cycles
  sleep 3
done

log "Max cycles (${MAX_CYCLES}) reached."
log "Check llm.working.status for current progress."
