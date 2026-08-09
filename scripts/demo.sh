#!/usr/bin/env bash
# Lance AfterBudget en mode démonstration : base de démonstration (fausses
# données) dans demo/ (ignoré par git), aucune donnée réelle touchée.
#
# Usage :
#   scripts/demo.sh            # carnet fictif « utilisé depuis des mois »
#   scripts/demo.sh --onboarding  # écran de premier lancement
set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/seed_demo_db.sh "$@"

DATA_HOME="$PWD/demo"
if [ "${1:-}" = "--onboarding" ]; then
  DATA_HOME="$PWD/demo/onboarding"
fi

echo "Lancement d'AfterBudget avec la base de démonstration (XDG_DATA_HOME=$DATA_HOME)…"
echo "Appuyez sur Ctrl+C pour quitter."
XDG_DATA_HOME="$DATA_HOME" cargo run --quiet
