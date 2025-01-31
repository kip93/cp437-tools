#!/usr/bin/env python3

from itertools import chain
from os import listdir
from pathlib import Path
from subprocess import run

root = Path(__file__).parent.parent.parent
target = root / "target/release"
schemes = root / "res/schemes"

for path in schemes.glob("*.png"):
  path.unlink()

with (schemes / "CLASSIC.png").open("w") as file:
  run([
    str(target / "cp437-to-png"),
    str(schemes / "CLASSIC.ans"),
    "CLASSIC",
  ], stdout=file)
run([
  "magick",
  str(schemes / "CLASSIC.png"),
  "-resize",
  "10%",
  str(schemes / "CLASSIC.png"),
])

with (schemes / "MODERN.png").open("w") as file:
  run([
    str(target / "cp437-to-png"),
    str(schemes / "MODERN.ans"),
    "MODERN",
  ], stdout=file)
run([
  "magick",
  str(schemes / "MODERN.png"),
  "-resize",
  "10%",
  str(schemes / "MODERN.png"),
])
