#!/usr/bin/env python3
"""Render the sorting-only benchmark CSV as a dependency-free SVG chart."""

import argparse
import csv
import math
from pathlib import Path

WIDTH = 1200
HEIGHT = 720
LEFT = 108
RIGHT = 42
TOP = 78
BOTTOM = 92
PLOT_WIDTH = WIDTH - LEFT - RIGHT
PLOT_HEIGHT = HEIGHT - TOP - BOTTOM


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("csv", type=Path)
    parser.add_argument("svg", type=Path)
    args = parser.parse_args()

    with args.csv.open(newline="", encoding="utf-8") as source:
        rows = list(csv.DictReader(source))
    if not rows:
        raise SystemExit("CSV contains no measurements")

    series = [
        ("CPU sort_unstable", "cpu_sort_median_ms", "#1f4e79"),
        ("GPU end-to-end", "gpu_end_to_end_median_ms", "#c44e52"),
        ("GPU radix sort only", "gpu_radix_sort_median_ms", "#4c956c"),
    ]
    x_values = [float(row["n"]) for row in rows]
    all_y = [float(row[column]) for _, column, _ in series for row in rows]
    x_min, x_max = min(x_values), max(x_values)
    y_min = 10 ** math.floor(math.log10(min(all_y)))
    y_max = 10 ** math.ceil(math.log10(max(all_y)))

    def x_position(value: float) -> float:
        fraction = (math.log10(value) - math.log10(x_min)) / (
            math.log10(x_max) - math.log10(x_min)
        )
        return LEFT + fraction * PLOT_WIDTH

    def y_position(value: float) -> float:
        fraction = (math.log10(value) - math.log10(y_min)) / (
            math.log10(y_max) - math.log10(y_min)
        )
        return TOP + (1.0 - fraction) * PLOT_HEIGHT

    crossover = None
    for previous, current in zip(rows, rows[1:]):
        if float(previous["cpu_over_gpu_end_to_end"]) <= 1.0 < float(
            current["cpu_over_gpu_end_to_end"]
        ):
            crossover = (int(previous["n"]), int(current["n"]))
            break

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">',
        "<style>",
        "text { font-family: Inter, ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif; fill: #222; }",
        ".tick { font-size: 14px; } .label { font-size: 17px; } .legend { font-size: 15px; }",
        "</style>",
        '<rect width="100%" height="100%" fill="white"/>',
        f'<text x="{WIDTH / 2}" y="35" text-anchor="middle" font-size="24" font-weight="600">Sorting random u32 values: CPU vs GPU</text>',
        f'<text x="{WIDTH / 2}" y="59" text-anchor="middle" font-size="14" fill="#555">Median wall time; NVIDIA GeForce RTX 3060, wgpu Vulkan, Lampshade radix sort</text>',
    ]

    if crossover:
        low, high = crossover
        low_x, high_x = x_position(low), x_position(high)
        svg.extend(
            [
                f'<rect x="{low_x:.2f}" y="{TOP}" width="{high_x - low_x:.2f}" height="{PLOT_HEIGHT}" fill="#d8d8d8" opacity="0.28"/>',
                f'<line x1="{high_x:.2f}" y1="{TOP}" x2="{high_x:.2f}" y2="{TOP + PLOT_HEIGHT}" stroke="#666" stroke-width="1.4" stroke-dasharray="6 5"/>',
                f'<text x="{(low_x + high_x) / 2:.2f}" y="{TOP + 22}" text-anchor="middle" font-size="14" fill="#444">crossover bracket</text>',
                f'<text x="{(low_x + high_x) / 2:.2f}" y="{TOP + 40}" text-anchor="middle" font-size="14" fill="#444">{format_count(low)}–{format_count(high)}</text>',
                f'<text x="{high_x + 8:.2f}" y="{TOP + 61}" font-size="13" fill="#555">first measured GPU win</text>',
            ]
        )

    for exponent in range(
        math.floor(math.log10(y_min)), math.ceil(math.log10(y_max)) + 1
    ):
        for multiplier in (1, 2, 5):
            value = multiplier * 10**exponent
            if not y_min <= value <= y_max:
                continue
            y = y_position(value)
            major = multiplier == 1
            svg.append(
                f'<line x1="{LEFT}" y1="{y:.2f}" x2="{LEFT + PLOT_WIDTH}" y2="{y:.2f}" stroke="#{"c9c9c9" if major else "e8e8e8"}" stroke-width="{1.0 if major else 0.7}"/>'
            )
            if major:
                svg.append(
                    f'<text class="tick" x="{LEFT - 12}" y="{y + 5:.2f}" text-anchor="end">{format_time(value)}</text>'
                )

    for value in decade_ticks(x_min, x_max):
        x = x_position(value)
        svg.extend(
            [
                f'<line x1="{x:.2f}" y1="{TOP}" x2="{x:.2f}" y2="{TOP + PLOT_HEIGHT}" stroke="#d5d5d5" stroke-width="1"/>',
                f'<text class="tick" x="{x:.2f}" y="{TOP + PLOT_HEIGHT + 25}" text-anchor="middle">{format_count(int(value))}</text>',
            ]
        )

    svg.extend(
        [
            f'<line x1="{LEFT}" y1="{TOP + PLOT_HEIGHT}" x2="{LEFT + PLOT_WIDTH}" y2="{TOP + PLOT_HEIGHT}" stroke="#333" stroke-width="1.5"/>',
            f'<line x1="{LEFT}" y1="{TOP}" x2="{LEFT}" y2="{TOP + PLOT_HEIGHT}" stroke="#333" stroke-width="1.5"/>',
            f'<text class="label" x="{LEFT + PLOT_WIDTH / 2}" y="{HEIGHT - 28}" text-anchor="middle">Number of u32 values (log scale)</text>',
            f'<text class="label" x="26" y="{TOP + PLOT_HEIGHT / 2}" text-anchor="middle" transform="rotate(-90 26 {TOP + PLOT_HEIGHT / 2})">Elapsed time (ms, log scale)</text>',
        ]
    )

    for name, column, color in series:
        points = [
            (x_position(float(row["n"])), y_position(float(row[column])))
            for row in rows
        ]
        path = " ".join(
            ("M" if index == 0 else "L") + f" {x:.2f} {y:.2f}"
            for index, (x, y) in enumerate(points)
        )
        svg.append(
            f'<path d="{path}" fill="none" stroke="{color}" stroke-width="3" stroke-linejoin="round" stroke-linecap="round"/>'
        )
        for x, y in points:
            svg.append(
                f'<circle cx="{x:.2f}" cy="{y:.2f}" r="3.8" fill="white" stroke="{color}" stroke-width="2.2"/>'
            )

    legend_x = LEFT + 18
    legend_y = TOP + PLOT_HEIGHT - 82
    svg.append(
        f'<rect x="{legend_x - 10}" y="{legend_y - 23}" width="222" height="91" rx="4" fill="white" opacity="0.92" stroke="#bbb"/>'
    )
    for index, (name, _, color) in enumerate(series):
        y = legend_y + index * 27
        svg.extend(
            [
                f'<line x1="{legend_x}" y1="{y}" x2="{legend_x + 34}" y2="{y}" stroke="{color}" stroke-width="3"/>',
                f'<circle cx="{legend_x + 17}" cy="{y}" r="3.4" fill="white" stroke="{color}" stroke-width="2"/>',
                f'<text class="legend" x="{legend_x + 45}" y="{y + 5}">{name}</text>',
            ]
        )

    svg.append("</svg>")
    args.svg.write_text("\n".join(svg) + "\n", encoding="utf-8")


def decade_ticks(low: float, high: float) -> list[float]:
    return [
        10.0**exponent
        for exponent in range(math.floor(math.log10(low)), math.ceil(math.log10(high)) + 1)
        if low <= 10.0**exponent <= high
    ]


def format_count(value: int) -> str:
    if value >= 1_000_000:
        return f"{value / 1_000_000:g}M"
    if value >= 1_000:
        return f"{value / 1_000:g}k"
    return str(value)


def format_time(value: float) -> str:
    if value < 0.001:
        return f"{value * 1_000:g} µs"
    if value < 1:
        return f"{value:g} ms"
    return f"{value:g} ms"


if __name__ == "__main__":
    main()
