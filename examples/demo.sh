#!/usr/bin/env bash
#
# Cairn end-to-end demo.
#
# Builds both tools, then walks through: hashing, a cache MISS that runs a
# "build" and caches its output, a cache HIT that skips the build, and a
