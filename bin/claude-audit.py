#!/usr/bin/env python3
"""
claude-audit: Match Claude Code sessions with git commits to produce an audit trail.

Scans the git history of the current repo, finds Claude Code session transcripts
that overlap with each commit's timestamp, and produces a per-commit markdown
file containing the conversation and (optionally) tool calls, thinking, and
tool output.

Output goes to .claude-audit/ at the repo root.

Usage:
    python claude-audit.py [--repo-path PATH] [--output-dir DIR] [--author AUTHOR]
                           [--detail minimal|standard|full]
"""

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def get_repo_root(repo_path: str | None = None) -> Path:
    """Get the git repository root directory."""
    cmd = ["git", "rev-parse", "--show-toplevel"]
    cwd = repo_path or os.getcwd()
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=cwd)
    if result.returncode != 0:
        print(f"Error: not a git repository (or any parent): {cwd}", file=sys.stderr)
        sys.exit(1)
    return Path(result.stdout.strip())


def get_git_commits(repo_root: Path, author: str | None = None) -> list[dict]:
    """Get all commits from git log with hash, timestamp, author, subject, and full body."""
    # Use a record separator (%x1e) between commits and field separator (%x1f) between fields.
    # %B = full commit message (subject + body), which can contain newlines.
    RS = "\x1e"  # record separator
    FS = "\x1f"  # field separator
    fmt = f"%H{FS}%aI{FS}%an{FS}%ae{FS}%s{FS}%B{RS}"
    cmd = ["git", "log", "--all", f"--format={fmt}"]
    if author:
        cmd.append(f"--author={author}")
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=repo_root)
    if result.returncode != 0:
        print(f"Error running git log: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    commits = []
    for record in result.stdout.split(RS):
        record = record.strip()
        if not record:
            continue
        parts = record.split(FS, 5)
        if len(parts) < 6:
            continue
        commit_hash, date_iso, author_name, author_email, subject, full_message = parts
        commit_time = datetime.fromisoformat(date_iso)
        commits.append({
            "hash": commit_hash,
            "short_hash": commit_hash[:10],
            "timestamp": commit_time,
            "author_name": author_name,
            "author_email": author_email,
            "subject": subject,
            "message": full_message.strip(),
        })
    return commits


def get_commit_diff_stat(repo_root: Path, commit_hash: str) -> str:
    """Get the --stat summary for a commit."""
    cmd = ["git", "diff-tree", "--stat", "--no-commit-id", "-r", commit_hash]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=repo_root)
    return result.stdout.strip() if result.returncode == 0 else ""


def encode_project_path(path: Path) -> str:
    """Encode a project path the way Claude Code does: replace / with -."""
    path_str = str(path)
    # Claude Code replaces special chars (/, spaces, ~) with -
    encoded = re.sub(r"[/\s~]", "-", path_str)
    # Some versions also strip leading dashes or handle differently
    return encoded


def find_session_dir(repo_root: Path) -> Path | None:
    """Find the Claude Code session directory for this project."""
    claude_projects = Path.home() / ".claude" / "projects"
    if not claude_projects.exists():
        return None

    encoded = encode_project_path(repo_root)

    # Try exact match first
    candidate = claude_projects / encoded
    if candidate.exists():
        return candidate

    # Try matching: the encoding can vary slightly between versions.
    # Look for directories that end with the repo name or match closely.
    repo_name = repo_root.name
    for entry in claude_projects.iterdir():
        if entry.is_dir() and entry.name == encoded:
            return entry

    # Fallback: look for any directory whose decoded path matches repo_root
    for entry in claude_projects.iterdir():
        if not entry.is_dir():
            continue
        # Heuristic: the encoded name should end with the repo directory name
        # and the full path parts should be recoverable
        decoded_guess = entry.name.replace("-", "/")
        if decoded_guess.endswith(str(repo_root)):
            return entry
        # Also try: the entry name should contain the repo root path segments
        if repo_name in entry.name:
            # Further verify by checking if path segments align
            root_parts = str(repo_root).strip("/").split("/")
            entry_parts = entry.name.strip("-").split("-")
            if all(part in entry_parts for part in root_parts):
                return entry

    return None


def parse_session_file(filepath: Path, detail: str = "minimal") -> list[dict]:
    """
    Parse a Claude Code session JSONL file.

    Returns a list of message dicts with:
      - role: "human" or "assistant"
      - blocks: list of content block dicts (see extract_content_blocks)
      - timestamp: datetime
      - session_id: str
    
    What is included depends on the detail level (minimal / standard / full).
    """
    messages = []
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            for line_num, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue

                # Skip subagent files (filename starts with "agent-")
                # This is handled at the caller level, but double-check
                if entry.get("isSidechain", False):
                    continue

                # Skip meta/system injected messages
                if entry.get("isMeta", False):
                    continue

                # Skip compact summaries
                if entry.get("isCompactSummary", False):
                    continue

                # We only want user and assistant message types
                entry_type = entry.get("type")
                if entry_type not in ("user", "assistant"):
                    continue

                message = entry.get("message", {})
                role = message.get("role", "")
                if role not in ("user", "assistant"):
                    continue

                timestamp_str = entry.get("timestamp")
                if not timestamp_str:
                    continue

                try:
                    timestamp = datetime.fromisoformat(timestamp_str)
                except (ValueError, TypeError):
                    continue

                session_id = entry.get("sessionId", filepath.stem)

                content = message.get("content", "")
                blocks = extract_content_blocks(content, detail)

                if not blocks:
                    continue

                messages.append({
                    "role": "human" if role == "user" else "assistant",
                    "blocks": blocks,
                    "timestamp": timestamp,
                    "session_id": session_id,
                })
    except Exception as e:
        print(f"  Warning: failed to parse {filepath.name}: {e}", file=sys.stderr)

    return messages


def extract_content_blocks(content, detail: str = "minimal") -> list[dict]:
    """Extract content blocks from a message's content field.

    Returns a list of typed dicts:
      {"type": "text", "text": "..."}
      {"type": "tool_use", "name": "...", "input": {...}}
      {"type": "thinking", "text": "..."}
      {"type": "tool_result", "content": "...", "is_error": bool}

    What gets included depends on the detail level:
      minimal  — text only
      standard — text + tool_use
      full     — text + tool_use + thinking + tool_result
    """
    if isinstance(content, str):
        stripped = content.strip()
        return [{"type": "text", "text": stripped}] if stripped else []

    if not isinstance(content, list):
        return []

    blocks = []
    for block in content:
        if not isinstance(block, dict):
            continue
        btype = block.get("type", "")

        if btype == "text":
            text = block.get("text", "").strip()
            if text:
                blocks.append({"type": "text", "text": text})

        elif btype == "tool_use" and detail in ("standard", "full"):
            blocks.append({
                "type": "tool_use",
                "name": block.get("name", "unknown"),
                "input": block.get("input", {}),
            })

        elif btype == "thinking" and detail == "full":
            text = block.get("thinking", "").strip()
            if text:
                blocks.append({"type": "thinking", "text": text})

        elif btype == "tool_result" and detail == "full":
            result_content = _extract_tool_result_text(block)
            if result_content:
                blocks.append({
                    "type": "tool_result",
                    "content": result_content,
                    "is_error": block.get("is_error", False),
                })

    return blocks


# Maximum lines / chars to keep from a single tool result at detail=full.
_TOOL_RESULT_MAX_LINES = 8
_TOOL_RESULT_MAX_CHARS = 600


def _extract_tool_result_text(block: dict) -> str:
    """Pull a displayable string out of a tool_result content block, truncated."""
    content = block.get("content", "")

    # content can be a string or a list of sub-blocks
    if isinstance(content, str):
        raw = content
    elif isinstance(content, list):
        parts = []
        for sub in content:
            if isinstance(sub, dict) and sub.get("type") == "text":
                parts.append(sub.get("text", ""))
        raw = "\n".join(parts)
    else:
        return ""

    raw = raw.strip()
    if not raw:
        return ""

    # Truncate
    lines = raw.splitlines()
    if len(lines) > _TOOL_RESULT_MAX_LINES:
        lines = lines[:_TOOL_RESULT_MAX_LINES]
        lines.append(f"… ({len(raw.splitlines()) - _TOOL_RESULT_MAX_LINES} more lines)")
    text = "\n".join(lines)
    if len(text) > _TOOL_RESULT_MAX_CHARS:
        text = text[:_TOOL_RESULT_MAX_CHARS] + " …(truncated)"
    return text


def load_all_sessions(session_dir: Path, detail: str = "minimal") -> dict[str, list[dict]]:
    """Load all non-agent session files from the session directory.
    
    Returns a dict mapping session_id -> sorted list of messages.
    """
    sessions = {}
    jsonl_files = sorted(session_dir.glob("*.jsonl"))

    for filepath in jsonl_files:
        # Skip subagent session files
        if filepath.name.startswith("agent-"):
            continue

        messages = parse_session_file(filepath, detail)
        if not messages:
            continue

        # Group by session_id (usually one per file, but be safe)
        for msg in messages:
            sid = msg["session_id"]
            if sid not in sessions:
                sessions[sid] = []
            sessions[sid].append(msg)

    # Sort each session's messages by timestamp
    for sid in sessions:
        sessions[sid].sort(key=lambda m: m["timestamp"])

    return sessions


def get_session_time_range(messages: list[dict]) -> tuple[datetime, datetime]:
    """Get the first and last timestamp of a session."""
    return messages[0]["timestamp"], messages[-1]["timestamp"]


def match_commits_to_sessions(
    commits: list[dict],
    sessions: dict[str, list[dict]],
) -> dict[str, list[tuple[str, list[dict]]]]:
    """
    For each commit, find sessions that were active at the time of the commit.

    A session is considered relevant to a commit if the commit timestamp falls
    within the session's time range (first message to last message), with a
    small buffer (15 minutes before first message, 5 minutes after last).

    Returns: dict mapping commit_hash -> list of (session_id, relevant_messages).
    The relevant_messages are all human/assistant messages from the session that
    occurred before or at the commit time (the context that led to the commit).
    """
    PRE_BUFFER_SECONDS = 15 * 60   # 15 minutes before session start
    POST_BUFFER_SECONDS = 5 * 60   # 5 minutes after session end

    # Precompute session time ranges
    session_ranges = {}
    for sid, messages in sessions.items():
        start, end = get_session_time_range(messages)
        session_ranges[sid] = (start, end)

    commit_sessions = {}
    for commit in commits:
        ct = commit["timestamp"]
        matched = []

        for sid, messages in sessions.items():
            s_start, s_end = session_ranges[sid]

            # Make all datetimes offset-aware for comparison
            ct_aware = ensure_aware(ct)
            s_start_aware = ensure_aware(s_start)
            s_end_aware = ensure_aware(s_end)

            from datetime import timedelta
            window_start = s_start_aware - timedelta(seconds=PRE_BUFFER_SECONDS)
            window_end = s_end_aware + timedelta(seconds=POST_BUFFER_SECONDS)

            if window_start <= ct_aware <= window_end:
                # Include messages up to the commit time (+ small buffer)
                cutoff = ct_aware + timedelta(seconds=60)
                relevant = [
                    m for m in messages
                    if ensure_aware(m["timestamp"]) <= cutoff
                ]
                if relevant:
                    matched.append((sid, relevant))

        if matched:
            commit_sessions[commit["hash"]] = matched

    return commit_sessions


def ensure_aware(dt: datetime) -> datetime:
    """Ensure a datetime is timezone-aware (assume UTC if naive)."""
    if dt.tzinfo is None:
        return dt.replace(tzinfo=timezone.utc)
    return dt


def format_timestamp(dt: datetime) -> str:
    """Format a datetime as ISO-8601 with minute precision."""
    return dt.strftime("%Y-%m-%dT%H:%M")


def generate_commit_markdown(
    commit: dict,
    matched_sessions: list[tuple[str, list[dict]]],
    diff_stat: str,
    prev_commit: dict | None = None,
    next_commit: dict | None = None,
    include_nav: bool = True,
) -> str:
    """Generate the markdown audit file for a single commit."""
    lines = []

    # YAML front matter (machine-readable)
    lines.append("---")
    lines.append(f'commit: "{commit["hash"]}"')
    lines.append(f'date: "{commit["timestamp"].isoformat()}"')
    lines.append(f'author: "{commit["author_name"]} <{commit["author_email"]}>"')
    lines.append(f'subject: "{escape_yaml_string(commit["subject"])}"')
    session_ids = [sid for sid, _ in matched_sessions]
    lines.append(f"sessions: {json.dumps(session_ids)}")
    if prev_commit:
        lines.append(f'prev: "{prev_commit["short_hash"]}.md"')
    if next_commit:
        lines.append(f'next: "{next_commit["short_hash"]}.md"')
    lines.append("---")
    lines.append("")

    # Navigation (top)
    if include_nav:
        nav = _format_nav(prev_commit, next_commit)
        lines.append(nav)
        lines.append("")

    # Human-readable header
    lines.append(f"# Commit {commit['short_hash']}")
    lines.append("")
    lines.append(f"**Date:** {format_timestamp(commit['timestamp'])}  ")
    lines.append(f"**Author:** {commit['author_name']} <{commit['author_email']}>")
    lines.append("")

    # Full commit message
    lines.append("## Commit message")
    lines.append("")
    lines.append(commit["message"])
    lines.append("")

    if diff_stat:
        lines.append("## Changed files")
        lines.append("")
        lines.append("```")
        lines.append(diff_stat)
        lines.append("```")
        lines.append("")

    # Conversation transcript(s)
    msg_counter = 0
    for session_idx, (sid, messages) in enumerate(matched_sessions):
        if len(matched_sessions) > 1:
            lines.append(f"## Session {session_idx + 1} (`{sid[:8]}…`)")
        else:
            lines.append(f"## Session `{sid[:8]}…`")
        lines.append("")

        session_start = format_timestamp(messages[0]["timestamp"])
        session_end = format_timestamp(messages[-1]["timestamp"])
        lines.append(f"*{session_start} → {session_end}*")
        lines.append("")

        for msg in messages:
            # If a "human" message contains only tool results (no actual user
            # text), render the results without a misleading "Human" header.
            has_text = any(b["type"] == "text" for b in msg["blocks"])
            only_tool_results = all(b["type"] == "tool_result" for b in msg["blocks"])

            if msg["role"] == "human" and not has_text and only_tool_results:
                # Render tool results directly (they follow the preceding
                # assistant tool_use call and are self-explanatory).
                lines.extend(_render_blocks(msg["blocks"]))
            else:
                msg_counter += 1
                role_label = "🧑 Human" if msg["role"] == "human" else "🤖 Assistant"
                role_tag = "human" if msg["role"] == "human" else "assistant"
                ts = format_timestamp(msg["timestamp"])
                anchor = f"msg-{role_tag}-{msg_counter}"
                lines.append(f'<a id="{anchor}"></a>')
                lines.append("")
                lines.append(f"### {role_label} ({ts})")
                lines.append("")
                lines.extend(_render_blocks(msg["blocks"]))
                lines.append("")

    # Bottom navigation
    if include_nav:
        lines.append("---")
        lines.append("")
        lines.append(nav)
        lines.append("")

    return "\n".join(lines)


def _render_blocks(blocks: list[dict]) -> list[str]:
    """Render a list of content blocks to markdown lines."""
    lines = []
    for block in blocks:
        btype = block["type"]

        if btype == "text":
            lines.append(block["text"])
            lines.append("")

        elif btype == "tool_use":
            compact_input = _format_tool_input(block["input"])
            lines.append("<details>")
            lines.append(f"<summary>🔧 <code>{block['name']}</code></summary>")
            lines.append("")
            lines.append(f"```")
            lines.append(compact_input)
            lines.append(f"```")
            lines.append("")
            lines.append("</details>")
            lines.append("")

        elif btype == "thinking":
            lines.append("<details>")
            lines.append("<summary>💭 Thinking</summary>")
            lines.append("")
            lines.append(block["text"])
            lines.append("")
            lines.append("</details>")
            lines.append("")

        elif btype == "tool_result":
            error_tag = " ❌" if block.get("is_error") else ""
            lines.append("<details>")
            lines.append(f"<summary>📎 Result{error_tag}</summary>")
            lines.append("")
            lines.append("```")
            lines.append(block["content"])
            lines.append("```")
            lines.append("")
            lines.append("</details>")
            lines.append("")

    return lines


def _format_tool_input(input_dict: dict) -> str:
    """Format tool input for display inside a code block, one arg per line."""
    if not input_dict:
        return "(no arguments)"

    lines = []
    for key, value in input_dict.items():
        val_str = json.dumps(value, ensure_ascii=False) if not isinstance(value, str) else value
        # Truncate individual values that are very long
        if len(val_str) > 120:
            val_str = val_str[:117] + "…"
        lines.append(f"{key}: {val_str}")

    return "\n".join(lines)


def _format_nav(prev_commit: dict | None, next_commit: dict | None) -> str:
    """Format prev/next navigation line."""
    parts = []
    if prev_commit:
        parts.append(f"[← Previous ({prev_commit['short_hash']})]({prev_commit['short_hash']}.md)")
    parts.append("[Index](index.md)")
    if next_commit:
        parts.append(f"[Next ({next_commit['short_hash']}) →]({next_commit['short_hash']}.md)")
    return " | ".join(parts)


def escape_yaml_string(s: str) -> str:
    """Escape a string for use in YAML front matter."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def generate_index(
    commits_with_sessions: list[tuple[dict, list[tuple[str, list[dict]]]]],
    total_commits: int,
    repo_root: Path,
    detail: str = "minimal",
) -> str:
    """Generate the index.md file listing all audited commits."""
    lines = []
    lines.append("---")
    lines.append(f'repo: "{repo_root}"')
    lines.append(f"total_commits: {total_commits}")
    lines.append(f"audited_commits: {len(commits_with_sessions)}")
    lines.append(f'detail: "{detail}"')
    lines.append(f'generated: "{datetime.now(timezone.utc).isoformat()}"')
    lines.append("---")
    lines.append("")
    lines.append("# Claude Code Audit Log")
    lines.append("")
    lines.append(f"Repository: `{repo_root}`  ")
    lines.append(f"Total commits: {total_commits}  ")
    lines.append(f"Commits with Claude Code sessions: {len(commits_with_sessions)}")
    lines.append("")
    lines.append("## Commits")
    lines.append("")

    for commit, matched in commits_with_sessions:
        short = commit["short_hash"]
        date = format_timestamp(commit["timestamp"])
        subject = commit["subject"]
        n_sessions = len(matched)
        n_messages = sum(len(msgs) for _, msgs in matched)
        session_info = f"{n_sessions} session{'s' if n_sessions > 1 else ''}, {n_messages} message{'s' if n_messages > 1 else ''}"
        filename = f"{short}.md"
        lines.append(f"- [{short}]({filename}) — {date} — {subject} ({session_info})")

    lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Generate an audit trail matching Claude Code sessions to git commits."
    )
    parser.add_argument(
        "--repo-path",
        type=str,
        default=None,
        help="Path to the git repository (default: current directory)",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default=None,
        help="Output directory (default: .claude-audit/ in repo root)",
    )
    parser.add_argument(
        "--author",
        type=str,
        default=None,
        help="Filter commits by author (passed to git log --author)",
    )
    parser.add_argument(
        "--session-dir",
        type=str,
        default=None,
        help="Explicitly specify the Claude Code session directory "
             "(default: auto-detect from ~/.claude/projects/)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be generated without writing files",
    )
    parser.add_argument(
        "--detail",
        choices=("minimal", "standard", "full"),
        default="minimal",
        help="Detail level: minimal (human/assistant text only), "
             "standard (+tool call names & arguments), "
             "full (+thinking blocks, +truncated tool output). Default: minimal.",
    )
    parser.add_argument(
        "--rebuild",
        action="store_true",
        help="Regenerate all audit files (default: only create files for new commits)",
    )
    parser.add_argument(
        "--single-file",
        nargs="?",
        const="-",
        default=None,
        metavar="FILE",
        help="Write all audited commits to a single file (oldest first). "
             "Use without a value or with '-' for stdout.",
    )
    args = parser.parse_args()

    # When stdout is used for content output, route progress to stderr
    _stdout_is_content = args.single_file == "-"

    def info(msg=""):
        print(msg, file=sys.stderr if _stdout_is_content else sys.stdout)

    # Resolve paths
    repo_root = get_repo_root(args.repo_path)
    output_dir = Path(args.output_dir) if args.output_dir else repo_root / ".claude-audit"

    info(f"Repository: {repo_root}")

    # Find session directory
    if args.session_dir:
        session_dir = Path(args.session_dir)
        if not session_dir.exists():
            print(f"Error: specified session directory does not exist: {session_dir}", file=sys.stderr)
            sys.exit(1)
    else:
        session_dir = find_session_dir(repo_root)

    if session_dir is None:
        print("Error: could not find Claude Code session directory for this project.", file=sys.stderr)
        print(f"Looked in: {Path.home() / '.claude' / 'projects'}", file=sys.stderr)
        print(f"Expected encoded path for: {repo_root}", file=sys.stderr)
        print(f"Try: ls ~/.claude/projects/ | grep {repo_root.name}", file=sys.stderr)
        print(f"Then re-run with --session-dir <path>", file=sys.stderr)
        sys.exit(1)

    info(f"Session dir: {session_dir}")

    # Load sessions
    info("Loading sessions...")
    sessions = load_all_sessions(session_dir, detail=args.detail)
    if not sessions:
        info("No sessions found (or all sessions were empty at this detail level).")
        sys.exit(0)

    total_messages = sum(len(msgs) for msgs in sessions.values())
    detail_label = {"minimal": "conversation", "standard": "conversation+tools", "full": "full detail"}
    info(f"  Found {len(sessions)} session(s) with {total_messages} entries ({detail_label[args.detail]})")

    # Get commits
    info("Reading git history...")
    commits = get_git_commits(repo_root, author=args.author)
    if not commits:
        info("No commits found.")
        sys.exit(0)
    info(f"  Found {len(commits)} commit(s)")

    # Match
    info("Matching commits to sessions...")
    commit_sessions = match_commits_to_sessions(commits, sessions)
    info(f"  {len(commit_sessions)} commit(s) matched to session(s)")

    if not commit_sessions:
        info("\nNo commits could be matched to any Claude Code sessions.")
        info("This can happen if session timestamps don't overlap with commit times.")
        sys.exit(0)

    # Single-file output mode
    if args.single_file is not None:
        # Build list oldest-first (git log returns newest-first)
        audited = [c for c in reversed(commits) if c["hash"] in commit_sessions]
        parts = []

        # Header
        parts.append(generate_index(
            [(c, commit_sessions[c["hash"]]) for c in audited],
            len(commits), repo_root, detail=args.detail,
        ))

        # Each commit, oldest first, no nav links
        for commit in audited:
            matched = commit_sessions[commit["hash"]]
            diff_stat = get_commit_diff_stat(repo_root, commit["hash"])
            md = generate_commit_markdown(commit, matched, diff_stat, include_nav=False)
            parts.append(md)

        combined = "\n---\n\n".join(parts) + "\n"

        if args.single_file == "-":
            sys.stdout.write(combined)
        else:
            outpath = Path(args.single_file)
            outpath.parent.mkdir(parents=True, exist_ok=True)
            outpath.write_text(combined, encoding="utf-8")
            print(f"Written {len(audited)} audited commit(s) to {outpath}",
                  file=sys.stderr)
        return

    if args.dry_run:
        new_count = 0
        existing_count = 0
        for commit in commits:
            if commit["hash"] in commit_sessions:
                filename = f"{commit['short_hash']}.md"
                filepath = output_dir / filename
                exists = filepath.exists()
                if exists and not args.rebuild:
                    existing_count += 1
                    continue
                new_count += 1
                matched = commit_sessions[commit["hash"]]
                n_msgs = sum(len(msgs) for _, msgs in matched)
                label = " (rebuild)" if exists else ""
                info(f"  {filename} — {commit['subject'][:60]} ({n_msgs} msgs){label}")
        if not args.rebuild and existing_count:
            info(f"  ({existing_count} existing file(s) skipped; use --rebuild to regenerate)")
        if new_count == 0:
            info(f"\nDry run — no new files to generate in {output_dir}/")
        else:
            info(f"\nDry run — would generate {new_count} file(s) in {output_dir}/")
        return

    # Generate output
    output_dir.mkdir(parents=True, exist_ok=True)

    # Build ordered list of audited commits (git log order: index 0 is newest)
    audited = []
    for commit in commits:
        if commit["hash"] in commit_sessions:
            audited.append(commit)

    # Determine which files need (re)generation
    new_indices = set()
    for i, commit in enumerate(audited):
        filename = f"{commit['short_hash']}.md"
        filepath = output_dir / filename
        if args.rebuild or not filepath.exists():
            new_indices.add(i)

    # Also regenerate immediate neighbors of new files, because their
    # prev/next navigation links may have changed.
    neighbor_indices = set()
    for i in new_indices:
        if i > 0:
            neighbor_indices.add(i - 1)
        if i < len(audited) - 1:
            neighbor_indices.add(i + 1)
    write_indices = new_indices | neighbor_indices

    if not new_indices:
        info("\nNo new commits to audit.")
        # Still regenerate index in case it was deleted
        commits_with_sessions = [(c, commit_sessions[c["hash"]]) for c in audited]
        index_md = generate_index(commits_with_sessions, len(commits), repo_root, detail=args.detail)
        index_path = output_dir / "index.md"
        index_path.write_text(index_md, encoding="utf-8")
        info(f"  index.md refreshed — {len(commits_with_sessions)} audited commits")
        return

    n_new = len(new_indices)
    n_updated = len(neighbor_indices - new_indices)
    info(f"\nWriting audit files to {output_dir}/")
    if not args.rebuild:
        info(f"  {n_new} new, {n_updated} updated (nav links), "
              f"{len(audited) - len(write_indices)} unchanged")

    for i, commit in enumerate(audited):
        if i not in write_indices:
            continue

        matched = commit_sessions[commit["hash"]]
        diff_stat = get_commit_diff_stat(repo_root, commit["hash"])

        prev_commit = audited[i - 1] if i > 0 else None
        next_commit = audited[i + 1] if i < len(audited) - 1 else None

        md = generate_commit_markdown(commit, matched, diff_stat, prev_commit, next_commit)

        filename = f"{commit['short_hash']}.md"
        filepath = output_dir / filename
        filepath.write_text(md, encoding="utf-8")

        n_msgs = sum(len(msgs) for _, msgs in matched)
        tag = "" if i in new_indices else " (nav update)"
        info(f"  {filename} — {commit['subject'][:60]} ({n_msgs} msgs){tag}")

    commits_with_sessions = [(c, commit_sessions[c["hash"]]) for c in audited]

    # Write index
    index_md = generate_index(commits_with_sessions, len(commits), repo_root, detail=args.detail)
    index_path = output_dir / "index.md"
    index_path.write_text(index_md, encoding="utf-8")
    info(f"  index.md — {len(commits_with_sessions)} audited commits")

    info(f"\nDone. {len(write_indices)} file(s) written, "
          f"{len(commits_with_sessions)} total audited commits.")


if __name__ == "__main__":
    main()
