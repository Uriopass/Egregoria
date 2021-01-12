import polyskel
import collections
import math
import json
from shapely.geometry import MultiPolygon, box
from shapely.ops import cascaded_union
import random

random.seed(3)


def length(v):
    return math.sqrt(sum(map(lambda x: x * x, v.values())))


def normalize(a):
    l = length(a)
    return {v: a[v] / l for v in a}


def sub(a, b):
    return {v: a[v] - b[v] for v in a}


def cross(a, b):
    return {"x": a["y"] * b["z"] - a["z"] * b["y"],
            "y": a["z"] * b["x"] - a["x"] * b["z"],
            "z": a["x"] * b["y"] - a["y"] * b["x"]}


def dot(a, b):
    return sum(map(lambda x: x[0] * x[1], zip(a.values(), b.values())))


def normal(face):
    p1 = face[2]
    p2 = face[1]
    p3 = face[0]
    a = sub(p1, p2)
    b = sub(p1, p3)
    return normalize(cross(a, b))


r = lambda: random.random()


def near(v):
    return int(v * 3.0)


def gen_plan_exterior():
    n_rect = int(r() * r() * 5 + 2)

    poly = None
    for i in range(n_rect):
        x = near(r() * 8.0)
        y = near(r() * 8.0)
        w = near(5.0 + r() * 10.0)
        h = near(5.0 + r() * 10.0)
        # print(x, y, w, h)
        b = box(x, y, x + w, y + h)
        if poly is None:
            poly = b
        else:
            poly = union(poly, b)
    if isinstance(poly, MultiPolygon):
        return gen_plan_exterior()
    return poly


def gen_plan_skel():
    while True:
        ext = gen_plan_exterior()
        skeleton = mk_skel(ext)
        if skeleton is None:
            continue
        for face in skeleton:
            for (a, b) in zip(face, face[1:]):
                if abs(a["x"] - b["x"]) + abs(a["y"] - b["y"]) < 2:
                    return gen_plan_skel()
        return ext, skeleton


def union(a, b):
    return cascaded_union([a, b]).simplify(0.001)


def main():
    s = []
    for _ in range(100):
        poly, skel = gen_plan_skel()
        s.append({"faces": skel})
        # showlithouse(skel)
    print(json.dumps({"houses": s}), file=open("generated/houses.json", "w+"))


def showlithouse(skel):
    import matplotlib.pyplot as plt
    sun = normalize({"x": 0.8, "y": 0.5, "z": 0.5})
    l2 = normalize({"x": -0.8, "y": -0.5, "z": 0.5})

    def showpoly(poly, col=None):
        x = list(map(lambda x: x[0], poly))
        y = list(map(lambda x: x[1], poly))

        plt.fill(x, y, color=col)
        # plt.scatter(x, y)
        plt.axis('equal')

    for face in skel:
        n = normal(face)
        vv = 0.3 + max(0., 0.6 * dot(n, sun) + 0.1 * dot(n, l2))
        col = (vv, vv, vv)
        showpoly(list(map(lambda v: (v["x"], v["y"]), face)), col=col)

    plt.show()


def mk_skel(poly):
    # poly = list(reversed(poly.exterior.coords[:-1]))
    poly = list(poly.exterior.coords[:-1])

    skeleton = polyskel.skeletonize(poly)

    graph = collections.defaultdict(list)
    heights = {}

    for i, p in enumerate(poly):
        graph[p].append(poly[i - 1])
        graph[poly[i - 1]].append(p)
        heights[p] = 0.0

    for res in skeleton:
        t = (res.source.x, res.source.y)
        if abs(t[0]) + abs(t[1]) > 1e5:
            return None
        heights[t] = res.height
        for v in res.sinks:
            if abs(v.x) + abs(v.y) > 1e5:
                return None
            graph[t].append((v.x, v.y))
            graph[(v.x, v.y)].append(t)

    def angle(a, b):
        diff = (b[0] - a[0], b[1] - a[1])
        return math.atan2(diff[0], diff[1])  # clockwise inverse arguments for anti clockwise

    for c, v in graph.items():
        # print(c, heights[c])
        v.sort(key=lambda e: angle(c, e))
        # for j in v:
        #    print("   ", j)

    faces = []
    visited = set()

    def next_v(start, nextt):
        i = graph[nextt].index(start)
        return nextt, graph[nextt][i - 1]

    def explore(start, nextt):
        face = [start]
        visited.add((start, nextt))
        cur = next_v(start, nextt)
        while cur[0] != start:
            visited.add(cur)
            face.append(cur[0])
            cur = next_v(cur[0], cur[1])
        return face

    expected_around = [poly[0]] + list(reversed(poly[1:]))

    for node in graph:
        for edge in graph[node]:
            if (node, edge) not in visited:
                face = explore(node, edge)
                if face == expected_around:
                    continue
                faces.append(face)

    encoded = []
    for face in faces:
        f = []
        for v in face:
            f.append({"x": v[0], "y": v[1], "z": heights[v]})
        encoded.append(f)
    return encoded


main()
