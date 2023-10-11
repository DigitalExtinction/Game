#!/usr/bin/env python3

import sys
import json

MAP_SIZE = 1000.
MARGIN = 50.

def main():
    geojson = json.load(sys.stdin)

    points = []
    polygons = []

    for feature in geojson['features']:
        geometry = feature['geometry']
        geometry_type = geometry['type']
        coordinates = geometry['coordinates']

        if geometry_type == 'Point':
            assert len(coordinates) == 2
            name = feature['properties']['name']
            points.append((coordinates[0], coordinates[1], name))
        elif geometry_type == 'Polygon':
            assert len(coordinates) == 1  # no inner rings
            outer_ring = coordinates[0]
            assert len(outer_ring) >= 4  # at least a triangle
            assert outer_ring[0] == outer_ring[-1]
            vertices = [(v[0], v[1]) for v in outer_ring[:-1]]
            polygons.append(vertices)
        else:
            raise Exception(f'Unexpected geometry type: {geometry_type}')

    x_min, y_min, _ = points[0]
    x_max, y_max = x_min, y_min

    for (x, y, _) in points:
        x_min = min(x_min, x)
        x_max = max(x_max, x)
        y_min = min(y_min, y)
        y_max = max(y_max, y)

    for polygon in polygons:
        for (x, y) in polygon:
            x_min = min(x_min, x)
            x_max = max(x_max, x)
            y_min = min(y_min, y)
            y_max = max(y_max, y)

    x_shift = (x_max + x_min) / 2.
    x_scale = (MAP_SIZE - MARGIN) / (x_max - x_min)
    y_shift = (y_max + y_min) / 2.
    y_scale = (MAP_SIZE - MARGIN) / (y_max - y_min)

    for (x, y, name) in points:
        x = x_scale * (x - x_shift)
        y = y_scale * (y - y_shift)
        print(f'Point {name}: ({x}, {y})')

    for polygon in polygons:
        print('Ichnography::from(ConvexPolygon::from_convex_hull(&[')
        for x, y in polygon:
            x = x_scale * (x - x_shift)
            y = y_scale * (y - y_shift)
            print(f'Point::new({x}, {y}),')
        print(']).unwrap()),')


if __name__ == '__main__':
    main()
