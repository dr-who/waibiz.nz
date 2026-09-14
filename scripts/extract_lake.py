#!/usr/bin/env python3
"""Convert the OSM Lake Taupo multipolygon relation into Rust canvas points."""

import json
import math
from pathlib import Path

SOURCE = Path("/tmp/lake-taupo-osm.json")
OUTPUT = Path("src/lake_geometry.rs")


def distance(point, start, end):
    dx = end[0] - start[0]
    dy = end[1] - start[1]
    if dx == 0 and dy == 0:
        return math.hypot(point[0] - start[0], point[1] - start[1])
    t = ((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / (dx * dx + dy * dy)
    projection = (start[0] + t * dx, start[1] + t * dy)
    return math.hypot(point[0] - projection[0], point[1] - projection[1])


def simplify(points, tolerance):
    if len(points) < 3:
        return points
    farthest = max(range(1, len(points) - 1), key=lambda index: distance(points[index], points[0], points[-1]))
    maximum = distance(points[farthest], points[0], points[-1])
    if maximum <= tolerance:
        return [points[0], points[-1]]
    return simplify(points[: farthest + 1], tolerance)[:-1] + simplify(points[farthest:], tolerance)


data = json.loads(SOURCE.read_text())
members = data["elements"][0]["members"]
segments = [
    [(point["lon"], point["lat"]) for point in member["geometry"]]
    for member in members
    if member.get("role") == "outer" and member.get("geometry")
]

ring = segments.pop(0)
while segments:
    endpoint = ring[-1]
    for index, segment in enumerate(segments):
        if segment[0] == endpoint:
            ring.extend(segment[1:])
            segments.pop(index)
            break
        if segment[-1] == endpoint:
            ring.extend(reversed(segment[:-1]))
            segments.pop(index)
            break
    else:
        raise RuntimeError(f"Could not join shoreline after {endpoint}")

if ring[0] != ring[-1]:
    ring.append(ring[0])

# Split the closed ring in two so Ramer-Douglas-Peucker retains the shoreline shape.
split = len(ring) // 2
ring = simplify(ring[: split + 1], 0.00035)[:-1] + simplify(ring[split:], 0.00035)

mean_latitude = sum(point[1] for point in ring) / len(ring)
longitude_scale = math.cos(math.radians(mean_latitude))
physical = [(lon * longitude_scale, -lat) for lon, lat in ring]
minimum_x = min(point[0] for point in physical)
maximum_x = max(point[0] for point in physical)
minimum_y = min(point[1] for point in physical)
maximum_y = max(point[1] for point in physical)
width = maximum_x - minimum_x
height = maximum_y - minimum_y
normalized = [((x - minimum_x) / width, (y - minimum_y) / height) for x, y in physical]

lines = [
    "// Generated from OpenStreetMap relation 1130806 (Lake Taupo).",
    "// Source data (c) OpenStreetMap contributors, ODbL 1.0.",
    f"pub const TAUPO_ASPECT: f64 = {width / height:.6f};",
    "pub const TAUPO_SHORE: &[(f64, f64)] = &[",
]
lines.extend(f"    ({x:.6f}, {y:.6f})," for x, y in normalized)
lines.append("];\n")
OUTPUT.write_text("\n".join(lines))
print(f"Wrote {len(normalized)} Lake Taupo points to {OUTPUT}")
