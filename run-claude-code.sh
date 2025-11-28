#!/usr/bin/env bash
#
# run-claude-code.sh - Autonomous Claude Code runner for Apollo
#
# This script runs Claude Code in a loop, handling sessions and ensuring
# regular commits and pushes.
#
# Usage:
#   ./run-claude-code.sh [options]
#
# Options:
#   --max-sessions N    Maximum number of sessions to run (default: unlimited)
#   --session-timeout S Timeout per session in seconds (default: 3600 = 1 hour)
#   --cooldown S        Cooldown between sessions in seconds (default: 60)
#   --dry-run           Print what would be done without running
#

set -euo pipefail

# Configuration
PROJECT_DIR="${PROJECT_DIR:-$(pwd)}"
MAX_SESSIONS="${MAX_SESSIONS:-3}"  # 0 = unlimited
SESSION_TIMEOUT="${SESSION_TIMEOUT:-1800}"  # 1 hour
COOLDOWN="${COOLDOWN:-5}"  # 1 minute between sessions
DRY_RUN="${DRY_RUN:-false}"
LOG_DIR="${LOG_DIR:-$HOME/.claude-logs}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $*"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --max-sessions)
                MAX_SESSIONS="$2"
                shift 2
                ;;
            --session-timeout)
                SESSION_TIMEOUT="$2"
                shift 2
                ;;
            --cooldown)
                COOLDOWN="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

show_help() {
    cat << EOF
Usage: $0 [options]

Options:
  --max-sessions N    Maximum number of sessions to run (default: unlimited)
  --session-timeout S Timeout per session in seconds (default: 3600)
  --cooldown S        Cooldown between sessions in seconds (default: 60)
  --dry-run           Print what would be done without running
  --help, -h          Show this help message

Environment variables:
  PROJECT_DIR         Project directory (default: current directory)
  LOG_DIR             Directory for session logs (default: .claude-logs)
EOF
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v claude &> /dev/null; then
        log_error "Claude Code CLI not found. Install it first."
        exit 1
    fi

    if ! command -v git &> /dev/null; then
        log_error "Git not found. Install it first."
        exit 1
    fi

    if [[ ! -f "$PROJECT_DIR/PROMPT.md" ]]; then
        log_error "PROMPT.md not found in $PROJECT_DIR"
        exit 1
    fi

    if [[ ! -f "$PROJECT_DIR/CLAUDE.md" ]]; then
        log_warn "CLAUDE.md not found - Claude Code may not have project context"
    fi

    log_success "Prerequisites OK"
}

# Ensure we're on a clean git state before starting
ensure_clean_git() {
    cd "$PROJECT_DIR"
    
    # Check if there are uncommitted changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
        log_warn "Uncommitted changes detected, committing..."
        git add -A
        git commit -m "chore: auto-commit before Claude Code session" || true
    fi
    
    # Try to push any pending commits
    git push origin HEAD 2>/dev/null || log_warn "Could not push to remote"
}

# Run a single Claude Code session
run_session() {
    local session_num=$1
    local session_id
    session_id=$(date '+%Y%m%d_%H%M%S')
    local log_file="$LOG_DIR/session_${session_id}.log"
    
    log_info "Starting session #$session_num (ID: $session_id)"
    
    # Create log directory if needed
    mkdir -p "$LOG_DIR"
    
    cd "$PROJECT_DIR"
    
    # Build the prompt for Claude Code
    local prompt
    prompt=$(cat << 'PROMPT_END'
Read PROMPT.md and CLAUDE.md for project context and instructions.
Then check TASKS.md for your current task and continue working.
Remember to:
1. Commit and push at least every 30 minutes
2. Log decisions to DECISIONS_NEEDED.md instead of blocking
3. Update SESSION_LOG.md with your progress
4. Run cargo fmt, clippy, and tests before committing
PROMPT_END
)

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would run: claude \"$prompt\""
        return 0
    fi
    
    # Run Claude Code with timeout
    log_info "Running Claude Code (timeout: ${SESSION_TIMEOUT}s)..."
    
    # Use timeout and capture exit code
    local exit_code=0
    timeout "$SESSION_TIMEOUT" claude --dangerously-skip-permissions "$prompt" 2>&1 | tee "$log_file" || exit_code=$?
    
    case $exit_code in
        0)
            log_success "Session completed successfully"
            ;;
        124)
            log_warn "Session timed out after ${SESSION_TIMEOUT}s"
            # Force a commit on timeout
            force_commit_and_push
            ;;
        *)
            log_error "Session exited with code $exit_code"
            # Still try to save work
            force_commit_and_push
            ;;
    esac
    
    return $exit_code
}

# Force commit and push any pending work
force_commit_and_push() {
    cd "$PROJECT_DIR"
    
    if ! git diff --quiet || ! git diff --cached --quiet; then
        log_info "Committing pending changes..."
        git add -A
        git commit -m "wip: auto-save after session interruption" || true
        git push origin HEAD 2>/dev/null || log_warn "Could not push"
    fi
}

# Check if we should continue running
should_continue() {
    local session_num=$1
    
    # Check max sessions limit
    if [[ $MAX_SESSIONS -gt 0 && $session_num -ge $MAX_SESSIONS ]]; then
        log_info "Reached maximum sessions ($MAX_SESSIONS)"
        return 1
    fi
    
    # Check for stop file
    if [[ -f "$PROJECT_DIR/.claude-stop" ]]; then
        log_info "Stop file detected (.claude-stop)"
        rm -f "$PROJECT_DIR/.claude-stop"
        return 1
    fi
    
    # Check if all tasks are done (basic check)
    if grep -q '^\- \[TODO\]' "$PROJECT_DIR/TASKS.md" 2>/dev/null; then
        return 0  # There are still TODO tasks
    fi
    
    if grep -q '^\- \[IN PROGRESS\]' "$PROJECT_DIR/TASKS.md" 2>/dev/null; then
        return 0  # There are still in-progress tasks
    fi
    
    log_info "No remaining tasks found in TASKS.md"
    return 1
}

# Main loop
main() {
    parse_args "$@"
    
    log_info "=== Apollo Claude Code Runner ==="
    log_info "Project: $PROJECT_DIR"
    log_info "Max sessions: ${MAX_SESSIONS:-unlimited}"
    log_info "Session timeout: ${SESSION_TIMEOUT}s"
    log_info "Cooldown: ${COOLDOWN}s"
    
    check_prerequisites
    ensure_clean_git
    
    local session_num=0
    
    while should_continue $session_num; do
        session_num=$((session_num + 1))
        
        run_session $session_num || true  # Don't exit on session failure
        
        if should_continue $session_num; then
            log_info "Cooling down for ${COOLDOWN}s before next session..."
            sleep "$COOLDOWN"
        fi
    done
    
    log_success "=== Runner finished after $session_num sessions ==="
}

# Run main with all arguments
main "$@"
