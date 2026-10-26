"""Generated MD3 models for the Sith maps: the hooded statues, and the MD3 writer.

The statues face +x and are 100 units tall: robed figures whose robe is a
height field round their axis (folds deepening to a rolled hem, a cape flaring
behind with a raised edge, shoulders, a hood reaching over a face set back in
its shadow) with parts built on it: the hood's rolled rim, a stern face with
glowing eyes, wide folded sleeves with cuffs, hands with fingers, a belt with
buckle, pouches and sash, a chained clasp, boots, and their hilts or sword, in
several poses ([`STATUES`]). Surfaces (materials): the stone (`statue`,
`statue_dark`, `statue_bronze`), `face` (the hood's shadow, the sleeves'
openings), `hilt` (metal), `eyes` (glowing red), and a lit saber's `blade` and
`blade_core`. Triangles are built counter-clockwise seen from outside and
written clockwise, as Quake 3 draws them.
"""
import math
import struct


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def scale(a, k):
    return tuple(x * k for x in a)


def norm(a):
    length = math.sqrt(sum(x * x for x in a)) or 1.0
    return tuple(x / length for x in a)


def lerp(points, h):
    """Linear interpolation through (x, y) points sorted by x."""
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        if x0 <= h <= x1:
            return y0 + (y1 - y0) * (h - x0) / (x1 - x0)
    return points[-1][1] if h > points[-1][0] else points[0][1]


class Mesh:
    """Triangles by material; vertices are per material and duplicated as needed."""

    def __init__(self):
        self.parts = {}

    def triangle(self, material, a, b, c, uva, uvb, uvc):
        verts, uvs, tris = self.parts.setdefault(material, ([], [], []))
        base = len(verts)
        verts.extend((a, b, c))
        uvs.extend((uva, uvb, uvc))
        tris.append((base, base + 1, base + 2))

    def grid(self, rows, uvs, material, wrap=True):
        """Quads between consecutive rows of points; `material(i, j)` names each
        quad's (row i, column j) material. Rows run along the surface's axis with
        each row counter-clockwise around it, so the quads face outwards."""
        width = len(rows[0])
        columns = width if wrap else width - 1
        for i in range(len(rows) - 1):
            for j in range(columns):
                k = (j + 1) % width
                a, b, c, d = rows[i][j], rows[i][k], rows[i + 1][j], rows[i + 1][k]
                ua, ub, uc, ud = uvs[i][j], uvs[i][k], uvs[i + 1][j], uvs[i + 1][k]
                m = material(i, j)
                if a != b:
                    self.triangle(m, a, b, c, ua, ub, uc)
                if c != d:
                    self.triangle(m, b, d, c, ub, ud, uc)

    def cap(self, ring, center, material, facing_forward):
        """A fan closing a ring; it faces along the ring's axis or against it."""
        n = len(ring)
        for j in range(n):
            a, b = ring[j], ring[(j + 1) % n]
            if facing_forward:
                self.triangle(material, center, a, b, (0.5, 0.5), (0, 0), (1, 0))
            else:
                self.triangle(material, center, b, a, (0.5, 0.5), (1, 0), (0, 0))

    def surfaces(self):
        return {name: (list(v), list(uv), list(t)) for name, (v, uv, t) in self.parts.items()}


def frame_for(tangent):
    """Two unit vectors completing a right-handed frame (u, v, tangent)."""
    t = norm(tangent)
    up = (0, 0, 1) if abs(t[2]) < 0.9 else (1, 0, 0)
    u = norm(cross(up, t))
    v = cross(t, u)
    return u, v


def tube(mesh, path, radii, material, segments=12, cap_end=None, squash=(1.0, 1.0)):
    """A tube along `path` with a radius per point; `cap_end` closes its far end.
    `squash` stretches the cross-section along the frame's two axes (a blade's)."""
    rows, uvs = [], []
    for i, (p, r) in enumerate(zip(path, radii)):
        ahead = path[min(i + 1, len(path) - 1)]
        behind = path[max(i - 1, 0)]
        u, v = frame_for(sub(ahead, behind))
        ring = []
        for j in range(segments):
            a = 2 * math.pi * j / segments
            ring.append(add(p, add(scale(u, r * squash[0] * math.cos(a)),
                                   scale(v, r * squash[1] * math.sin(a)))))
        rows.append(ring)
        uvs.append([(j / segments, i / 4) for j in range(segments)])
    mesh.grid(rows, uvs, lambda i, j: material)
    if cap_end:
        mesh.cap(rows[-1], path[-1], cap_end, True)


def ellipsoid(mesh, c, r, material, rings=8, segments=16):
    rows, uvs = [], []
    for i in range(rings + 1):  # bottom pole to top pole, so rows run up the z axis
        phi = math.pi * (1 - i / rings)
        row = []
        for j in range(segments):
            th = 2 * math.pi * j / segments
            row.append((c[0] + r[0] * math.sin(phi) * math.cos(th),
                        c[1] + r[1] * math.sin(phi) * math.sin(th),
                        c[2] + r[2] * math.cos(phi)))
        rows.append(row)
        uvs.append([(j / segments, i / rings) for j in range(segments)])
    mesh.grid(rows, uvs, lambda i, j: material)


def cylinder(mesh, x, y, z0, z1, r, material, segments=12):
    rows = []
    for z in (z0, z1):
        rows.append([(x + r * math.cos(2 * math.pi * j / segments),
                      y + r * math.sin(2 * math.pi * j / segments), z) for j in range(segments)])
    uvs = [[(j / segments, z / 25) for j in range(segments)] for z in (z0, z1)]
    mesh.grid(rows, uvs, lambda i, j: material)
    mesh.cap(rows[1], (x, y, z1), material, True)
    mesh.cap(rows[0], (x, y, z0), material, False)


# ---------------------------------------------------------------- shapes

def rotate_toward(d, n, angle):
    """(d, n) turned by `angle` in their plane: d towards -n (a finger curling to
    its palm, `n` the back of the hand), both kept unit and perpendicular."""
    c, s = math.cos(angle), math.sin(angle)
    return norm(sub(scale(d, c), scale(n, s))), norm(add(scale(d, s), scale(n, c)))


def blob(mesh, c, axes, radii, material, rings=8, segments=14):
    """An ellipsoid at `c` with its radii along the three unit `axes`."""
    rows, uvs = [], []
    u, v, w = axes
    for i in range(rings + 1):
        phi = math.pi * (1 - i / rings)
        row = []
        for j in range(segments):
            th = 2 * math.pi * j / segments
            a, b, z = (radii[0] * math.sin(phi) * math.cos(th), radii[1] * math.sin(phi) * math.sin(th),
                       radii[2] * math.cos(phi))
            row.append(add(c, add(scale(u, a), add(scale(v, b), scale(w, z)))))
        rows.append(row)
        uvs.append([(j / segments, i / rings) for j in range(segments)])
    # the three axes may be left-handed: then the rows run the other way round
    flip = sum(cross(u, v)[k] * w[k] for k in range(3)) < 0
    mesh.grid([list(reversed(r)) for r in rows] if flip else rows, uvs, lambda i, j: material)


def band(mesh, z, radius, profile, material, segments=96, uv_scale=8.0):
    """A ring round the figure's axis at height z, `radius(theta)` out, with a closed
    cross-section `profile` of (outwards, up) offsets listed counter-clockwise seen
    from the +theta side: a belt, a cuff, a hem."""
    rows, uvs = [], []
    loop = profile + profile[:1]
    for k, (dr, dz) in enumerate(loop):
        row = []
        for j in range(segments):
            th = 2 * math.pi * j / segments
            r = radius(th) + dr
            row.append((r * math.cos(th), r * math.sin(th), z + dz))
        rows.append(row)
        uvs.append([(uv_scale * j / segments, k / 4) for j in range(segments)])
    mesh.grid(rows, uvs, lambda i, j: material)


def rounded_box_profile(thick, height, bevel=0.35):
    """A belt's or hem's cross-section: (outwards, up) round a rounded rectangle whose
    inner side sits a little inside the robe."""
    t, h, b = thick, height / 2, bevel
    return [(-0.4, -h), (t - b, -h), (t, -h + b), (t, h - b), (t - b, h), (-0.4, h)]


def smooth(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)


# ---------------------------------------------------------------- the statues

# The robe's radius by height (a hooded figure 100 tall facing +x), before the
# shoulders, the cape, the folds and the hood's opening shape it.
PROFILE = [(0, 22.5), (1.5, 22.2), (8, 21.2), (18, 19.8), (30, 18.4), (40, 16.8), (45, 15.6),
           (47, 15.2), (50, 15.4), (56, 16.0), (62, 16.6), (68, 17.0), (72, 17.0), (75, 16.6),
           (78, 15.4), (80, 14.0), (82, 13.0), (85, 12.7), (88, 12.6), (91, 12.2), (94, 11.2),
           (96.5, 9.4), (98.5, 6.6), (99.6, 3.4), (100, 0.6)]
NECK = [(80, 14.0), (81.5, 9.0), (83.5, 7.6)]
SEGMENTS = 112
FACE_HALF_ANGLE = math.radians(52)  # the hood's opening, either side of +x
FACE_CENTER, FACE_HALF_HEIGHT = 88.3, 5.6
SHOULDERS = {1: (0.5, 16.0, 71.5), -1: (0.5, -16.0, 71.5)}


def heights(hood=True):
    out = [0, 0.5, 1.1, 1.8, 2.6, 3.5, 4.6, 6, 7.5, 9]
    out += [10 + 2 * k for k in range(17)]  # to 42
    out += [43 + 0.75 * k for k in range(13)]  # the belt, 43 to 52
    out += [53 + 1.25 * k for k in range(9)]  # 53 to 63
    out += [64 + 1.0 * k for k in range(16)]  # 64 to 79
    if hood:
        out += [80 + 0.5 * k for k in range(40)] + [100]
    else:
        out += [80, 81, 81.5, 82.5, 83.5]
    return out


def body_radius(h, theta, cape, hood=True):
    r = lerp(PROFILE if hood or h <= 80 else PROFILE[:16] + NECK, h)
    c, s = math.cos(theta), math.sin(theta)
    back = max(0.0, -c)
    k = 1.0
    # shoulders: wider sideways, sloping into the hood
    k *= 1 + 0.19 * s * s * math.exp(-((h - 71.5) / 6.5) ** 2)
    # chest forward, shoulder blades behind
    k *= 1 + 0.06 * max(0.0, c) ** 2 * math.exp(-((h - 64) / 7) ** 2)
    k *= 1 + 0.04 * back ** 2 * math.exp(-((h - 70) / 6) ** 2)
    # the cape: it hangs from the shoulders and flares behind to the ground, folded
    # deep, with a raised edge where it meets the robe
    if h < 78:
        drop = (1 - h / 78)
        w = smooth((back - 0.15) / 0.5)
        k *= 1 + cape * w * drop ** 1.3
        k *= 1 + w * drop ** 0.6 * (0.05 * math.sin(7 * theta + 0.6) + 0.02 * math.sin(13 * theta))
        edge = abs(abs(theta) - 1.95)
        k *= 1 + 0.03 * math.exp(-(edge / 0.07) ** 2) * min(1.0, (78 - h) / 6)
    # the skirt's folds, deeper and wider apart towards the hem, gathered under the belt
    if h < 47:
        d = 1 - h / 47
        sway = 0.9 * math.sin(2 * theta) + 0.35 * d
        k *= 1 + d ** 0.75 * (0.055 * math.sin(11 * theta + sway) + 0.022 * math.sin(23 * theta + 1.7)
                              + 0.012 * math.sin(37 * theta + 0.4))
    # a rolled hem
    if h < 3.2:
        k *= 1 + 0.025 * math.sin(math.pi * h / 3.2)
    # the robe drawn in by the belt, soft folds over the chest
    if 43 <= h <= 52:
        k *= 1 + 0.02 * math.sin(math.pi * (h - 43) / 9) * math.sin(15 * theta)
    if 50 <= h <= 66:
        k *= 1 + 0.014 * math.sin(math.pi * (h - 50) / 16) * math.sin(9 * theta + 0.15 * (h - 50))
    # the hood's short mantle over chest and back, its scalloped edge
    if hood and 66 <= h <= 81:  # cloth gathered under the hood
        k *= 1 + 0.01 * math.sin(14 * theta + 0.3 * h) * math.sin(math.pi * (h - 66) / 15)
    if hood:
        # the brim reaches forward over the face, the face sits back inside it
        if 91.5 <= h <= 99.6:
            k *= 1 + 0.32 * max(0.0, c) ** 2 * math.sin(math.pi * (h - 91.5) / 8.1)
        if 82 <= h <= 93:
            k *= 1 - 0.34 * max(0.0, c) ** 3 * math.sin(math.pi * (h - 82) / 11)
        # cloth gathered behind the hood
        if 80 <= h <= 97:
            k *= 1 + 0.025 * back * math.sin(math.pi * (h - 80) / 17) * math.sin(6 * theta)
    return r * k


def surface_point(h, theta, cape, out=0.0, lean=True):
    """A point on the robe at height h and angle theta, `out` units off it."""
    r = body_radius(h, theta, cape) + out
    dx = -3.0 * max(0.0, (h - 84) / 16) if lean else 0.0
    return (dx + r * math.cos(theta), r * math.sin(theta), h)


def rod(mesh, a, b, r, material, segments=12):
    """A closed cylinder from a to b."""
    tube(mesh, [a, b], [r, r], material, segments, cap_end=material)
    u, v = frame_for(sub(b, a))
    ring = [add(a, add(scale(u, r * math.cos(2 * math.pi * j / segments)),
                       scale(v, r * math.sin(2 * math.pi * j / segments)))) for j in range(segments)]
    mesh.cap(ring, a, material, False)


def saber(mesh, base, direction, blade):
    """A hilt from `base` along `direction` (emitter last), and its lit blade."""
    d = norm(direction)
    at = lambda k: add(base, scale(d, k))  # noqa: E731
    rod(mesh, at(-1), at(10), 1.6, "hilt", 16)
    rod(mesh, at(-1.8), at(-0.8), 2.1, "hilt", 16)  # pommel
    rod(mesh, at(-2.4), at(-1.8), 1.3, "hilt", 12)
    for k in (1.6, 2.8, 4.0, 5.2, 6.4):  # grip ridges
        rod(mesh, at(k), at(k + 0.55), 1.85, "hilt", 16)
    rod(mesh, at(7.4), at(8.3), 2.0, "hilt", 16)
    rod(mesh, at(9), at(11.5), 2.4, "hilt", 16)  # emitter shroud
    u, _ = frame_for(d)
    blob(mesh, add(at(7.9), scale(u, 1.9)), (u, norm(cross(d, u)), d), (0.6, 0.6, 0.8), "eyes", 4, 8)
    if blade:
        rod(mesh, at(11.5), at(11.5 + blade), 1.25, "blade", 10)
        rod(mesh, at(11.5), at(11.5 + blade + 0.6), 0.55, "blade_core", 8)


def hand(mesh, wrist, forward, back, stone, curl=1.1, spread=0.12, thumb=1, size=1.0):
    """A hand from `wrist`: a palm along `forward` with its back towards `back`, four
    fingers bent `curl` radians over their three joints, and a thumb on the `thumb`
    side (+1 or -1 of forward x back)."""
    f = norm(forward)
    n = norm(sub(back, scale(f, sum(back[k] * f[k] for k in range(3)))))
    side = cross(f, n)
    palm = add(wrist, scale(f, 2.0 * size))
    blob(mesh, palm, (f, side, n), (2.3 * size, 2.0 * size, 0.95 * size), stone, 6, 12)
    blob(mesh, add(wrist, scale(f, 0.2 * size)), (f, side, n), (1.4 * size, 1.6 * size, 1.0 * size),
         stone, 5, 10)
    for k, (off, length) in enumerate(((-1.35, 1.25), (-0.45, 1.5), (0.45, 1.45), (1.35, 1.1))):
        d, nb = f, n
        # fan the fingers a little, side to side
        d = norm(add(d, scale(side, spread * (k - 1.5))))
        p = add(palm, add(scale(f, 1.9 * size), add(scale(side, off * size), scale(n, 0.15 * size))))
        path, radii = [p], [0.55 * size]
        for joint, part in enumerate((1.0, 0.75, 0.6)):
            d, nb = rotate_toward(d, nb, curl * (0.8 if joint == 0 else 1.0))
            p = add(p, scale(d, length * part * size * 1.15))
            path.append(p)
            radii.append((0.52 - 0.06 * joint) * size)
        tube(mesh, path, radii, stone, 8, cap_end=stone)
        blob(mesh, path[-1], (d, cross(d, nb), nb), (0.45 * size, 0.42 * size, 0.42 * size), stone, 4, 8)
    t0 = add(wrist, add(scale(f, 0.9 * size), add(scale(side, thumb * 1.5 * size), scale(n, -0.5 * size))))
    t_dir = norm(add(scale(f, 0.8), add(scale(side, thumb * 0.5), scale(n, -0.5))))
    t1 = add(t0, scale(t_dir, 1.6 * size))
    in_dir = norm(add(scale(f, 0.6), add(scale(side, -thumb * (0.3 + 0.5 * curl)), scale(n, -0.4))))
    t2 = add(t1, scale(in_dir, 1.4 * size))
    tube(mesh, [t0, t1, t2], [0.62 * size, 0.55 * size, 0.47 * size], stone, 8, cap_end=stone)


def arm(mesh, side, elbow, wrist, stone, cape, hand_args=None, sleeve=(4.6, 5.0, 5.6, 6.6)):
    """A wide sleeve from the shoulder through `elbow` to `wrist`, folded, with a cuff
    and a dark opening, and the hand coming out of it."""
    shoulder = SHOULDERS[side]
    mid1 = tuple((a * 2 + b) / 3 for a, b in zip(shoulder, elbow))
    mid2 = tuple((a + b * 2) / 3 for a, b in zip(elbow, wrist))
    out = norm(sub(wrist, elbow))
    cuff = add(wrist, scale(out, -1.6))
    path = [shoulder, mid1, elbow, mid2, cuff]
    radii = [sleeve[0], sleeve[1], sleeve[2], (sleeve[2] + sleeve[3]) / 2, sleeve[3]]
    # a sleeve is a tube rippled into folds along it
    rows, uvs = [], []
    segments = 24
    for i, (p, r) in enumerate(zip(path, radii)):
        ahead, behind = path[min(i + 1, len(path) - 1)], path[max(i - 1, 0)]
        u, v = frame_for(sub(ahead, behind))
        ring = []
        for j in range(segments):
            a = 2 * math.pi * j / segments
            rr = r * (1 + 0.07 * math.sin(5 * a + 1.3 * i) * (0.4 + 0.6 * i / 4))
            ring.append(add(p, add(scale(u, rr * math.cos(a)), scale(v, rr * math.sin(a)))))
        rows.append(ring)
        uvs.append([(j / segments, i / 4) for j in range(segments)])
    mesh.grid(rows, uvs, lambda i, j: stone)
    mesh.cap(rows[-1], path[-1], "face", True)
    # the cuff: a thick rolled rim
    u, v = frame_for(out)
    ring = []
    for j in range(25):
        a = 2 * math.pi * j / 24
        rr = sleeve[3] * (1 + 0.07 * math.sin(5 * a + 1.3 * 4))
        ring.append(add(cuff, add(scale(u, rr * math.cos(a)), scale(v, rr * math.sin(a)))))
    tube(mesh, ring, [0.9] * len(ring), stone, 8)
    hand(mesh, add(wrist, scale(out, -1.2)), out, **(hand_args or {"back": (0, 0, 1)}),
         stone=stone, size=1.35)


def face_features(mesh, stone, eyes=True):
    """In the hood's shadow: a hint of a stern face, its chin, mouth and the tip of
    its nose catching the light, and the eyes glowing."""
    x = surface_point(88, 0, 0)[0]  # the face's plane, set back in the hood
    axes = ((1, 0, 0), (0, 1, 0), (0, 0, 1))
    blob(mesh, (x + 0.2, 0, 84.4), axes, (1.5, 2.8, 1.6), stone, 6, 14)  # chin
    blob(mesh, (x + 0.75, 0, 86.0), axes, (0.5, 1.4, 0.28), "face", 4, 8)  # mouth
    blob(mesh, (x + 0.1, 0, 88.0), axes, (1.1, 0.8, 1.6), stone, 6, 10)  # nose
    if eyes:
        for y in (-1, 1):
            blob(mesh, (x + 0.2, y * 1.8, 89.8), axes, (0.4, 0.8, 0.3), "eyes", 4, 8)


def hood_rim(mesh, stone, cape):
    """The rolled edge of the hood round its opening."""
    path = []
    for k in range(49):
        t = 2 * math.pi * k / 48
        th = 1.08 * FACE_HALF_ANGLE * math.cos(t)
        h = FACE_CENTER + 1.12 * FACE_HALF_HEIGHT * math.sin(t)
        path.append(surface_point(h, th, cape, 0.15))
    tube(mesh, path, [0.95] * len(path), stone, 8)


def belt(mesh, stone, cape):
    """A broad belt with stitched edges, a round buckle, pouches and a sash hanging
    from it in front."""
    radius = lambda th: body_radius(47, th, cape)  # noqa: E731
    band(mesh, 47, radius, rounded_box_profile(1.2, 4.6), "hilt")
    for z in (45.2, 48.8):
        band(mesh, z, lambda th: radius(th) + 1.1, rounded_box_profile(0.35, 0.5, 0.15), "hilt")
    x = radius(0) + 1.2
    rod(mesh, (x - 0.5, 0, 47), (x + 0.7, 0, 47), 3.2, "hilt", 20)
    rod(mesh, (x + 0.7, 0, 47), (x + 1.2, 0, 47), 2.2, "hilt", 20)
    blob(mesh, (x + 1.5, 0, 47), ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (0.6, 1.0, 1.4), "eyes", 4, 8)
    for y in (-1, 1):  # pouches
        th = y * 1.15
        p = surface_point(44.6, th, cape, 1.3, False)
        out = (math.cos(th), math.sin(th), 0)
        blob(mesh, p, (out, (-math.sin(th), math.cos(th), 0), (0, 0, 1)), (1.6, 2.4, 2.8), stone, 6, 12)
        blob(mesh, add(p, (0, 0, 2.2)), (out, (-math.sin(th), math.cos(th), 0), (0, 0, 1)),
             (1.9, 2.6, 0.7), stone, 4, 12)
    for th, end in ((0.22, 17), (0.36, 22)):  # the sash's two tails
        path = [surface_point(h, th + 0.02 * math.sin(h / 4), cape, 1.6, False)
                for h in range(45, end - 1, -3)]
        tube(mesh, path, [1.25] * len(path), stone, 6, squash=(2.4, 0.3))
        blob(mesh, add(path[-1], (0, 0, -1.0)), ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (0.8, 2.6, 1.3),
             stone, 4, 10)


def clasp(mesh, stone, cape):
    """The cape's clasp: two medallions on the chest and the chain between them."""
    ends = []
    for y in (-1, 1):
        th = y * 0.62
        p = surface_point(74.5, th, cape, 0.5, False)
        out = norm((math.cos(th), math.sin(th), 0))
        rod(mesh, p, add(p, scale(out, 0.8)), 2.0, "hilt", 16)
        blob(mesh, add(p, scale(out, 0.9)), (out, (-out[1], out[0], 0), (0, 0, 1)), (0.5, 0.9, 0.9),
             "eyes", 4, 8)
        ends.append(add(p, scale(out, 0.6)))
    for k in range(1, 10):
        t = k / 10
        p = tuple(ends[0][i] * (1 - t) + ends[1][i] * t for i in range(3))
        p = add(p, (1.6 * math.sin(math.pi * t), 0, -2.6 * math.sin(math.pi * t)))
        blob(mesh, p, ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (0.45, 0.6, 0.45), "hilt", 4, 8)


def boots(mesh, stone):
    """The toes of two boots under the hem."""
    for side in (1, -1):
        y = side * 7.5
        blob(mesh, (19.5, y, 1.9), ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (6.0, 3.6, 2.4), stone, 6, 14)
        blob(mesh, (22.5, y, 1.2), ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (3.2, 3.2, 1.3), stone, 5, 12)


def helmet(mesh, stone, crest):
    """A Sith helm on the neck: a dome, a dark face plate, glowing eye slits."""
    ellipsoid(mesh, (0, 0, 90), (9, 8.6, 10.5), stone, 10, 18)
    ellipsoid(mesh, (5.2, 0, 88.5), (5, 7.4, 7.6), "face", 8, 14)
    for side in (1, -1):
        ellipsoid(mesh, (9.9, side * 3, 91), (0.9, 2.2, 0.8), "eyes", 4, 8)
        ellipsoid(mesh, (0.5, side * 8.4, 88), (2.5, 1.6, 6), stone, 6, 10)  # cheek guards
    if crest:
        tube(mesh, [(6, 0, 98), (1, 0, 102.5), (-6, 0, 101), (-10, 0, 95)],
             [1.8, 2.4, 2.0, 1.0], stone, 10)
        for side in (1, -1):  # horns
            tube(mesh, [(1, side * 7, 96), (-1, side * 11, 101), (-5, side * 13, 106)],
                 [1.8, 1.3, 0.4], stone, 8)


def sword(mesh, x, lean=0.09):
    """A long sword held point down before the figure, its point leaning out past the
    hem: a spiked pommel under the hands, a wrapped grip, a crossguard curving down
    to its finials round red gems, and a ridged blade tapering to the ground."""
    top = 64.4
    at = lambda z, y=0.0: (x + (top - z) * lean, y, z)  # noqa: E731
    d = norm(sub(at(0), at(top)))
    fwd = norm(cross((0, 1, 0), d))
    axes = (fwd, (0, 1, 0), scale(d, -1))
    blob(mesh, at(top), axes, (2.2, 2.2, 1.8), "hilt", 8, 16)
    tube(mesh, [at(top + 1.4), at(top + 3.8)], [1.0, 0.1], "hilt", 10)
    for y in (-1, 1):
        tube(mesh, [at(top, y * 1.8), at(top, y * 3.4)], [0.7, 0.1], "hilt", 8)
    rod(mesh, at(54.5), at(63.0), 1.15, "hilt", 12)
    for k in range(7):
        z = 55.0 + 1.15 * k
        rod(mesh, at(z), at(z + 0.45), 1.35, "hilt", 12)
    guard = [at(53.2 - 0.022 * y * y - (1.4 if abs(y) > 9 else 0) * (abs(y) - 9), y)
             for y in (-11, -10, -8, -5, -2, 0, 2, 5, 8, 10, 11)]
    tube(mesh, guard, [0.7, 1.0, 1.2, 1.3, 1.5, 1.8, 1.5, 1.3, 1.2, 1.0, 0.7], "hilt", 10)
    for end in (guard[0], guard[-1]):
        blob(mesh, end, axes, (1.2, 1.2, 1.5), "hilt", 6, 10)
    blob(mesh, at(53.2), axes, (2.0, 2.6, 2.0), "hilt", 6, 12)
    for s in (-1, 1):
        blob(mesh, add(at(53.2), scale(fwd, s * 1.9)), axes, (0.6, 1.0, 1.0), "eyes", 4, 8)
    tube(mesh, [at(51.5), at(48), at(30), at(14), at(6), at(3)],
         [1.9, 1.85, 1.65, 1.35, 0.8, 0.12], "hilt", 4, squash=(2.3, 0.55))


def figure(head="hood", stone="statue", cape=0.17, pauldrons=False, arms=(), sabers=(),
           face="face", long_sword=False, eyes=True):
    """A robed figure 100 units tall facing +x; `arms` are (side, elbow, wrist, hand
    arguments), `sabers` (base, direction, blade length); `face` is the hood's inside
    (dark, or the stone for a face that shows); `long_sword` adds a sword held point
    down."""
    hood = head == "hood"
    mesh = Mesh()
    hs = heights(hood)
    rows, uvs = [], []
    for h in hs:
        row = []
        for j in range(SEGMENTS + 1):  # the seam's column twice, for its texture
            th = 2 * math.pi * j / SEGMENTS
            row.append(surface_point(h, th, cape) if hood else
                       (body_radius(h, th, cape, False) * math.cos(th),
                        body_radius(h, th, cape, False) * math.sin(th), h))
        rows.append(row)
        uvs.append([(4 * j / SEGMENTS, h / 25) for j in range(SEGMENTS + 1)])

    def material(i, j):
        h = (hs[i] + hs[i + 1]) / 2
        th = 2 * math.pi * (j + 0.5) / SEGMENTS
        th = math.atan2(math.sin(th), math.cos(th))
        oval = (th / FACE_HALF_ANGLE) ** 2 + ((h - FACE_CENTER) / FACE_HALF_HEIGHT) ** 2
        return face if hood and oval < 1 else stone

    mesh.grid(rows, uvs, material, wrap=False)
    if hood:
        hood_rim(mesh, stone, cape)
        face_features(mesh, stone, eyes)
    else:
        mesh.cap(rows[-1][:-1], (0, 0, hs[-1]), stone, True)
        helmet(mesh, stone, head == "crest")
    belt(mesh, stone, cape)
    clasp(mesh, stone, cape)
    boots(mesh, stone)
    if pauldrons:
        for side in (1, -1):
            for k, (z, r, t) in enumerate(((76.5, 6.6, 2.6), (74.3, 6.0, 1.7), (72.6, 5.4, 1.4))):
                y = side * (16.0 + 1.3 * k)
                tilt = norm((0, side * 0.45, 1))  # each plate lies on the sloping shoulder
                blob(mesh, (0.5, y, z), ((1, 0, 0), norm(cross(tilt, (1, 0, 0))), tilt),
                     (r + 0.6, r, t), "hilt", 8, 18)
            blob(mesh, (0.5, side * 16.3, 79.3), ((1, 0, 0), (0, 1, 0), (0, 0, 1)), (1.0, 1.0, 1.1),
                 "eyes", 4, 8)
    for side, elbow, wrist, *rest in arms:
        arm(mesh, side, elbow, wrist, stone, cape, rest[0] if rest else None)
    for base, direction, blade in sabers:
        saber(mesh, base, direction, blade)
    if long_sword:
        sword(mesh, 21.5)
    return mesh.surfaces()


FIST = dict(curl=1.25, spread=0.0)
# The figures the maps place: name -> figure's arguments. An arm is (side, elbow,
# wrist, hand): the hand's back faces `back`, its thumb on the `thumb` side.
STATUES = {
    # hooded, both hands on a hilt held point down
    "acolyte": dict(arms=[(1, (10, 17, 61), (19.2, 3.6, 54.5), dict(back=(0.4, 1, 0.1), thumb=-1, **FIST)),
                          (-1, (10, -17, 58), (19.2, -3.6, 50.0), dict(back=(0.4, -1, 0.1), thumb=1, **FIST))],
                    sabers=[((23, 0, 47), (0, 0, 1), 0)]),
    # hooded lord, arms crossed, a heavy cape and armoured shoulders
    "lord": dict(stone="statue_bronze", pauldrons=True, cape=0.26,
                 arms=[(-1, (12, -15, 63), (15, 8, 66.5), dict(back=(1, 0, 0.3), curl=0.35, thumb=-1)),
                       (1, (12, 15, 61), (16, -7, 63.5), dict(back=(1, 0, 0.3), curl=0.35, thumb=1))]),
    # hooded, one hand thrust out with the Force, the other on a hilt at the hip
    "caster": dict(stone="statue_dark",
                   arms=[(1, (12, 20, 70), (26, 15, 73), dict(back=(-0.2, 0, 1), curl=0.12, spread=0.22,
                                                            thumb=1)),
                         (-1, (4, -20, 59), (9.5, -17.5, 50), dict(back=(0, -1, 0), thumb=1, **FIST))],
                   sabers=[((11, -19.5, 49), (0, 0, -1), 0)]),
    # the guardian of the last hall: a wide cape, both hands resting on the pommel
    # of a long sword held point down
    "guardian": dict(cape=0.32, long_sword=True,
                     arms=[(1, (11, 16, 65), (19.0, 3.4, 61.2), dict(back=(0.4, 1, 0.1), thumb=-1, **FIST)),
                           (-1, (11, -16, 61), (19.2, -3.4, 57.4), dict(back=(0.4, -1, 0.1), thumb=1,
                                                                        **FIST))]),
}


def statue_mesh(name="acolyte"):
    """Surfaces of a statue: material -> (vertices, uvs, triangles)."""
    return figure(**STATUES[name])


# ---------------------------------------------------------------- MD3

def normals(verts, tris):
    """Smooth normals, shared across vertices at the same position."""
    acc = {}
    for a, b, c in tris:
        n = cross(sub(verts[b], verts[a]), sub(verts[c], verts[a]))
        for k in (a, b, c):
            key = tuple(round(x, 3) for x in verts[k])
            s = acc.setdefault(key, [0.0, 0.0, 0.0])
            for j in range(3):
                s[j] += n[j]
    return [norm(acc[tuple(round(x, 3) for x in v)]) for v in verts]


def compact(verts, uvs, tris):
    """Merge vertices equal in position and texture coordinates."""
    index, out_v, out_uv, out_t = {}, [], [], []
    for tri in tris:
        new = []
        for k in tri:
            key = (tuple(round(x, 4) for x in verts[k]), tuple(round(x, 5) for x in uvs[k]))
            if key not in index:
                index[key] = len(out_v)
                out_v.append(verts[k])
                out_uv.append(uvs[k])
            new.append(index[key])
        out_t.append(tuple(new))
    return out_v, out_uv, out_t


def chunks(verts, uvs, tris, max_verts=1000, max_tris=2000):
    """Split a surface into pieces within the MD3 renderer's limits."""
    out, start = [], 0
    while start < len(tris):
        end = min(len(tris), start + max_tris)
        while True:
            v, uv, t = compact(verts, uvs, tris[start:end])
            if len(v) <= max_verts:
                break
            end = start + (end - start) * 3 // 4
        out.append((v, uv, t))
        start = end
    return out


def write_md3(path, surfaces, shaders):
    """One frame; `surfaces` maps a name to (vertices, uvs, counter-clockwise
    triangles) and `shaders` each name to its shader."""
    def name(s, n):
        return s.encode()[:n - 1].ljust(n, b"\0")

    blobs = []
    lo, hi = [1e9] * 3, [-1e9] * 3
    for key, (verts, uvs, tris) in surfaces.items():
        def kept(tri):  # not collapsed to a line or a point on MD3's 1/64-unit grid
            q = [tuple(round(x * 64) for x in verts[k]) for k in tri]
            return cross(sub(q[1], q[0]), sub(q[2], q[0])) != (0, 0, 0)

        tris = [tri for tri in tris if kept(tri)]
        for piece, (pv, puv, pt) in enumerate(chunks(verts, uvs, tris)):
            norms = normals(pv, pt)
            for v in pv:
                for k in range(3):
                    lo[k], hi[k] = min(lo[k], v[k]), max(hi[k], v[k])
            tri_data = b"".join(struct.pack("<3i", a, c, b) for a, b, c in pt)  # to clockwise
            st = b"".join(struct.pack("<2f", u, v) for u, v in puv)
            xyzn = b""
            for (x, y, z), (nx, ny, nz) in zip(pv, norms):
                # high byte: azimuth, low byte: angle from +z (R_LoadMD3's decoding)
                azimuth = int(round(math.atan2(ny, nx) * 128 / math.pi)) & 255
                polar = int(round(math.acos(max(-1.0, min(1.0, nz))) * 128 / math.pi)) & 255
                xyzn += struct.pack("<3hH", round(x * 64), round(y * 64), round(z * 64),
                                    azimuth << 8 | polar)
            shader = name(shaders[key], 64) + struct.pack("<i", 0)
            hdr = 108
            ofs_tri = hdr + len(shader)
            ofs_st = ofs_tri + len(tri_data)
            ofs_xyz = ofs_st + len(st)
            end = ofs_xyz + len(xyzn)
            head = b"IDP3" + name("%s%d" % (key, piece), 64) + struct.pack(
                "<10i", 0, 1, 1, len(pv), len(pt), ofs_tri, hdr, ofs_st, ofs_xyz, end)
            blobs.append(head + shader + tri_data + st + xyzn)
    radius = max(math.sqrt(sum(v * v for v in c)) for c in (lo, hi))
    frame = struct.pack("<3f3f3ff", *lo, *hi, 0, 0, 0, radius) + name("frame0", 16)
    ofs_frames = 108
    ofs_surf = ofs_frames + len(frame)
    eof = ofs_surf + sum(len(b) for b in blobs)
    head = b"IDP3" + struct.pack("<i", 15) + name(path.stem, 64) + struct.pack(
        "<9i", 0, 1, 0, len(blobs), 0, ofs_frames, ofs_surf, ofs_surf, eof)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(head + frame + b"".join(blobs))
