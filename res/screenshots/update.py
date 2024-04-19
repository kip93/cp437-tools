#!/usr/bin/env python3

from pathlib import Path
import selenium.common
from selenium import webdriver
from selenium.webdriver.common.by import By
from subprocess import run
from time import sleep

root = Path(__file__).parent.parent.parent
target = root / "target/release"
tests = root / "res/test"
screenshots = root / "res/screenshots"

(screenshots / "svg.png").unlink()

options = webdriver.firefox.options.Options()
options.add_argument("--headless")
options.add_argument("--disable-gpu")
with webdriver.Firefox(options=options) as driver:
  driver.get(f"file://{tests / 'background.svg'}")
  sleep(1)
  root = driver.find_element(By.TAG_NAME, "svg")
  root.screenshot(str(screenshots / "svg.png"))

run([
  "magick",
  str(screenshots / "svg.png"),
  "-resize",
  "5%",
  str(screenshots / "svg.png"),
])

(screenshots / "png.png").unlink()

run([
  "magick",
  str(tests / "background.png"),
  "-resize",
  "5%",
  str(screenshots / "png.png"),
])
