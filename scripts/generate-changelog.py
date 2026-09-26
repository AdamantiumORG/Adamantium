#!/usr/bin/env python3
"""Generate deterministic Markdown release notes from Git history."""

import argparse
import pathlib
import subprocess


def git(*arguments: str) -> str:
    return subprocess.check_output(
        ["git", *arguments], text=True, encoding="utf-8", errors="replace"
    ).strip()


def category(subject: str, body: str) -> str:
    prefix = subject.split(":", 1)[0].lower()
    if "!" in prefix or "breaking change" in body.lower():
        return "Breaking changes"
    if prefix.startswith("feat"):
        return "Features"
    if prefix.startswith("fix"):
        return "Fixes"
    if prefix.startswith(("perf", "refactor")):
        return "Performance and internals"
    if prefix.startswith(("docs", "test", "ci", "build", "chore")):
        return "Documentation and tooling"
    return "Other changes"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--from-ref")
    parser.add_argument("--to-ref", default="HEAD")
    parser.add_argument("--output", default="RELEASE_NOTES.md")
    args = parser.parse_args()

    revision = f"{args.from_ref}..{args.to_ref}" if args.from_ref else args.to_ref
    records = git("log", revision, "--pretty=format:%H%x1f%s%x1f%b%x1e")
    groups: dict[str, list[tuple[str, str]]] = {}
    for record in records.split("\x1e"):
        fields = record.strip("\r\n ").split("\x1f", 2)
        if len(fields) != 3:
            continue
        commit, subject, body = fields
        groups.setdefault(category(subject, body), []).append((commit[:8], subject))

    lines = [f"# Adamantium {args.version}", ""]
    order = [
        "Breaking changes",
        "Features",
        "Fixes",
        "Performance and internals",
        "Documentation and tooling",
        "Other changes",
    ]
    for heading in order:
        entries = groups.get(heading)
        if not entries:
            continue
        lines.extend([f"## {heading}", ""])
        lines.extend(f"- {subject} (`{commit}`)" for commit, subject in entries)
        lines.append("")
    if not groups:
        lines.extend(["No commits were found for this release range.", ""])

    pathlib.Path(args.output).write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
