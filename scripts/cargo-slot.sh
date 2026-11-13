#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Runs cargo in one of N build slots, each with its own target directory, so
# several builds can run at once without sharing cargo's build-directory lock.
# Every build is capped at 8 GB of virtual memory and 4 jobs.
#   scripts/cargo-slot.sh check -q -p radolint-core --tests
SLOTS=${RADOLINT_SLOTS:-4}
BASE=${RADOLINT_TARGETS:-/media/devansh/Experiment/playground/hadolint-rust-targets}
BUSY=75
while :; do
  i=0
  while [ "$i" -lt "$SLOTS" ]; do
    flock -n -E $BUSY /tmp/hadolint-rust-slot-$i.lock \
      sh -c 'ulimit -v 8000000; export CARGO_TARGET_DIR="$0"; exec timeout 1200 nice cargo "$@"' \
      "$BASE/slot-$i" "$@"
    rc=$?
    [ "$rc" -ne $BUSY ] && exit "$rc"
    i=$((i + 1))
  done
  sleep 2
done
