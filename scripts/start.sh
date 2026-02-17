#!/bin/bash
# start.sh — Start the orchestrator in a tmux session
# Usage: bash scripts/start.sh [max_cycles]
# Monitor: tmux attach -t text-graph
# Stop: bash scripts/stop.sh

cd "$(dirname "$0")/.." || exit 1
PROJECT_DIR="$(pwd)"
MAX_CYCLES="${1:-50}"
SESSION="text-graph"

# Kill existing session if any
tmux kill-session -t "$SESSION" 2>/dev/null

# Create tmux session with orchestrator in window 0
tmux new-session -d -s "$SESSION" -n "orchestrator" \
  "cd ${PROJECT_DIR} && bash scripts/orchestrator.sh ${MAX_CYCLES}; read"

echo "========================================="
echo " text-graph autonomous agents started"
echo "========================================="
echo ""
echo " tmux session: ${SESSION}"
echo " Monitor:      tmux attach -t ${SESSION}"
echo " Stop:         bash scripts/stop.sh"
echo " Max cycles:   ${MAX_CYCLES}"
echo ""
echo " Window layout:"
echo "   0: orchestrator — main control loop"
echo "   Workers spawn as needed in the same window"
echo ""
