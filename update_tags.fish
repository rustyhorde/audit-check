#!/usr/bin/env fish
git tag -d v1; and \
git push gh :refs/tags/v1; and \
git tag -s "v1" -m "v1"; and \
git push gh --tags