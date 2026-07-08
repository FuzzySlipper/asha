#!/usr/bin/env bash
# ECRP term-gravity gate.
#
# ASHA's vocabulary is a steering mechanism: ECS-framework terms pull coding
# agents toward generic component bags, hidden schedulers, and World-owned
# authority. This check turns the vocabulary ADR (Den: asha/ecrp-vocabulary-taxonomy,
# repo: docs/ecrp-capability-rule-ownership.md) into a mechanical gate.
#
# Tier 1 — banned outright in Rust/TS source:
#   - declaring types named *Component / *Archetype
#   - the identifier "Archetype"/"archetype" anywhere
#   Fix: use Capability / EntityDefinition vocabulary.
#
# Tier 2 — legacy-gated: WorldState/WorldBundle-family names are allowed only in
#   files listed in legacy-term-allowlist.txt (pre-vocabulary code with an owning
#   removal task) or on lines carrying an explicit `vocab-allow: <reason>` marker
#   (e.g. migration phrasing that names the legacy type deliberately).
#   Fix: use SessionState / RuntimeSession / ProjectBundle vocabulary.
#
# Prose comments are free to discuss rejected ECS terminology; tier 1 only
# matches type declarations, not comment text.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ALLOWLIST="$REPO_ROOT/harness/vocab/legacy-term-allowlist.txt"

TIER1_DECL='(struct|enum|trait|type|interface|class)[[:space:]]+[A-Za-z_]*(Component|Archetype)\b'
TIER1_WORD='[Aa]rchetype'
TIER2='WorldState|WorldBundle|world_state|world_bundle|worldState|worldBundle|world-bundle'
SUPPRESS='vocab-allow:'

self_test() {
  local fail=0
  echo "pub struct HealthComponent {" | grep -qE "$TIER1_DECL" || { echo "self-test: tier1 missed Rust Component decl"; fail=1; }
  echo "export interface HudComponent {" | grep -qE "$TIER1_DECL" || { echo "self-test: tier1 missed TS Component decl"; fail=1; }
  echo "let archetype = pick();" | grep -qE "$TIER1_WORD" || { echo "self-test: tier1 missed archetype identifier"; fail=1; }
  echo "/// Components per vertex (e.g. 3)." | grep -qE "$TIER1_DECL" && { echo "self-test: tier1 false-positive on prose"; fail=1; }
  echo "pub struct WorldBundleManifest {" | grep -qE "$TIER2" || { echo "self-test: tier2 missed WorldBundleManifest"; fail=1; }
  echo "pub type WorldScalar = f64;" | grep -qE "$TIER2" && { echo "self-test: tier2 false-positive on world-space types"; fail=1; }
  if [ "$fail" -ne 0 ]; then
    echo "term-gravity self-test FAILED"
    exit 1
  fi
  echo "term-gravity self-test passed"
}

if [ "${1:-}" = "--self-test" ]; then
  self_test
  exit 0
fi

is_allowlisted() {
  local file="$1"
  local entry
  while IFS= read -r entry; do
    [ -z "$entry" ] && continue
    case "$entry" in \#*) continue ;; esac
    case "$entry" in
      */) case "$file" in "$entry"*) return 0 ;; esac ;;
      *) [ "$file" = "$entry" ] && return 0 ;;
    esac
  done < "$ALLOWLIST"
  return 1
}

violations=0
allowed_report="$(mktemp)"
trap 'rm -f "$allowed_report"' EXIT

report() {
  local file="$1" line="$2" text="$3" advice="$4"
  echo "VOCAB: $file:$line: $text"
  echo "       -> $advice"
  violations=$((violations + 1))
}

while IFS= read -r file; do
  case "$file" in
    */dist/*|*/generated/*|*/node_modules/*|*/target/*) continue ;;
  esac
  [ -f "$REPO_ROOT/$file" ] || continue

  while IFS= read -r hit; do
    line="${hit%%:*}"; text="${hit#*:}"
    case "$text" in *"$SUPPRESS"*) continue ;; esac
    report "$file" "$line" "$text" \
      "ECS-gravity type name. Use Capability (typed authority facet) or EntityDefinition (stored template). See docs/ecrp-capability-rule-ownership.md."
  done < <(grep -nE "$TIER1_DECL|$TIER1_WORD" "$REPO_ROOT/$file" || true)

  if is_allowlisted "$file"; then
    allowed_hits="$(
      (grep -nE "$TIER2" "$REPO_ROOT/$file" 2>/dev/null || true) \
        | (grep -vF "$SUPPRESS" || true) \
        | wc -l \
        | tr -d ' '
    )"
    if [ "${allowed_hits:-0}" -gt 0 ]; then
      printf '%s\t%s\n' "$file" "$allowed_hits" >> "$allowed_report"
    fi
  else
    while IFS= read -r hit; do
      line="${hit%%:*}"; text="${hit#*:}"
      case "$text" in *"$SUPPRESS"*) continue ;; esac
      report "$file" "$line" "$text" \
        "Legacy World* naming. New code uses SessionState/RuntimeSession (live authority) and ProjectBundle (durable content). Deliberate legacy references need 'vocab-allow: <reason>' on the line; whole legacy files belong in harness/vocab/legacy-term-allowlist.txt."
    done < <(grep -nE "$TIER2" "$REPO_ROOT/$file" || true)
  fi
done < <(git -C "$REPO_ROOT" ls-files --cached --others --exclude-standard -- 'engine-rs/crates/**/*.rs' 'ts/packages/**/*.ts')

if [ -s "$allowed_report" ]; then
  echo "Remaining allowed legacy World vocabulary:"
  sort "$allowed_report" | while IFS="$(printf '\t')" read -r file count; do
    echo "  $file ($count hit(s))"
  done
else
  echo "Remaining allowed legacy World vocabulary: none"
fi

if [ "$violations" -ne 0 ]; then
  echo ""
  echo "term-gravity check FAILED: $violations violation(s)."
  echo "Vocabulary source of truth: Den doc asha/ecrp-vocabulary-taxonomy."
  exit 1
fi

echo "term-gravity check passed"
