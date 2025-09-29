#!/usr/bin/env bash
set -euo pipefail

DIC_DIR="_files/data/dictionary"
TEMPLATES_HTML="_files/data/templates"
PUBLIC_HTML="_files/output/public_html"
MARKDOWN_DIR="_files/output/markdown"
INDEX_DIR="_files/output/.index"
USER_DIR="_files/data/security"

mkdir -p "$PUBLIC_HTML"

for d in $(find "$PUBLIC_HTML" -maxdepth 1 -mindepth 1 -type d); do
  name=$(basename "$d")
  if [[ "$name" =~ ^[0-9]+$ ]]; then
    rm -rf "$d"
  fi
done

rm -rf "$MARKDOWN_DIR"
mkdir -p "$MARKDOWN_DIR"

rm -rf "$INDEX_DIR"
mkdir -p "$INDEX_DIR"

rm -rf "$USER_DIR"
mkdir -p "$USER_DIR"

mkdir -p "$DIC_DIR"

mkdir -p "$TEMPLATES_HTML"
