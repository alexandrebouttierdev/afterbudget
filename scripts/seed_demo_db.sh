#!/usr/bin/env bash
# Recrée la base SQLite de démonstration (aucune donnée réelle).
#
# Usage :
#   scripts/seed_demo_db.sh              # carnet fictif utilisé depuis des mois
#   scripts/seed_demo_db.sh --onboarding # base vierge (écran de bienvenue)
#
# La base est écrite dans demo/ (ignoré par git) et l'application s'y pointe
# via XDG_DATA_HOME :
#   XDG_DATA_HOME="$PWD/demo" ./target/debug/afterbudget
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run --quiet --bin seed_demo -- "$@"
echo "Base de démonstration prête dans demo/ — aucune donnée réelle touchée."
