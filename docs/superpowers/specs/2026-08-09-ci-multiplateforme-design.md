# Design — CI GitHub Actions multi-plateforme + Release « Latest »

Date : 2026-08-09
Statut : approuvé

## Objectif

À chaque `git push origin master` (ou `workflow_dispatch`), GitHub Actions :

1. vérifie le projet (fmt, check, clippy, tests, audit) ;
2. compile l'application en release sur 4 plateformes (Ubuntu, Fedora, macOS, Windows) ;
3. génère un package adapté à chaque plateforme ;
4. rassemble les fichiers dans `builds/` ;
5. publie les artifacts Actions ;
6. met à jour la release GitHub permanente `latest`.

Aucune compilation locale.

## Architecture

```
quality (ubuntu-latest)
   ↓
   ├── build-ubuntu   (ubuntu-latest, .deb garanti + .AppImage best-effort)
   ├── build-fedora   (container fedora:latest, .rpm)
   ├── build-macos    (macos-latest, .dmg universel arm64 + x86_64)
   └── build-windows  (windows-latest, .exe)
           ↓
        publish (contents: write)
           ↓
    Release GitHub « Latest » (tag `latest`)
```

## Fichiers

| Fichier | Action |
|---|---|
| `.github/workflows/build.yml` | nouveau — workflow principal |
| `.github/workflows/ci.yml` | trigger réduit aux pull requests |
| `Cargo.toml` | ajout `[package.metadata.deb]` et `[package.metadata.generate-rpm]` |
| `packaging/afterbudget.desktop` | nouveau — entrée desktop Linux |

## Détails

### quality (ubuntu-latest, permissions read)
`cargo fmt --check`, `cargo check --locked --all-targets --all-features`,
`cargo clippy --locked --all-targets --all-features -- -D warnings`,
`cargo test --locked --all-targets --all-features`, doc-tests,
`rustsec/audit-check` (reprend la CI existante). Un échec bloque tout.

### build-ubuntu (ubuntu-latest)
Dépendances apt : `pkg-config libx11-dev libxkbcommon-dev libxkbcommon-x11-dev
libfontconfig1-dev libgtk-3-dev librsvg2-bin`. Build
`cargo build --locked --release --all-features`. Icône PNG générée par
`rsvg-convert` depuis `assets/icones/portefeuille.svg` (aucun binaire dans le
repo). `.deb` via `cargo install cargo-deb` + `cargo deb`. `.AppImage` via
linuxdeploy + plugin GTK + libs dlopen (libEGL/libGLESv2/libxkbcommon/libfontconfig)
avec `continue-on-error` (best-effort, ne bloque jamais la release).

### build-fedora (container fedora:latest)
`dnf install rust cargo pkgconfig libX11-devel libxkbcommon-devel
libxkbcommon-x11-devel fontconfig-devel gtk3-devel librsvg2-tools rpm-build git`.
Build release dans Fedora, `.rpm` via `cargo install cargo-generate-rpm` +
`cargo generate-rpm`. Cache Rust avec préfixe `fedora-container` (clés distinctes
du job Ubuntu).

### build-macos (macos-latest)
Toolchain stable + targets `aarch64-apple-darwin` et `x86_64-apple-darwin`,
2 builds release, `lipo -create` (binaire universel), bundle
`AfterBudget.app/Contents/MacOS/afterbudget` + `Info.plist` (version lue depuis
`Cargo.toml`), codesign ad-hoc (`codesign --force --deep --sign -`), `.dmg` via
`hdiutil`.

### build-windows (windows-latest)
`cargo build --locked --release --all-features`, artifact = `afterbudget.exe`.

### publish (ubuntu-latest, seul job `permissions: contents: write`)
`needs: [build-ubuntu, build-fedora, build-macos, build-windows]`. Nom du binaire
lu depuis `Cargo.toml` (`sed`). `actions/download-artifact@v4` → normalisation
dans `builds/` : `afterbudget-<plateforme>-latest.<ext>` (extension détectée,
multi-fichiers supportés). Vérification : chaque plateforme doit produire au
moins un fichier, sinon échec (pas de release). Publication avec
`softprops/action-gh-release@v2` : tag `latest` permanent, nom `Latest`,
`overwrite: true` (les anciens assets sont remplacés).

## Sécurité

- `concurrency: group: build-${{ github.ref }}, cancel-in-progress: true`
- Permissions : `contents: read` au niveau workflow, `contents: write` uniquement
  sur `publish` ; `GITHUB_TOKEN` uniquement, aucun secret dans le YAML.
- Cache `Swatinem/rust-cache@v2` (registry/git/target, keyé OS + Cargo.lock).

## Limites connues

- `.AppImage` : pièce la plus fragile (libs chargées par `dlopen` par wgpu) —
  d'où le best-effort ; le `.deb` reste garanti.
- Pas de signature code (macOS/Windows) faute de certificats dans le dépôt ;
  le `.dmg` est signé ad-hoc (avertissement Gatekeeper au premier lancement).
