import bpy, bmesh, math, sys
from mathutils import Vector

args = sys.argv[sys.argv.index("--") + 1:]
OUT, MODE = args[0], args[1]
FRAMES = 96
scene = bpy.context.scene
for o in list(bpy.data.objects):
    bpy.data.objects.remove(o, do_unlink=True)
for eng in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE"):
    try:
        scene.render.engine = eng; break
    except TypeError:
        pass
scene.render.resolution_x = 1280
scene.render.resolution_y = 860
scene.render.fps = 30
scene.frame_start, scene.frame_end = 1, FRAMES
for attr, val in (("taa_render_samples", 48), ("use_raytracing", True), ("use_shadows", True)):
    try: setattr(scene.eevee, attr, val)
    except Exception: pass
vt = [i.identifier for i in bpy.types.ColorManagedViewSettings.bl_rna.properties["view_transform"].enum_items]
scene.view_settings.view_transform = "AgX" if "AgX" in vt else "Filmic"
looks = [i.identifier for i in bpy.types.ColorManagedViewSettings.bl_rna.properties["look"].enum_items]
for lk in ("AgX - Punchy", "Punchy"):
    if lk in looks: scene.view_settings.look = lk; break

world = bpy.data.worlds.new("W"); scene.world = world; world.use_nodes = True
bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
bg.inputs[0].default_value = (0.006, 0.004, 0.016, 1); bg.inputs[1].default_value = 1.0

def mat(name, base, rough=0.3, metal=0.0, emis=None, estr=0.0, coat=0.0, alpha=1.0):
    m = bpy.data.materials.new(name); m.use_nodes = True
    b = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    vals = {"Base Color": (*base, 1), "Roughness": rough, "Metallic": metal, "Coat Weight": coat, "Alpha": alpha}
    if emis: vals.update({"Emission Color": (*emis, 1), "Emission Strength": estr})
    for inp in b.inputs:
        if inp.name in vals: inp.default_value = vals[inp.name]
    if alpha < 1:
        try: m.surface_render_method = "BLENDED"
        except Exception: pass
    return m, b

shell, _ = mat("Shell", (0.06, 0.035, 0.14), rough=0.18, metal=0.6, coat=1.0)
glow, _ = mat("Glow", (0.5, 0.35, 1.0), emis=(0.55, 0.38, 1.0), estr=18)
port, _ = mat("Port", (0.02, 0.02, 0.03), rough=0.5, metal=0.8)
floor_m, _ = mat("Floor", (0.012, 0.008, 0.03), rough=0.12, metal=0.0)

def box(name, scale, loc, m, bevel, parent=None):
    bpy.ops.mesh.primitive_cube_add(size=1, location=loc)
    o = bpy.context.object; o.name = name; o.scale = scale
    bpy.ops.object.transform_apply(scale=True)
    md = o.modifiers.new("bev", "BEVEL"); md.width = bevel; md.segments = 8; md.limit_method = "NONE"
    o.data.materials.append(m); bpy.ops.object.shade_smooth()
    if parent: o.parent = parent
    return o

rig = bpy.data.objects.new("Rig", None); scene.collection.objects.link(rig)
body = box("Body", (2.0, 1.25, 0.36), (0, 0, 0), shell, 0.16, rig)
box("Stripe", (1.3, 0.05, 0.02), (0, -0.25, 0.185), glow, 0.02, rig)
box("Stripe2", (0.7, 0.05, 0.02), (-0.3, 0.05, 0.185), glow, 0.02, rig)
box("LED", (0.08, 0.02, 0.06), (0.75, -0.63, 0.0), glow, 0.01, rig)
box("Port", (0.34, 0.04, 0.1), (0, -0.63, 0.0), port, 0.03, rig)

def feather_mesh(length, width):
    bm = bmesh.new()
    segs, cols = 40, 9
    rows = []
    for i in range(segs + 1):
        t = i / segs
        x = t * length
        w = width * (math.sin(math.pi * min(1, t * 1.15)) ** 0.7) * (1 - 0.35 * t)
        if t > 0.87: w *= max(0.0, (1 - t) / 0.13)
        row = []
        for j in range(cols):
            s = j / (cols - 1) * 2 - 1
            y = s * w
            z = 0.06 * (1 - s * s) * math.sin(math.pi * t) + 0.18 * t * t
            row.append(bm.verts.new((x, y, z)))
        rows.append(row)
    for i in range(segs):
        for j in range(cols - 1):
            bm.faces.new((rows[i][j], rows[i + 1][j], rows[i + 1][j + 1], rows[i][j + 1]))
    me = bpy.data.meshes.new("Feather"); bm.to_mesh(me); bm.free()
    for p in me.polygons: p.use_smooth = True
    return me

def feather_mat():
    m = bpy.data.materials.new("FeatherM"); m.use_nodes = True
    nt = m.node_tree; b = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    tc = nt.nodes.new("ShaderNodeTexCoord"); sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    ramp = nt.nodes.new("ShaderNodeValToRGB")
    e = ramp.color_ramp.elements
    e[0].position = 0.0; e[0].color = (0.18, 0.12, 0.62, 1)
    e[1].position = 1.0; e[1].color = (0.7, 0.56, 1.0, 1)
    mid = e.new(0.55); mid.color = (0.4, 0.22, 0.92, 1)
    nt.links.new(tc.outputs["Generated"], sep.inputs[0]); nt.links.new(sep.outputs[0], ramp.inputs[0])
    em = nt.nodes.new("ShaderNodeMath"); em.operation = "MULTIPLY"; em.inputs[1].default_value = 1.1
    nt.links.new(sep.outputs[0], em.inputs[0])
    for inp in b.inputs:
        if inp.name in ("Base Color", "Emission Color"): nt.links.new(ramp.outputs[0], inp)
        if inp.name == "Emission Strength": nt.links.new(em.outputs[0], inp)
        if inp.name == "Roughness": inp.default_value = 0.35
        if inp.name == "Alpha": inp.default_value = 0.92
        if inp.name == "Sheen Weight": inp.default_value = 0.6
    m.use_backface_culling = False
    try: m.surface_render_method = "BLENDED"
    except Exception: pass
    return m

fm = feather_mat()
N = 15
wings = []
for side in (1, -1):
    root = bpy.data.objects.new(f"Wing{side}", None); scene.collection.objects.link(root)
    root.parent = rig
    root.location = (side * 0.95, 0.15, 0.12)
    root.scale = (side, 1, 1)
    for i in range(N):
        t = i / (N - 1)
        length = 1.6 + 1.9 * math.sin(math.pi * (0.2 + 0.62 * t))
        width = 0.36 + 0.1 * t
        f = bpy.data.objects.new(f"F{side}_{i}", feather_mesh(length, width))
        f.data.materials.append(fm)
        scene.collection.objects.link(f); f.parent = root
        ang = math.radians(4 + 58 * t)
        f.rotation_euler = (math.radians(-10 + 6 * t), -ang, math.radians(-26 + 30 * t))
        f.location = (0.05 * t, 0.12 - 0.3 * t, 0.02 * i)
    wings.append((root, side))

for fr in range(1, FRAMES + 2, 4):
    ph = 2 * math.pi * (fr - 1) / FRAMES
    for root, side in wings:
        root.rotation_euler = (math.radians(-4 * math.cos(ph)), math.radians(-14 * math.sin(ph)), math.radians(-6 + 4 * math.sin(ph)))
        root.keyframe_insert("rotation_euler", frame=fr)
    rig.location = (0, 0, 0.6 + 0.16 * math.sin(ph + 0.6))
    rig.rotation_euler = (math.radians(8 + 2 * math.sin(ph)), 0, math.radians(-18 + 5 * math.sin(ph * 1)))
    rig.keyframe_insert("location", frame=fr); rig.keyframe_insert("rotation_euler", frame=fr)


def light(name, kind, loc, energy, color, size=2.0):
    ld = bpy.data.lights.new(name, kind); ld.energy = energy; ld.color = color
    if kind == "AREA": ld.size = size
    o = bpy.data.objects.new(name, ld); o.location = loc; scene.collection.objects.link(o)
    o.rotation_euler = (Vector((0, 0, 0.5)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
light("Key", "AREA", (3.5, -6, 6.5), 1400, (0.85, 0.78, 1.0), 5)
light("Rim", "AREA", (-4, 6, 4), 1600, (0.5, 0.32, 1.0), 4)
light("Rim2", "AREA", (5, 5, 2), 900, (0.35, 0.3, 1.0), 3)
light("Under", "POINT", (0, -1.5, -1.0), 250, (0.6, 0.35, 1.0))

cam_d = bpy.data.cameras.new("Cam"); cam_d.lens = 48
cam = bpy.data.objects.new("Cam", cam_d); scene.collection.objects.link(cam)
cam.location = (0.5, -11.0, 3.6)
cam.rotation_euler = (Vector((0, 0, 1.25)) - cam.location).to_track_quat("-Z", "Y").to_euler()
scene.camera = cam

scene.render.image_settings.file_format = "PNG"
if MODE == "still":
    scene.frame_set(1); scene.render.filepath = OUT + "/still_a.png"; bpy.ops.render.render(write_still=True)
    scene.frame_set(FRAMES // 4); scene.render.filepath = OUT + "/still_b.png"; bpy.ops.render.render(write_still=True)
else:
    scene.render.filepath = OUT + "/f_"; bpy.ops.render.render(animation=True)
