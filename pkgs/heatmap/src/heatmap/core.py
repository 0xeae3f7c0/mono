"""Counting commits into a weekday-by-hour grid."""

from __future__ import annotations

import subprocess
from collections.abc import Iterable
from datetime import datetime

WEEKDAYS = ("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun")

Grid = list[list[int]]


def commit_times(repo: str, since: str | None = None) -> list[datetime]:
    """Author timestamps of every commit reachable from HEAD, in each author's local time."""
    cmd = ["git", "-C", repo, "log", "--format=%aI"]
    if since:
        cmd.append(f"--since={since}")
    out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
    return [datetime.fromisoformat(line) for line in out.splitlines() if line]


def build_grid(times: Iterable[datetime]) -> Grid:
    """A 7x24 grid: grid[weekday][hour] is the number of commits in that slot."""
    grid = [[0] * 24 for _ in WEEKDAYS]
    for t in times:
        grid[t.weekday()][t.hour] += 1
    return grid


def busiest(grid: Grid) -> tuple[str, int, int] | None:
    """The weekday, hour and count of the busiest slot, or None for an empty grid."""
    count, day, hour = max((c, d, h) for d, row in enumerate(grid) for h, c in enumerate(row))
    return (WEEKDAYS[day], hour, count) if count else None
