import re
from PIL import Image, ImageDraw, ImageFont
SRC = "/Users/mike/near-guard-1114/demo/transcript.txt"
lines = open(SRC, errors="replace").read().splitlines()
keep = []
for ln in lines:
    t = ln.rstrip()
    if any(k in t for k in ("Done deploying", "EVENT", "$ npx", "--- take", "Failure", "per_tx", "spent_today", "View call", "day:", "receivers:", "enabled:")):
        if "parseResultError" in t or "at " in t[:6]:
            continue
        keep.append(t)
keep = keep[:26]
W, H = 1400, 96 + 26 * 30
img = Image.new("RGB", (W, H), (13, 17, 23))
d = ImageDraw.Draw(img)
f = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 17)
fb = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 19)
d.text((20, 16), "near-guard / NEAR testnet / real transcript", font=fb, fill=(126, 231, 135))
y = 64
for t in keep:
    c = (255, 255, 255)
    if t.startswith("$ "):
        c = (139, 148, 158)
    elif "EVENT" in t:
        c = (126, 231, 135)
    elif "Failure" in t:
        c = (255, 123, 114)
    elif t.startswith("--- "):
        c = (210, 168, 255)
    d.text((20, y), t[:150], font=f, fill=c)
    y += 30
img.save("/Users/mike/near-guard-1114/demo/proof-card.png")
print("lines", len(keep), "size", img.size)
