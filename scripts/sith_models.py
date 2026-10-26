"""Generated MD3 models for the Sith maps: the hooded statue, and the MD3 writer.

The statues face +x and are 100 units tall: robed figures with a cape, boots,
a belt and buckle, under a deep hood whose face is in shadow or a helm with
glowing eye slits, in several poses ([`STATUES`]). Surfaces (materials): the
stone (`statue`, `statue_dark`, `statue_bronze`), `face` (the hood's shadow,
the face plate, the sleeves' openings), `hilt` (metal), `eyes`, and a lit
saber's `blade` and `blade_core`. Triangles are built counter-clockwise seen from outside and
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


# ---------------------------------------------------------------- the statues

PROFILE = [(0, 24), (6, 23), (20, 21), (36, 18.5), (46, 16), (50, 16.2), (62, 17.5),
           (70, 19.5), (74, 20.5), (77, 20), (79, 18.5), (81, 16), (83, 13.6), (86, 12.8),
           (90, 12.4), (94, 11), (97, 8), (99, 4.5), (100, 0.8)]
HEIGHTS = [0, 2, 5, 9, 14, 20, 26, 32, 38, 43, 46, 49, 53, 58, 63, 68, 72, 75, 77, 79, 80.5,
           82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96.5, 98, 99, 100]
# A helmeted figure's body stops at the neck; the head is built on it.
NECK = [(81, 15.5), (82.5, 8.5), (83.5, 7.5)]
SEGMENTS = 48
FACE_HALF_ANGLE = math.radians(55)  # the hood's opening, either side of +x
FACE_CENTER, FACE_HALF_HEIGHT = 88.0, 5.2
SHOULDERS = {1: (1, 16, 74), -1: (1, -16, 74)}


def body_radius(h, theta, cape, hood=True):
    r = lerp(PROFILE if hood or h <= 81 else PROFILE[:11] + NECK, h)
    c, s = math.cos(theta), math.sin(theta)
    k = 1.0
    if 62 <= h <= 81:  # shoulders, wider sideways
        k *= 1 + 0.30 * math.sin(math.pi * (h - 62) / 19) * s * s
    if 56 <= h <= 76:  # chest
        k *= 1 + 0.05 * max(0.0, c) * math.sin(math.pi * (h - 56) / 20)
    if h < 76:  # the cape flares behind, down to the ground
        k *= 1 + cape * max(0.0, -c) * (1 - h / 76) ** 1.4
    if h < 46:  # folds in the skirt, deeper near the hem
        k *= 1 + 0.045 * (1 - h / 46) * math.sin(9 * theta + 0.8 * math.sin(2 * theta))
    if hood and 92 <= h <= 99.5:  # the hood's brim reaches forward over the face
        k *= 1 + 0.30 * max(0.0, c) ** 2 * math.sin(math.pi * (h - 92) / 7.5)
    if hood and 83 <= h <= 92:  # and the face sits back inside it
        k *= 1 - 0.30 * max(0.0, c) ** 3 * math.sin(math.pi * (h - 83) / 9)
    return r * k


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
    rod(mesh, at(-1), at(10), 1.7, "hilt", 14)
    rod(mesh, at(-1.6), at(-0.6), 2.2, "hilt", 14)  # pommel
    for k in (2.5, 4.5, 6.5):  # grip ridges
        rod(mesh, at(k), at(k + 0.9), 2.05, "hilt", 14)
    rod(mesh, at(9), at(11.5), 2.5, "hilt", 14)  # emitter
    if blade:
        rod(mesh, at(11.5), at(11.5 + blade), 1.25, "blade", 10)
        rod(mesh, at(11.5), at(11.5 + blade + 0.6), 0.55, "blade_core", 8)


def arm(mesh, side, elbow, wrist, stone, sleeve=(5.0, 5.8, 6.4, 7.4)):
    """A sleeve from the shoulder through `elbow` to `wrist`, dark inside, and a hand."""
    shoulder = SHOULDERS[side]
    mid1 = tuple((a * 2 + b) / 3 for a, b in zip(shoulder, elbow))
    mid2 = tuple((a + b * 2) / 3 for a, b in zip(elbow, wrist))
    tube(mesh, [shoulder, mid1, elbow, mid2, wrist], [sleeve[0], sleeve[1], sleeve[2],
                                                      (sleeve[2] + sleeve[3]) / 2, sleeve[3]],
         stone, 14, cap_end="face")
    out = norm(sub(wrist, elbow))
    ellipsoid(mesh, add(wrist, scale(out, 2.8)), (3.2, 3.0, 3.4), stone, 6, 12)


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


def sword(mesh, x):
    """A long sword held point down before the figure: pommel under the hands, a
    crossguard, and a blade reaching almost to the ground."""
    ellipsoid(mesh, (x, 0, 64), (2.0, 2.0, 2.0), "hilt", 6, 12)
    rod(mesh, (x, 0, 53.5), (x, 0, 62.5), 1.3, "hilt", 10)
    rod(mesh, (x, -8.5, 52.5), (x, 8.5, 52.5), 1.3, "hilt", 10)
    tube(mesh, [(x, 0, 52), (x, 0, 12), (x, 0, 4)], [1.6, 1.6, 0.2], "hilt", 4,
         squash=(2.2, 0.55))


def figure(head="hood", stone="statue", cape=0.17, pauldrons=False, arms=(), sabers=(),
           face="face", long_sword=False):
    """A robed figure 100 units tall facing +x; `arms` are (side, elbow, wrist),
    `sabers` (base, direction, blade length); `face` is the hood's inside (dark, or
    the stone for a face that shows); `long_sword` adds a sword held point down."""
    hood = head == "hood"
    mesh = Mesh()
    heights = HEIGHTS if hood else [h for h in HEIGHTS if h <= 81] + [82.5, 83.5]
    rows, uvs = [], []
    for h in heights:
        lean = -3.0 * max(0.0, (h - 84) / 16)  # the hood's peak falls back
        row = []
        for j in range(SEGMENTS + 1):  # the seam's column twice, for its texture
            th = 2 * math.pi * j / SEGMENTS
            r = body_radius(h, th, cape, hood)
            row.append((lean + r * math.cos(th), r * math.sin(th), h))
        rows.append(row)
        uvs.append([(4 * j / SEGMENTS, h / 25) for j in range(SEGMENTS + 1)])

    def material(i, j):
        h = (heights[i] + heights[i + 1]) / 2
        th = 2 * math.pi * (j + 0.5) / SEGMENTS
        th = math.atan2(math.sin(th), math.cos(th))
        oval = (th / FACE_HALF_ANGLE) ** 2 + ((h - FACE_CENTER) / FACE_HALF_HEIGHT) ** 2
        return face if hood and oval < 1 else stone

    mesh.grid(rows, uvs, material, wrap=False)
    if not hood:
        mesh.cap(rows[-1][:-1], (0, 0, heights[-1]), stone, True)
        helmet(mesh, stone, head == "crest")
    # a belt with a buckle, boots under the hem
    belt = [[(1.05 * body_radius(h, 2 * math.pi * j / SEGMENTS, cape) * math.cos(2 * math.pi * j / SEGMENTS),
              1.05 * body_radius(h, 2 * math.pi * j / SEGMENTS, cape) * math.sin(2 * math.pi * j / SEGMENTS),
              h) for j in range(SEGMENTS)] for h in (45.0, 48.5)]
    mesh.grid(belt, [[(j / 8, 0) for j in range(SEGMENTS)], [(j / 8, 1) for j in range(SEGMENTS)]],
              lambda i, j: "hilt")
    ellipsoid(mesh, (17.4, 0, 46.8), (1.4, 3.2, 2.6), "hilt", 4, 10)
    for side in (1, -1):
        ellipsoid(mesh, (21, side * 7.5, 2.2), (6.5, 3.8, 2.6), stone, 6, 12)
    if pauldrons:
        for side in (1, -1):
            ellipsoid(mesh, (0.5, side * 19, 77), (7.5, 7, 4.2), "hilt", 8, 14)
            ellipsoid(mesh, (0.5, side * 20, 74.5), (6.5, 6, 2.5), "hilt", 6, 12)
    for side, elbow, wrist in arms:
        arm(mesh, side, elbow, wrist, stone)
    for base, direction, blade in sabers:
        saber(mesh, base, direction, blade)
    if long_sword:
        sword(mesh, 19.5)
    return mesh.surfaces()


# The figures the maps place: name -> figure's arguments.
STATUES = {
    # hooded, both hands on a hilt held point down
    "acolyte": dict(arms=[(1, (8, 17, 59), (16.5, 5, 52)), (-1, (8, -17, 59), (16.5, -5, 52))],
                    sabers=[((22, 0, 52), (0, 0, -1), 0)]),
    # hooded lord, arms crossed, a heavy cape and armoured shoulders
    "lord": dict(stone="statue_bronze", pauldrons=True, cape=0.26,
                 arms=[(-1, (11, -14, 64), (14, 9, 67)), (1, (11, 14, 62), (15, -8, 64))]),
    # hooded, one hand thrust out with the Force, the other on a hilt at the hip
    "caster": dict(stone="statue_dark",
                   arms=[(1, (12, 19, 70), (25, 14, 73)), (-1, (4, -19, 60), (9, -16, 50))],
                   sabers=[((10, -18, 46), (0, 0, -1), 0)]),
    # the guardian of the last hall: hood back enough to show the face, a wide
    # cape, both hands resting on the pommel of a long sword held point down
    "guardian": dict(face="statue", cape=0.32, long_sword=True,
                     arms=[(1, (9, 15, 63), (16.5, 3.5, 61)), (-1, (9, -15, 63), (16.5, -3.5, 61))]),
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
