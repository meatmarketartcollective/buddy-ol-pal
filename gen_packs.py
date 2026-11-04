# -*- coding: utf-8 -*-
import os

EYES = set("oO0\u25c9\u25cf\u25d5\u2665")
MOUTHS = set("\u15d7\u25bd\u03c93")

def swap(art, mapping):
    return "\n".join("".join(mapping.get(ch, ch) for ch in line) for line in art.splitlines())

WORK = {'o': '>', 'O': '>', '\u15d7': '-'}
BLINK = {c: '-' for c in EYES}
FOCUS_PLUS = {'>': '.'}
FOCUS_PLUS.update({c: '.' for c in EYES})
HAPPY = {c: '^' for c in EYES}
HAPPY2 = dict(HAPPY)
HAPPY2.update({c: '^' for c in MOUTHS})
ANNOY = {c: '>' for c in EYES}
ANNOY2 = dict(ANNOY)
ANNOY2.update({c: '<' for c in MOUTHS})

def sway(a):
    return "\n".join((" " + l) if l else l for l in a.splitlines())

def frames_pair(a, pre=None, m1=None, m2=None, static=False):
    a = a.lstrip("\n")
    if pre is not None:
        base = swap(a, pre)
        f1 = base.rstrip()
        f2 = swap(base, m2).rstrip()
    else:
        f1 = (swap(a, m1) if m1 else a).rstrip()
        f2 = (swap(a, m2) if m2 else f1).rstrip()
    if static:
        return [f1]
    if f1 == f2:
        f2 = sway(f1)
    return [f1] if f1 == f2 else [f1, f2]

def art_list(frames):
    if len(frames) == 1:
        return "'''\n%s\n'''" % frames[0]
    return "[" + ", ".join("'''\n%s\n'''" % fr for fr in frames) + "]"

def toml_str_list(xs):
    return "[" + ", ".join('"' + x.replace('"', '\\"') + '"' for x in xs) + "]"

def emit(f, slug, tier, names, idle, lines):
    st = (slug == "petrock")
    f.write("[[species]]\n")
    f.write('name = "%s"\n' % slug)
    f.write('tier = "%s"\n' % tier)
    f.write("names = %s\n" % toml_str_list(names))
    f.write("[species.art]\n")
    f.write("idle = %s\n" % art_list(frames_pair(idle, m2=BLINK, static=st)))
    f.write("working = %s\n" % art_list(frames_pair(idle, pre=WORK, m2=FOCUS_PLUS, static=st)))
    f.write("celebrating = %s\n" % art_list(frames_pair(idle, m1=HAPPY, m2=HAPPY2, static=st)))
    f.write("miffed = %s\n" % art_list(frames_pair(idle, m1=ANNOY, m2=ANNOY2, static=st)))
    f.write("[species.lines]\n")
    for mood in ("idle", "working", "celebrating", "miffed"):
        f.write("%s = %s\n" % (mood, toml_str_list(lines[mood])))
    f.write("\n")

PACKS = {}
exec(open("/workspace/pack_data.py").read(), PACKS)
DATA = PACKS["DATA"]

HEADERS = {
    "mario": "Mario-universe species pack: 20 characters, art by Casey Benson.",
    "pokemon": "Pokemon species pack: 20 characters, art by Casey Benson.",
    "starfox": "Star Fox universe species pack: 20 characters, art by Casey Benson.",
    "smash": "Super Smash Bros species pack: 20 characters, art by Casey Benson.",
    "nostalgia": "90s Nostalgia species pack: 20 objects, silhouettes per Casey's refinement doc.",
}
TAIL = ("# Every species and every mood carries two art frames that cycle every\n"
        "# tick; featureless designs get a gentle sway so nothing sits frozen.\n"
        "# The engine's 7-state animation layer (blink, zzz drift, hearts,\n"
        "# spinner, glitch) runs on top. Sleeping/resting/petted ride the idle\n"
        "# frames. Sole exception: petrock, who is a rock and does not animate.\n"
        "# That is the joke.")

for key, header in HEADERS.items():
    path = os.path.join(os.path.expanduser("~/hatch/packs"), key + ".toml")
    with open(path, "w") as f:
        f.write("# %s\n# %s\n# Drop in ~/.config/hatch/packs/ (or $HATCH_HOME/packs).\n\n" % (header, TAIL))
        for (slug, tier, names, art, lines) in DATA[key]:
            emit(f, slug, tier, names, art, lines)
    print("wrote", path)
print("total species:", sum(len(v) for v in DATA.values()))
