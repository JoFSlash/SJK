"""Generated MD3 models for the Sith maps: the hooded statue, and the MD3 writer.

The statue faces +x and is 100 units tall: a robed figure with a cape, a deep
hood whose face is in shadow, sleeved arms meeting in front, and both hands on
the hilt of a saber held point down. Surfaces (materials):
`statue` (stone), `face` (the hood's shadow and the sleeves' openings) and
`hilt` (metal). Triangles are built counter-clockwise seen from outside and
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


def tube(mesh, path, radii, material, segments=12, cap_end=None):
    """A tube along `path` with a radius per point; `cap_end` closes its far end."""
    rows, uvs = [], []
    for i, (p, r) in enumerate(zip(path, radii)):
        ahead = path[min(i + 1, len(path) - 1)]
        behind = path[max(i - 1, 0)]
        u, v = frame_for(sub(ahead, behind))
        ring = []
        for j in range(segments):
            a = 2 * math.pi * j / segments
            ring.append(add(p, add(scale(u, r * math.cos(a)), scale(v, r * math.sin(a)))))
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


# ---------------------------------------------------------------- the statue

PROFILE = [(0, 24), (6, 23), (20, 21), (36, 18.5), (46, 16), (50, 16.2), (62, 17.5),
           (70, 19.5), (74, 20.5), (77, 20), (79, 18.5), (81, 16), (83, 13.6), (86, 12.8),
           (90, 12.4), (94, 11), (97, 8), (99, 4.5), (100, 0.8)]
HEIGHTS = [0, 2, 5, 9, 14, 20, 26, 32, 38, 43, 46, 49, 53, 58, 63, 68, 72, 75, 77, 79, 80.5,
           82, 83.5, 85, 87, 89, 91, 93, 95, 96.5, 98, 99, 100]
SEGMENTS = 40
FACE_HALF_ANGLE = math.radians(55)  # the hood's opening, either side of +x
FACE_CENTER, FACE_HALF_HEIGHT = 88.0, 5.2


def body_radius(h, theta):
    r = lerp(PROFILE, h)
    c, s = math.cos(theta), math.sin(theta)
    k = 1.0
    if 62 <= h <= 81:  # shoulders, wider sideways
        k *= 1 + 0.30 * math.sin(math.pi * (h - 62) / 19) * s * s
    if 56 <= h <= 76:  # chest
        k *= 1 + 0.05 * max(0.0, c) * math.sin(math.pi * (h - 56) / 20)
    if h < 76:  # the cape flares behind, down to the ground
        k *= 1 + 0.17 * max(0.0, -c) * (1 - h / 76) ** 1.4
    if h < 46:  # folds in the skirt, deeper near the hem
        k *= 1 + 0.045 * (1 - h / 46) * math.sin(9 * theta + 0.8 * math.sin(2 * theta))
    if 92 <= h <= 99.5:  # the hood's brim reaches forward over the face
        k *= 1 + 0.30 * max(0.0, c) ** 2 * math.sin(math.pi * (h - 92) / 7.5)
    if 83 <= h <= 92:  # and the face sits back inside it
        k *= 1 - 0.30 * max(0.0, c) ** 3 * math.sin(math.pi * (h - 83) / 9)
    return r * k


def statue_mesh():
    """Surfaces of the statue: name -> (vertices, uvs, triangles)."""
    mesh = Mesh()
    rows, uvs = [], []
    for h in HEIGHTS:
        lean = -3.0 * max(0.0, (h - 84) / 16)  # the hood's peak falls back
        row = []
        for j in range(SEGMENTS + 1):  # the seam's column twice, for its texture
            th = 2 * math.pi * j / SEGMENTS
            r = body_radius(h, th)
            row.append((lean + r * math.cos(th), r * math.sin(th), h))
        rows.append(row)
        uvs.append([(4 * j / SEGMENTS, h / 25) for j in range(SEGMENTS + 1)])

    def material(i, j):
        h = (HEIGHTS[i] + HEIGHTS[i + 1]) / 2
        th = 2 * math.pi * (j + 0.5) / SEGMENTS
        th = math.atan2(math.sin(th), math.cos(th))
        oval = (th / FACE_HALF_ANGLE) ** 2 + ((h - FACE_CENTER) / FACE_HALF_HEIGHT) ** 2
        return "face" if oval < 1 else "statue"

    mesh.grid(rows, uvs, material, wrap=False)
    # the belt
    belt = []
    for h in (45.0, 48.5):
        belt.append([(1.05 * body_radius(h, 2 * math.pi * j / SEGMENTS) * math.cos(2 * math.pi * j / SEGMENTS),
                      1.05 * body_radius(h, 2 * math.pi * j / SEGMENTS) * math.sin(2 * math.pi * j / SEGMENTS),
                      h) for j in range(SEGMENTS)])
    mesh.grid(belt, [[(j / 8, 0) for j in range(SEGMENTS)], [(j / 8, 1) for j in range(SEGMENTS)]],
              lambda i, j: "hilt")
    # sleeves from inside the shoulders, bending at the elbow, widening to the
    # wrist, where the opening is dark; the hands meet on the hilt
    for side in (1, -1):
        path = [(1, side * 16, 74), (4, side * 17.5, 66), (8, side * 17, 59),
                (13, side * 11, 54.5), (16.5, side * 5, 52)]
        tube(mesh, path, [5.0, 5.6, 6.0, 6.8, 7.6], "statue", 14, cap_end="face")
        ellipsoid(mesh, (19.5, side * 2.4, 50.5), (3.3, 2.7, 3.8), "statue", 6, 12)
    # the saber: emitter below, grip ridges, pommel under the hands
    x = 22.0
    cylinder(mesh, x, 0, 26, 50, 1.7, "hilt", 14)
    cylinder(mesh, x, 0, 24, 28, 2.5, "hilt", 14)
    for z in (33, 37, 41):
        cylinder(mesh, x, 0, z, z + 1.2, 2.05, "hilt", 14)
    ellipsoid(mesh, (x, 0, 50.6), (2.2, 2.2, 1.4), "hilt", 4, 12)
    return mesh.surfaces()


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
