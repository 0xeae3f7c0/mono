from datetime import datetime

from heatmap.cli import shade
from heatmap.core import build_grid, busiest


def test_build_grid_counts_weekday_and_hour():
    times = [
        datetime(2026, 11, 16, 9, 30),  # Monday 09:00
        datetime(2026, 11, 16, 9, 5),
        datetime(2026, 11, 22, 23, 59),  # Sunday 23:00
    ]
    grid = build_grid(times)
    assert grid[0][9] == 2
    assert grid[6][23] == 1
    assert sum(map(sum, grid)) == 3


def test_busiest_slot():
    grid = build_grid([datetime(2026, 11, 18, 14), datetime(2026, 11, 18, 14, 45)])
    assert busiest(grid) == ("Wed", 14, 2)


def test_busiest_empty_grid():
    assert busiest(build_grid([])) is None


def test_shade_scales_with_peak():
    assert shade(0, 10) == " "
    assert shade(10, 10) == "█"
    assert shade(1, 10) == "░"
