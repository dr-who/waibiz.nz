#!/usr/bin/env python3
"""Convert the OSM Waikato River relation into a compact Rust point array."""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path


def distance(a: tuple[float, float], b: tuple[float, float]) -> float:
    lat = math.radians((a[0] + b[0]) / 2)
    dx = (a[1] - b[1]) * math.cos(lat)
    dy = a[0] - b[0]
    return math.hypot(dx, dy)


def point_line_distance(point, start, end):
    if start == end:
        return distance(point, start)
    x, y = point[1], point[0]
    x1, y1 = start[1], start[0]
    x2, y2 = end[1], end[0]
    denominator = (x2 - x1) ** 2 + (y2 - y1) ** 2
    t = max(0.0, min(1.0, ((x - x1) * (x2 - x1) + (y - y1) * (y2 - y1)) / denominator))
    projection = (y1 + t * (y2 - y1), x1 + t * (x2 - x1))
    return distance(point, projection)


def simplify(points, tolerance):
    if len(points) <= 2:
        return points
    maximum = 0.0
    index = 0
    for candidate in range(1, len(points) - 1):
        value = point_line_distance(points[candidate], points[0], points[-1])
        if value > maximum:
            index, maximum = candidate, value
    if maximum > tolerance:
        left = simplify(points[: index + 1], tolerance)
        right = simplify(points[index:], tolerance)
        return left[:-1] + right
    return [points[0], points[-1]]


def main(source: Path, target: Path) -> None:
    relation = json.loads(source.read_text())["elements"][0]
    segments = [
        [(point["lat"], point["lon"]) for point in member["geometry"]]
        for member in relation["members"]
        if member["type"] == "way" and member.get("geometry")
    ]

    # Taupō is the southernmost endpoint in this relation.
    start_segment = max(range(len(segments)), key=lambda i: segments[i][0][0])
    # The numeric comparison is inverted for southern latitudes; choose explicitly.
    start_segment = min(range(len(segments)), key=lambda i: min(segments[i][0][0], segments[i][-1][0]))
    segment = segments.pop(start_segment)
    if segment[0][0] > segment[-1][0]:
        segment.reverse()
    ordered = segment

    while segments:
        current = ordered[-1]
        candidates = []
        for index, candidate in enumerate(segments):
            candidates.append((distance(current, candidate[0]), index, False))
            candidates.append((distance(current, candidate[-1]), index, True))
        gap, index, reverse = min(candidates)
        if gap > 0.003:
            break
        candidate = segments.pop(index)
        if reverse:
            candidate.reverse()
        ordered.extend(candidate[1:] if distance(current, candidate[0]) < 0.0002 else candidate)

    ordered = simplify(ordered, 0.00016)
    lengths = [0.0]
    for previous, current in zip(ordered, ordered[1:]):
        lengths.append(lengths[-1] + distance(previous, current))
    total = lengths[-1]
    longitudes = [point[1] for point in ordered]
    west, east = min(longitudes), max(longitudes)

    lines = [
        "// Generated from OpenStreetMap relation 2751038 (Waikato River).",
        "// Source data © OpenStreetMap contributors, ODbL 1.0.",
        "pub const WAIKATO_RIVER: &[(f64, f64)] = &[",
    ]
    for (_, longitude), travelled in zip(ordered, lengths):
        x = (longitude - west) / (east - west)
        progress = travelled / total
        lines.append(f"    ({progress:.6f}, {x:.6f}),")
    lines.append("];")
    target.write_text("\n".join(lines) + "\n")
    print(f"wrote {len(ordered)} points to {target}")


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
