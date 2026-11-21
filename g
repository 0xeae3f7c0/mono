#!/usr/bin/env bash
# Shortcut for gen.sh (any prefix of "gen" works).
exec bash "$(dirname "$0")/gen.sh" "$@"
