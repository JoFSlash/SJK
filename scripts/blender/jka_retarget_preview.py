# Preview: drive another skeleton from JKA animations in Blender (proof of concept, 14/10/2026).
# Input JSON comes from `cargo run -p sjk-model --example gla_dump`. JSON and ARM below
# are the test paths; the MAP uses Unreal-mannequin bone names.
# Drive a skeleton from JKA _humanoid animations (dumped by sjk-model's gla_dump).
# Each mapped target bone takes the JKA bone's world-space rotation change from its
# base pose, after its own rest is aligned onto the JKA rest direction. Unmapped
# bones (fingers, face, twist, cloth) keep their rest relative to their parent.
import bpy, json
from mathutils import Matrix, Vector

JSON = r"jka_anims.json"  # set to the gla_dump output
ARM = None  # an armature object name, or None for the active object

# target bone: (JKA bone, target end bone, JKA end bone); end None = keep rest alignment;
# (JKA bone, ALIGN_AS, other target bone) = reuse that bone's alignment
ALIGN_AS = object()
MAP = {
    "pelvis": ("pelvis", "spine_01", "lower_lumbar"),
    "spine_01": ("lower_lumbar", "spine_03", "upper_lumbar"),
    "spine_02": ("lower_lumbar", "spine_03", "upper_lumbar"),
    "spine_03": ("upper_lumbar", "spine_04", "thoracic"),
    "spine_04": ("thoracic", "neck_01", "cervical"),
    "spine_05": ("thoracic", "neck_01", "cervical"),
    "neck_01": ("cervical", "head", "cranium"),
    "neck_02": ("cervical", "head", "cranium"),
    "head": ("cranium", None, None),
}
for s, j in (("l", "l"), ("r", "r")):
    MAP.update({
        f"upperarm_{s}": (f"{j}humerus", f"lowerarm_{s}", f"{j}radius"),
        f"lowerarm_{s}": (f"{j}radius", f"hand_{s}", f"{j}hand"),
        f"hand_{s}": (f"{j}hand", f"middle_01_{s}", f"{j}_d2_j1"),
        f"thigh_{s}": (f"{j}femurYZ", f"calf_{s}", f"{j}tibia"),
        f"calf_{s}": (f"{j}tibia", f"foot_{s}", f"{j}talus"),
        f"foot_{s}": (f"{j}talus", None, None),
    })
    # JKA has two joints for three finger groups: thumb (d1), index+middle (d2),
    # ring+pinky (d4). The first joint aligns the finger; the rest follow its alignment.
    for finger, d in (("thumb", "d1"), ("index", "d2"), ("middle", "d2"), ("ring", "d4"), ("pinky", "d4")):
        MAP[f"{finger}_01_{s}"] = (f"{j}_{d}_j1", f"{finger}_02_{s}", f"{j}_{d}_j2")
        MAP[f"{finger}_02_{s}"] = (f"{j}_{d}_j2", ALIGN_AS, f"{finger}_01_{s}")
        MAP[f"{finger}_03_{s}"] = (f"{j}_{d}_j2", ALIGN_AS, f"{finger}_01_{s}")

def m34(v):
    return Matrix(((v[0], v[1], v[2], v[3]), (v[4], v[5], v[6], v[7]),
                   (v[8], v[9], v[10], v[11]), (0, 0, 0, 1)))

def frame(direction, ref=Vector((0, -1, 0))):
    x = direction.normalized()
    z = x.cross(ref).normalized()
    y = z.cross(x)
    return Matrix((x, y, z)).transposed()

def stand_pelvis(data, jidx, jbase):
    j = jidx["pelvis"]
    stand = next(a for a in data["animations"] if a["name"] == "BOTH_STAND1")
    return (m34(stand["frames"][0][j]) @ jbase[j]).translation

def run(names=None, fps=30):
    data = json.load(open(JSON))
    jidx = {b["name"]: i for i, b in enumerate(data["bones"])}
    jbase = [m34(b["base"]) for b in data["bones"]]
    arm = bpy.data.objects[ARM] if ARM else bpy.context.view_layer.objects.active
    bones = arm.data.bones
    rest = {b.name: b.matrix_local.copy() for b in bones}
    head = lambda n: rest[n].translation
    jhead = lambda n: jbase[jidx[n]].translation
    align = {}
    for t, (j, tend, jend) in MAP.items():
        if tend is ALIGN_AS:
            continue
        if tend is None:
            align[t] = Matrix.Identity(3)
        else:
            align[t] = frame(jhead(jend) - jhead(j)) @ frame(head(tend) - head(t)).transposed()
    for t, (j, tend, other) in MAP.items():
        if tend is ALIGN_AS:
            align[t] = align[other]
    missing = [t for t in MAP if t not in rest] + [MAP[t][0] for t in MAP if MAP[t][0] not in jidx]
    if missing:
        raise KeyError(f"unknown bones: {missing}")
    scale = head("pelvis").z / jhead("pelvis").z
    order = []
    def visit(b):
        order.append(b)
        for c in b.children:
            visit(c)
    for b in bones:
        if b.parent is None:
            visit(b)
    pb = arm.pose.bones
    for p in pb:
        p.rotation_mode = "QUATERNION"
    if arm.animation_data is None:
        arm.animation_data_create()
    made = []
    for anim in data["animations"]:
        if names and anim["name"] not in names:
            continue
        action = bpy.data.actions.new("JKA_" + anim["name"])
        action.use_fake_user = True
        arm.animation_data.action = action
        step = fps / anim["fps"]
        # JKA poses sit relative to the entity origin (mid-body), ~27 units below the
        # base pose: measure the pelvis against the standing idle to keep feet on the floor.
        jp0 = stand_pelvis(data, jidx, jbase)
        for f, mats in enumerate(anim["frames"]):
            S = [m34(m) for m in mats]
            pose = {}
            for b in order:
                if b.parent:
                    base = pose[b.parent.name] @ rest[b.parent.name].inverted() @ rest[b.name]
                else:
                    base = rest[b.name].copy()
                if b.name in MAP:
                    j = jidx[MAP[b.name][0]]
                    rot = S[j].to_3x3().normalized() @ align[b.name] @ rest[b.name].to_3x3()
                    m = rot.to_4x4()
                    if b.name == "pelvis":
                        jp = (S[j] @ jbase[j]).translation
                        m.translation = head("pelvis") + (jp - jp0) * scale
                    else:
                        m.translation = base.translation
                    pose[b.name] = m
                else:
                    pose[b.name] = base
                basis = base.inverted() @ pose[b.name]
                loc, q, _ = basis.decompose()
                p = pb[b.name]
                p.rotation_quaternion = q
                p.location = loc
                t = 1 + f * step
                p.keyframe_insert("rotation_quaternion", frame=t)
                if b.name == "pelvis":
                    p.keyframe_insert("location", frame=t)
        made.append((action.name, len(anim["frames"]), round(1 + (len(anim["frames"]) - 1) * step)))
    return made
