#!/usr/bin/env bash

run_advisory() {
  local check_id="$1"
  local owner="$2"
  local next_action="$3"
  shift 3

  if "$@"; then
    return 0
  fi

  echo "WARN: advisory check ${check_id} failed; owner=${owner}; next=${next_action}" >&2
  return 0
}
