#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
ROUNDS="${ROUNDS:-2}"
SLEEP_MS="${SLEEP_MS:-250}"

echo "[fixture] chain_id=AOXC-0077-MAIN"
echo "[fixture] TEST ONLY seeds are public; do not reuse outside local/dev environments."

echo "[fixture] bootstrapping atlas" 
$AOXC_BIN node-bootstrap --home "$ROOT_DIR/homes/atlas" >/tmp/aoxc-atlas-bootstrap.json
$AOXC_BIN node-run --home "$ROOT_DIR/homes/atlas" --rounds "$ROUNDS" --sleep-ms "$SLEEP_MS" --tx-prefix "ATLAS-TX" >/tmp/aoxc-atlas-run.json
echo "[fixture] bootstrapping boreal" 
$AOXC_BIN node-bootstrap --home "$ROOT_DIR/homes/boreal" >/tmp/aoxc-boreal-bootstrap.json
$AOXC_BIN node-run --home "$ROOT_DIR/homes/boreal" --rounds "$ROUNDS" --sleep-ms "$SLEEP_MS" --tx-prefix "BOREAL-TX" >/tmp/aoxc-boreal-run.json
echo "[fixture] bootstrapping cypher" 
$AOXC_BIN node-bootstrap --home "$ROOT_DIR/homes/cypher" >/tmp/aoxc-cypher-bootstrap.json
$AOXC_BIN node-run --home "$ROOT_DIR/homes/cypher" --rounds "$ROUNDS" --sleep-ms "$SLEEP_MS" --tx-prefix "CYPHER-TX" >/tmp/aoxc-cypher-run.json
echo "[fixture] bootstrapping delta" 
$AOXC_BIN node-bootstrap --home "$ROOT_DIR/homes/delta" >/tmp/aoxc-delta-bootstrap.json
$AOXC_BIN node-run --home "$ROOT_DIR/homes/delta" --rounds "$ROUNDS" --sleep-ms "$SLEEP_MS" --tx-prefix "DELTA-TX" >/tmp/aoxc-delta-run.json
echo "[fixture] bootstrapping ember" 
$AOXC_BIN node-bootstrap --home "$ROOT_DIR/homes/ember" >/tmp/aoxc-ember-bootstrap.json
$AOXC_BIN node-run --home "$ROOT_DIR/homes/ember" --rounds "$ROUNDS" --sleep-ms "$SLEEP_MS" --tx-prefix "EMBER-TX" >/tmp/aoxc-ember-run.json
