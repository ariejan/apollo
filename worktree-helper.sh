#!/usr/bin/env bash
#
# worktree-helper.sh - Git worktree management for parallel development
#
# Usage:
#   ./worktree-helper.sh create <feature-name>   Create a new worktree
#   ./worktree-helper.sh list                    List all worktrees
#   ./worktree-helper.sh switch <feature-name>   Switch to a worktree
#   ./worktree-helper.sh finish <feature-name>   Merge and cleanup worktree
#   ./worktree-helper.sh cleanup                 Remove stale worktrees
#

set -euo pipefail

MAIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_BASE="$(dirname "$MAIN_DIR")"

cmd_create() {
    local feature_name="$1"
    local worktree_path="$WORKTREE_BASE/apollo-$feature_name"
    local branch_name="feature/$feature_name"
    
    if [[ -d "$worktree_path" ]]; then
        echo "Error: Worktree already exists at $worktree_path"
        exit 1
    fi
    
    cd "$MAIN_DIR"
    
    echo "Creating worktree for $feature_name..."
    git worktree add "$worktree_path" -b "$branch_name"
    
    echo "✓ Created worktree at: $worktree_path"
    echo "✓ Branch: $branch_name"
    echo ""
    echo "To work on this feature:"
    echo "  cd $worktree_path"
}

cmd_list() {
    cd "$MAIN_DIR"
    echo "Active worktrees:"
    echo ""
    git worktree list
}

cmd_switch() {
    local feature_name="$1"
    local worktree_path="$WORKTREE_BASE/apollo-$feature_name"
    
    if [[ ! -d "$worktree_path" ]]; then
        echo "Error: Worktree not found at $worktree_path"
        echo "Create it first with: $0 create $feature_name"
        exit 1
    fi
    
    cd "$worktree_path"
    echo "Switched to: $worktree_path"
    exec $SHELL
}

cmd_finish() {
    local feature_name="$1"
    local worktree_path="$WORKTREE_BASE/apollo-$feature_name"
    local branch_name="feature/$feature_name"
    
    if [[ ! -d "$worktree_path" ]]; then
        echo "Error: Worktree not found at $worktree_path"
        exit 1
    fi
    
    # Ensure worktree has no uncommitted changes
    cd "$worktree_path"
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "Error: Worktree has uncommitted changes"
        echo "Commit or stash them first"
        exit 1
    fi
    
    # Push the branch
    echo "Pushing branch $branch_name..."
    git push origin "$branch_name"
    
    # Go back to main directory
    cd "$MAIN_DIR"
    
    # Remove worktree
    echo "Removing worktree..."
    git worktree remove "$worktree_path"
    
    echo "✓ Worktree removed"
    echo ""
    echo "To merge the feature:"
    echo "  git checkout main"
    echo "  git merge $branch_name"
    echo "  git push origin main"
    echo "  git branch -d $branch_name"
}

cmd_cleanup() {
    cd "$MAIN_DIR"
    echo "Pruning stale worktrees..."
    git worktree prune -v
    echo "✓ Cleanup complete"
}

show_help() {
    cat << EOF
Git Worktree Helper for Apollo

Usage: $0 <command> [arguments]

Commands:
  create <name>   Create a new worktree for a feature
  list            List all active worktrees
  switch <name>   Open a shell in the specified worktree
  finish <name>   Push and remove a worktree (for completed features)
  cleanup         Remove stale worktree references

Examples:
  $0 create musicbrainz-client
  $0 switch musicbrainz-client
  $0 finish musicbrainz-client
EOF
}

# Main
case "${1:-}" in
    create)
        [[ -z "${2:-}" ]] && { echo "Error: Feature name required"; exit 1; }
        cmd_create "$2"
        ;;
    list)
        cmd_list
        ;;
    switch)
        [[ -z "${2:-}" ]] && { echo "Error: Feature name required"; exit 1; }
        cmd_switch "$2"
        ;;
    finish)
        [[ -z "${2:-}" ]] && { echo "Error: Feature name required"; exit 1; }
        cmd_finish "$2"
        ;;
    cleanup)
        cmd_cleanup
        ;;
    help|--help|-h|"")
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
