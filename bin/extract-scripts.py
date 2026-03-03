#!/usr/bin/env python3
"""
Extract "interesting" inline scripts from Claude Code session files.

Usage:
    python3 extract-scripts.py ~/.claude/projects/*/sessions/*.jsonl -o extracted-scripts.md
    python3 extract-scripts.py ~/.claude/projects/ -o extracted-scripts.md  # recursive search

Extracts bash/shell tool-use blocks that are longer than a threshold,
categorizes recurring patterns, and writes a compact markdown report.
"""

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path


MIN_LINES = 5  # default minimum lines to be "interesting"


def find_session_files(paths):
    """Find all .jsonl session files from given paths."""
    files = []
    for p in paths:
        p = Path(p).expanduser()
        if p.is_file() and p.suffix == '.jsonl':
            files.append(p)
        elif p.is_dir():
            files.extend(sorted(p.rglob('*.jsonl')))
    return files


def extract_tool_uses(filepath):
    """Parse a JSONL session file and extract bash tool-use blocks."""
    scripts = []
    line_num = 0

    with open(filepath, 'r', errors='replace') as f:
        for raw_line in f:
            line_num += 1
            raw_line = raw_line.strip()
            if not raw_line:
                continue
            try:
                msg = json.loads(raw_line)
            except json.JSONDecodeError:
                continue

            # Claude Code session messages can have several shapes.
            # We look for tool_use blocks in any content array we find.
            content_sources = []

            # Shape 1: {"type": "assistant", "message": {"content": [...]}}
            if isinstance(msg.get('message'), dict):
                c = msg['message'].get('content')
                if isinstance(c, list):
                    content_sources.append(c)

            # Shape 2: {"role": "assistant", "content": [...]}
            c = msg.get('content')
            if isinstance(c, list):
                content_sources.append(c)

            # Shape 3: top-level tool_use
            if msg.get('type') == 'tool_use':
                content_sources.append([msg])

            for content_list in content_sources:
                for block in content_list:
                    if not isinstance(block, dict):
                        continue
                    if block.get('type') != 'tool_use':
                        continue

                    tool_name = (block.get('name') or '').lower()
                    tool_input = block.get('input', {})

                    # Bash tool: the command is in input.command
                    if tool_name in ('bash', 'shell', 'execute', 'terminal',
                                     'bash_tool', 'run_command'):
                        cmd = tool_input.get('command', '')
                        if not cmd and isinstance(tool_input, str):
                            cmd = tool_input
                        if cmd:
                            scripts.append({
                                'file': str(filepath),
                                'session': filepath.stem,
                                'line': line_num,
                                'tool': tool_name,
                                'command': cmd,
                                'timestamp': _extract_timestamp(msg),
                            })

                    # Write/Edit tool: not a "script" but could be interesting
                    # We skip these for now; focus on executed commands.

    return scripts


def _extract_timestamp(msg):
    """Try to pull a timestamp from the message."""
    for key in ('timestamp', 'createdAt', 'created_at', 'ts'):
        val = msg.get(key)
        if val:
            return str(val)
        if isinstance(msg.get('message'), dict):
            val = msg['message'].get(key)
            if val:
                return str(val)
    return None


def classify_script(command):
    """Categorize a script by its primary purpose."""
    cmd = command.lower()

    # Order matters: more specific patterns first
    if re.search(r'xcodebuild\s+test\b', cmd):
        return 'xcode-test'
    if re.search(r'xcodebuild\s+build\b', cmd) or re.search(r'xcodebuild\b(?!.*test)', cmd):
        return 'xcode-build'
    if re.search(r'swift\s+test\b', cmd):
        return 'swift-test'
    if re.search(r'swift\s+build\b', cmd):
        return 'swift-build'
    if re.search(r'\bgit\s+commit\b', cmd):
        return 'git-commit'
    if re.search(r'\bgit\s+diff\b', cmd):
        return 'git-diff'
    if re.search(r'\bgit\s+log\b', cmd):
        return 'git-log'
    if re.search(r'\bgit\s+(add|restore|checkout|reset|stash|revert)\b', cmd):
        return 'git-other'
    if re.search(r'\bgit\b', cmd):
        return 'git-other'
    if re.search(r'\b(grep|rg|ag|ack)\b', cmd):
        return 'search'
    if re.search(r'\b(find|fd)\b.*\.(swift|xc)', cmd):
        return 'find-files'
    if re.search(r'\bfind\b', cmd):
        return 'find-files'
    if re.search(r'\b(cat|head|tail|less|more|bat)\b', cmd):
        return 'read-file'
    if re.search(r'\b(sed|awk|perl)\b.*-i', cmd):
        return 'inline-edit'
    if re.search(r'\b(sed|awk|perl|cut|sort|uniq|wc|tr)\b', cmd):
        return 'text-processing'
    if re.search(r'\b(mkdir|cp|mv|rm|ln|chmod)\b', cmd):
        return 'file-ops'
    if re.search(r'\b(ls|tree|du)\b', cmd):
        return 'list-files'
    if re.search(r'\bpython', cmd) or re.search(r'\bruby\b', cmd):
        return 'scripting'
    if re.search(r'\bcurl\b|\bwget\b', cmd):
        return 'network'
    if re.search(r'\bxcrun\b|\bsimctl\b', cmd):
        return 'xcode-tools'

    return 'other'


def line_count(command):
    """Count meaningful lines in a command."""
    lines = command.strip().split('\n')
    # Filter out blank lines and pure comments for counting
    meaningful = [l for l in lines if l.strip() and not l.strip().startswith('#')]
    return len(meaningful)


def is_interesting(command, min_lines=MIN_LINES):
    """Determine if a script is worth including."""
    return line_count(command) >= min_lines


def dedup_key(command):
    """
    Create a normalized key for deduplication.
    Strips variable parts like paths, hashes, timestamps.
    """
    # Normalize whitespace
    s = re.sub(r'\s+', ' ', command.strip())
    # Strip quoted strings (file paths, messages, etc.)
    s = re.sub(r'"[^"]*"', '""', s)
    s = re.sub(r"'[^']*'", "''", s)
    return s


def generate_report(scripts, min_lines, top_n_recurring=20, max_samples=3):
    """Generate the markdown report."""
    lines = []
    lines.append('# Claude Code Inline Script Analysis\n')

    # --- Overview ---
    total = len(scripts)
    interesting = [s for s in scripts if is_interesting(s['command'], min_lines)]
    categories = Counter(classify_script(s['command']) for s in scripts)
    interesting_categories = Counter(classify_script(s['command']) for s in interesting)

    lines.append(f'## Overview\n')
    lines.append(f'- Total bash tool-use invocations: {total}')
    lines.append(f'- Invocations with ≥{min_lines} meaningful lines: {len(interesting)}')
    lines.append(f'- Unique sessions: {len(set(s["session"] for s in scripts))}')
    lines.append('')

    # --- Category breakdown ---
    lines.append('## All Commands by Category\n')
    lines.append(f'| Category | Total | ≥{min_lines} lines |')
    lines.append('|----------|------:|----------:|')
    for cat, count in categories.most_common():
        int_count = interesting_categories.get(cat, 0)
        lines.append(f'| {cat} | {count} | {int_count} |')
    lines.append('')

    # --- Recurring patterns (deduplicated) ---
    lines.append(f'## Recurring Commands (top {top_n_recurring})\n')
    lines.append('Commands that appear multiple times (normalized). '
                 'These are prime candidates for extraction into scripts.\n')

    deduped = Counter()
    dedup_examples = {}
    for s in scripts:
        key = dedup_key(s['command'])
        deduped[key] += 1
        if key not in dedup_examples:
            dedup_examples[key] = s['command']

    for key, count in deduped.most_common(top_n_recurring):
        if count < 2:
            break
        example = dedup_examples[key]
        cat = classify_script(example)
        # Truncate for display
        display = example.strip().split('\n')[0][:120]
        lines.append(f'- **{count}×** [{cat}] `{display}`')
    lines.append('')

    # --- Interesting scripts (≥ min_lines) ---
    lines.append(f'## Interesting Scripts (≥{min_lines} lines)\n')
    lines.append(f'{len(interesting)} scripts found. '
                 f'Grouped by category, showing up to {max_samples} samples each.\n')

    by_category = defaultdict(list)
    for s in interesting:
        cat = classify_script(s['command'])
        by_category[cat].append(s)

    for cat in sorted(by_category, key=lambda c: -len(by_category[c])):
        entries = by_category[cat]
        lines.append(f'### {cat} ({len(entries)} scripts)\n')

        # Show a few representative samples, preferring longer/diverse ones
        seen_keys = set()
        shown = 0
        # Sort by length descending to show most substantial first
        for s in sorted(entries, key=lambda x: -line_count(x['command'])):
            key = dedup_key(s['command'])
            if key in seen_keys:
                continue
            seen_keys.add(key)

            lc = line_count(s['command'])
            ts = s['timestamp'] or 'unknown time'
            lines.append(f'**Session:** `{s["session"]}` | **Lines:** {lc} | **Time:** {ts}\n')
            lines.append('```bash')
            # Truncate very long scripts
            cmd_lines = s['command'].split('\n')
            if len(cmd_lines) > 60:
                lines.extend(cmd_lines[:50])
                lines.append(f'# ... ({len(cmd_lines) - 50} more lines)')
            else:
                lines.extend(cmd_lines)
            lines.append('```\n')

            shown += 1
            if shown >= max_samples:
                if len(seen_keys) < len(entries):
                    lines.append(f'*({len(entries) - shown} more in this category, '
                                 f'{len(entries) - len(seen_keys)} unique)*\n')
                break

    # --- Scriptability recommendations ---
    lines.append('## Scriptability Assessment\n')
    lines.append('Categories ranked by how much you\'d gain from extracting them '
                 'into reusable scripts:\n')

    scriptability = [
        ('xcode-test', 'HIGH',
         'Stable invocation pattern. Extract: build-and-test.sh with '
         'failure parsing and summary output.'),
        ('xcode-build', 'HIGH',
         'Stable invocation. Extract: build.sh with error formatting.'),
        ('git-commit', 'MEDIUM',
         'The commit itself varies, but pre-commit checks '
         '(test pass, no REVIEW: markers, etc.) are scriptable.'),
        ('search', 'MEDIUM',
         'Common grep/rg patterns for finding Swift symbols, '
         'TODO markers, etc. could be named shortcuts.'),
        ('text-processing', 'MEDIUM',
         'Pipelines for parsing test output, extracting failures, '
         'counting lines — worth extracting if patterns repeat.'),
        ('inline-edit', 'LOW',
         'Each edit is unique. Sed one-liners are hard to generalize.'),
        ('read-file', 'LOW',
         'Trivial commands, not worth wrapping.'),
        ('file-ops', 'LOW',
         'Mostly one-offs.'),
    ]

    for cat, rating, note in scriptability:
        count = categories.get(cat, 0)
        if count > 0:
            lines.append(f'- **{rating}** — `{cat}` ({count} invocations): {note}')

    lines.append('')
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(
        description='Extract interesting inline scripts from Claude Code sessions.')
    parser.add_argument('paths', nargs='+',
                        help='Session .jsonl files or directories to search recursively')
    parser.add_argument('-o', '--output', default='extracted-scripts.md',
                        help='Output markdown file (default: extracted-scripts.md)')
    parser.add_argument('-n', '--min-lines', type=int, default=MIN_LINES,
                        help=f'Minimum meaningful lines to be "interesting" (default: {MIN_LINES})')
    parser.add_argument('--max-samples', type=int, default=3,
                        help='Max sample scripts per category (default: 3)')
    parser.add_argument('--diag', action='store_true',
                        help='Print diagnostic info about file parsing')

    args = parser.parse_args()

    # Find files
    session_files = find_session_files(args.paths)
    if not session_files:
        print(f'No .jsonl files found in: {args.paths}', file=sys.stderr)
        sys.exit(1)

    print(f'Found {len(session_files)} session files', file=sys.stderr)

    # Extract
    all_scripts = []
    parse_errors = 0
    files_with_scripts = 0

    for sf in session_files:
        try:
            scripts = extract_tool_uses(sf)
            if scripts:
                files_with_scripts += 1
            all_scripts.extend(scripts)
        except Exception as e:
            parse_errors += 1
            if args.diag:
                print(f'  Error parsing {sf}: {e}', file=sys.stderr)

    print(f'Extracted {len(all_scripts)} bash invocations '
          f'from {files_with_scripts} sessions '
          f'({parse_errors} files had errors)', file=sys.stderr)

    if args.diag and all_scripts:
        # Show first script's raw structure to help debug format issues
        s = all_scripts[0]
        print(f'\nDiagnostic — first extracted command:', file=sys.stderr)
        print(f'  File: {s["file"]}', file=sys.stderr)
        print(f'  Tool: {s["tool"]}', file=sys.stderr)
        print(f'  Command preview: {s["command"][:200]}', file=sys.stderr)
        print(f'  Timestamp: {s["timestamp"]}', file=sys.stderr)

    if not all_scripts:
        print('\nNo bash tool-use blocks found. Possible causes:', file=sys.stderr)
        print('  - Session file format differs from expected', file=sys.stderr)
        print('  - Try --diag to see parsing details', file=sys.stderr)
        print('  - The tool name might differ (check a session file manually):', file=sys.stderr)
        print('    head -c 5000 <session-file> | python3 -m json.tool', file=sys.stderr)
        sys.exit(1)

    # Generate report
    report = generate_report(all_scripts, args.min_lines, max_samples=args.max_samples)

    with open(args.output, 'w') as f:
        f.write(report)

    print(f'Report written to {args.output}', file=sys.stderr)


if __name__ == '__main__':
    main()
