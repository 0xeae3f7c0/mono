#!/usr/bin/env bash
# Shortcut for check.sh (any prefix of "check" works).
exec bash "$(dirname "$0")/check.sh" "$@"
