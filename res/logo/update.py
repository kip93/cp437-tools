#!/usr/bin/env python3

from itertools import chain
from os import listdir
from pathlib import Path
from subprocess import run

root = Path(__file__).parent.parent.parent
target = root / "target/release"
logos = root / "res/logo"

for path in chain(logos.glob("*.png"), logos.glob("*.svg")):
  path.unlink()

with (logos / "logo.png").open("w") as file:
  run([
    str(target / "cp437-to-png"),
    str(logos / "logo.ans"),
  ], stdout=file)
with (logos / "logo.svg").open("w") as file:
  run([
    str(target / "cp437-to-svg"),
    str(logos / "logo.ans"),
  ], stdout=file)

run([
  "ln",
  "-sf",
  "logo.png",
  str(logos / "full.png"),
])

run([
  "magick",
  str(logos / "full.png"),
  "-resize",
  "5%",
  str(logos / "small.png"),
])

run([
  "magick",
  str(logos / "full.png"),
  "-resize",
  "2%",
  str(logos / "tiny.png"),
])
