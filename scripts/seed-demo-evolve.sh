#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO="$ROOT/data/repos/demo-evolve"

rm -rf "$REPO"
mkdir -p "$REPO"
cd "$REPO"

git init -q
git config user.email "genoma@local"
git config user.name "GENOMA"

cat > sample.txt <<'EOF'
GENOMA evolve seed v1
stable structure line
EOF
git add sample.txt
git commit -q -m "v1: seed structure"

cat > sample.txt <<'EOF'
GENOMA evolve seed v2
stable structure line
added entropy noise abc123xyz
more byte diversity !@#$%
EOF
git add sample.txt
git commit -q -m "v2: diversify content"

cat > sample.txt <<'EOF'
GENOMA evolve seed v3
stable structure line
added entropy noise abc123xyz
more byte diversity !@#$%
repetition block XXXXXXXXXXXXXXXXXXXX
repetition block XXXXXXXXXXXXXXXXXXXX
EOF
git add sample.txt
git commit -q -m "v3: introduce repetition"

echo "Seeded $REPO with $(git rev-list --count HEAD) commits"
