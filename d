#!/usr/bin/env bash
# Shortcut for deps.sh (any prefix of "deps" works).
exec bash "$(dirname "$0")/deps.sh" "$@"
