import bpy, math, sys
from mathutils import Vector

OUT = sys.argv[sys.argv.index("--") + 1]
FRAMES = 120
scene = bpy.context.scene
for o in list(bpy.data.objects):
    bpy.data.objects.remove(o, do_unlink=True)

# Engine: EEVEE (pick valid id).
for eng in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE"):
    try:
        scene.render.engine = eng
        break
    except TypeError:
        pass
print("ENGINE", scene.render.engine)
scene.render.resolution_x = 960
scene.render.resolution_y = 960
scene.render.fps = 30
scene.frame_start = 1
scene.frame_end = FRAMES
scene.render.film_transparent = False
try:
    scene.eevee.use_bloom = True
except Exception:
    pass
try:
    scene.eevee.taa_render_samples = 32
except Exception:
    pass
scene.view_settings.view_transform = "AgX" if "AgX" in [i.identifier for i in bpy.types.ColorManagedViewSettings.bl_rna.properties["view_transform"].enum_items] else "Filmic"
scene.view_settings.look = "None"

world = bpy.data.worlds.new("W")
scene.world = world
world.use_nodes = True
bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
bg.inputs[0].default_value = (0.0055, 0.0035, 0.014, 1)
bg.inputs[1].default_value = 1.0

def principled(name, base, rough=0.2, trans=0.0, emis=None, estr=0.0, alpha=1.0, ior=1.45):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    def s(key, val):
        for inp in b.inputs:
            if inp.name == key:
                inp.default_value = val
                return
    s("Base Color", (*base, 1))
    s("Roughness", rough)
    s("Transmission Weight", trans)
    s("Transmission", trans)
    s("IOR", ior)
    s("Alpha", alpha)
    if emis:
        s("Emission Color", (*emis, 1))
        s("Emission", (*emis, 1))
        s("Emission Strength", estr)
    try:
        m.surface_render_method = "BLENDED" if alpha < 1 else "DITHERED"
    except Exception:
        pass
    return m

glass = principled("Glass", (0.55, 0.45, 0.95), rough=0.08, trans=1.0, ior=1.3)
core = principled("Core", (0.2, 0.1, 0.5), rough=0.3, emis=(0.55, 0.36, 0.98), estr=6.0)
edge = principled("Edge", (0.7, 0.6, 1.0), rough=0.2, emis=(0.77, 0.71, 0.99), estr=12.0)
plate = principled("Plate", (0.07, 0.05, 0.14), rough=0.35)

# Drive body: rounded glass slab with glowing core and LED strip.
bpy.ops.mesh.primitive_cube_add(size=1)
body = bpy.context.object
body.scale = (1.5, 2.2, 0.28)
bev = body.modifiers.new("bev", "BEVEL"); bev.width = 0.12; bev.segments = 8
body.data.materials.append(glass)
bpy.ops.object.shade_smooth()

bpy.ops.mesh.primitive_cube_add(size=1)
inner = bpy.context.object
inner.scale = (1.1, 1.7, 0.08)
b2 = inner.modifiers.new("bev", "BEVEL"); b2.width = 0.05; b2.segments = 4
inner.data.materials.append(core)
inner.parent = body
inner.matrix_parent_inverse = body.matrix_world.inverted()

bpy.ops.mesh.primitive_cube_add(size=1, location=(0, -1.12, 0.0))
led = bpy.context.object
led.scale = (0.5, 0.02, 0.04)
led.data.materials.append(edge)
led.parent = body
led.matrix_parent_inverse = body.matrix_world.inverted()

drive = bpy.data.objects.new("DriveRig", None)
scene.collection.objects.link(drive)
body.parent = drive
drive.rotation_euler = (math.radians(62), 0, 0)

# Feathers: curved thin ellipsoids with gradient emission.
def feather_mat(i):
    m = bpy.data.materials.new(f"Feather{i}")
    m.use_nodes = True
    nt = m.node_tree
    b = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
    tc = nt.nodes.new("ShaderNodeTexCoord")
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    ramp = nt.nodes.new("ShaderNodeValToRGB")
    ramp.color_ramp.elements[0].color = (0.26, 0.22, 0.79, 1)
    ramp.color_ramp.elements[1].color = (0.75, 0.62, 1.0, 1)
    nt.links.new(tc.outputs["Generated"], sep.inputs[0])
    nt.links.new(sep.outputs[0], ramp.inputs[0])
    for inp in b.inputs:
        if inp.name in ("Base Color", "Emission Color", "Emission"):
            nt.links.new(ramp.outputs[0], inp)
        if inp.name == "Emission Strength":
            inp.default_value = 3.5
        if inp.name == "Roughness":
            inp.default_value = 0.25
        if inp.name in ("Transmission Weight", "Transmission"):
            inp.default_value = 0.4
    return m

pivot = bpy.data.objects.new("FeatherPivot", None)
scene.collection.objects.link(pivot)
N = 7
for i in range(N):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=48, ring_count=24, radius=1)
    f = bpy.context.object
    f.scale = (1.35, 0.32, 0.025)
    bpy.ops.object.transform_apply(scale=True)
    d = f.modifiers.new("bend", "SIMPLE_DEFORM"); d.deform_method = "BEND"; d.angle = math.radians(55); d.deform_axis = "Z"
    f.data.materials.append(feather_mat(i))
    bpy.ops.object.shade_smooth()
    a = 2 * math.pi * i / N
    r = 3.1
    f.location = (r * math.cos(a), r * math.sin(a), 0.35 * math.sin(3 * a))
    f.rotation_euler = (math.radians(15), math.radians(-10), a + math.radians(95))
    f.parent = pivot

# Animate: feathers orbit one full turn, drive bobs and turns gently (seamless loop).
pivot.rotation_euler = (math.radians(68), 0, 0)
pivot.keyframe_insert("rotation_euler", index=2, frame=1)
pivot.rotation_euler[2] = 2 * math.pi
pivot.keyframe_insert("rotation_euler", index=2, frame=FRAMES + 1)
for fc in (pivot.animation_data.action.fcurves if hasattr(pivot.animation_data.action, "fcurves") else []):
    for k in fc.keyframe_points:
        k.interpolation = "LINEAR"
try:
    for layer in pivot.animation_data.action.layers:
        for strip in layer.strips:
            for cb in strip.channelbags:
                for fc in cb.fcurves:
                    for k in fc.keyframe_points:
                        k.interpolation = "LINEAR"
except Exception as e:
    print("slotted action", e)

for fr in range(1, FRAMES + 2, 10):
    t = (fr - 1) / FRAMES
    drive.location = (0, 0, 0.18 * math.sin(2 * math.pi * t))
    drive.rotation_euler = (math.radians(62), math.radians(6 * math.sin(2 * math.pi * t)), math.radians(-20 + 12 * math.sin(2 * math.pi * t)))
    drive.keyframe_insert("location", frame=fr)
    drive.keyframe_insert("rotation_euler", frame=fr)

# Lights.
def light(name, kind, loc, energy, color, size=2.0):
    ld = bpy.data.lights.new(name, kind); ld.energy = energy; ld.color = color
    if kind == "AREA": ld.size = size
    o = bpy.data.objects.new(name, ld); o.location = loc
    scene.collection.objects.link(o)
    o.rotation_euler = (Vector((0, 0, 0)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    return o
light("Key", "AREA", (4, -5, 6), 900, (0.75, 0.65, 1.0), 4)
light("Rim", "AREA", (-5, 4, 3), 700, (0.45, 0.3, 1.0), 3)
light("Under", "POINT", (0, 0, -2.5), 300, (0.6, 0.35, 1.0))

cam_d = bpy.data.cameras.new("Cam"); cam_d.lens = 50
cam = bpy.data.objects.new("Cam", cam_d); scene.collection.objects.link(cam)
cam.location = (0, -11.5, 4.2)
cam.rotation_euler = (Vector((0, 0, 0.1)) - cam.location).to_track_quat("-Z", "Y").to_euler()
scene.camera = cam

scene.render.image_settings.file_format = "PNG"
scene.render.filepath = OUT + "/f_"
bpy.ops.render.render(animation=True)
