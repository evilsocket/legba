#!/usr/bin/env python3
import os
import re

if os.system("nerve run changelog -q") != 0:
    print("failed to generate changelog")
    quit()

version_match_re = r'^version\s*=\s*"([^"]+)"$'
with open('Cargo.toml', 'rt') as fp:
    manifest = fp.read()

# parse current version and get next from user
m = re.findall(version_match_re, manifest, re.MULTILINE)
if len(m) != 1:
    print("could not parse current version from Cargo.toml")
    quit()

current_ver = m[0]
next_ver = input("current version is %s, enter next: " % current_ver)

# generate new manifest
result = re.sub(version_match_re, 'version = "%s"' %
                next_ver, manifest, 0, re.MULTILINE)
with open('Cargo.toml', 'w+t') as fp:
    fp.write(result)

# make sure Cargo.lock is up to date
print("cargo update -p legba")

# commit, push and create new tag
print("git add Cargo.*")
print("git commit -m 'releasing version %s'" % next_ver)
print("git push")
print("git tag -a %s -m 'releasing v%s'" % (next_ver, next_ver))
print("git push origin %s" % next_ver)

print()

# publish on crates.io
print("cargo publish")
