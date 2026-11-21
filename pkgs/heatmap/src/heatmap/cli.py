"""commit-heatmap: print a repository's commits per weekday and hour."""

from __future__ import annotations

import argparse
import json

from rich.console import Console
from rich.text import Text

from .core import WEEKDAYS, Grid, build_grid, busiest, commit_times

SHADES = " ░▒▓█"


def shade(count: int, peak: int) -> str:
    """One block character whose density grows with count relative to the peak."""
    if count == 0 or peak == 0:
        return SHADES[0]
    return SHADES[1 + min(len(SHADES) - 2, (count * (len(SHADES) - 1) - 1) // peak)]


def render(grid: Grid) -> Text:
    """One row per weekday, two characters per hour, hour labels every 3 hours."""
    peak = max(max(row) for row in grid)
    text = Text("    " + "".join(f"{h:02}    " for h in range(0, 24, 3)).rstrip() + "\n")
    for day, row in zip(WEEKDAYS, grid, strict=True):
        text.append(f"{day} ")
        for count in row:
            text.append(shade(count, peak) * 2, style="green" if count else "dim")
        text.append("\n")
    return text


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="commit-heatmap", description=__doc__)
    parser.add_argument("repo", nargs="?", default=".", help="git repository (default: .)")
    parser.add_argument("--since", help='only commits after this date, e.g. "90 days ago"')
    parser.add_argument("--json", action="store_true", help="print the 7x24 grid as JSON")
    args = parser.parse_args(argv)

    grid = build_grid(commit_times(args.repo, args.since))
    if args.json:
        print(json.dumps({"weekdays": WEEKDAYS, "grid": grid}))
        return

    console = Console()
    console.print(render(grid))
    top = busiest(grid)
    total = sum(map(sum, grid))
    if top:
        day, hour, count = top
        console.print(f"{total} commits; busiest slot {day} {hour:02}:00 ({count})")
    else:
        console.print("no commits")
