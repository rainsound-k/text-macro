#!/usr/bin/env bash
#
# Create a self-signed macOS code-signing certificate for TextMacro and print
# the values to register as GitHub Actions secrets.
#
# Signing every release build with the SAME certificate gives the app a stable
# code identity, so the macOS Accessibility (손쉬운 사용) permission survives
# updates instead of being re-requested on every new DMG.
#
# Run once on your Mac:   bash scripts/create-macos-cert.sh
#
set -euo pipefail

IDENTITY="${1:-TextMacro Self Signed}"
WORKDIR="$(mktemp -d)"
# Hex passwords (no +,/,= ) so they paste cleanly and need no shell escaping.
P12_PASS="$(openssl rand -hex 18)"
KEYCHAIN_PW="$(openssl rand -hex 18)"

# 1) Self-signed certificate with the Code Signing extended key usage.
cat > "$WORKDIR/openssl.cnf" <<CNF
[req]
distinguished_name = dn
x509_extensions = v3
prompt = no
[dn]
CN = ${IDENTITY}
[v3]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CNF

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$WORKDIR/key.pem" -out "$WORKDIR/cert.pem" \
  -days 3650 -config "$WORKDIR/openssl.cnf" -extensions v3 >/dev/null 2>&1

# 2) Bundle key + cert into a password-protected .p12.
openssl pkcs12 -export \
  -inkey "$WORKDIR/key.pem" -in "$WORKDIR/cert.pem" \
  -out "$WORKDIR/cert.p12" -name "$IDENTITY" \
  -passout "pass:${P12_PASS}" >/dev/null 2>&1

# base64 with NO line wrapping, so it's a single clean blob.
B64="$(base64 < "$WORKDIR/cert.p12" | tr -d '\n')"

# Write ALL FOUR secret values to files from THIS single run, with no trailing
# newline (printf '%s'). The repeated "wrong password" failures came from mixing
# values across runs (cert from a file, password from old terminal scrollback).
# Copy each secret straight from its file so they always match.
OUT_DIR="${HOME}/textmacro-signing"
mkdir -p "$OUT_DIR"
printf '%s' "$IDENTITY"     > "$OUT_DIR/APPLE_SIGNING_IDENTITY.txt"
printf '%s' "$P12_PASS"     > "$OUT_DIR/APPLE_CERTIFICATE_PASSWORD.txt"
printf '%s' "$KEYCHAIN_PW"  > "$OUT_DIR/KEYCHAIN_PASSWORD.txt"
printf '%s' "$B64"          > "$OUT_DIR/APPLE_CERTIFICATE.txt"

cat <<EOF

✅  Self-signed code-signing certificate created: "${IDENTITY}"

All 4 secret values were written (no trailing newline) to:
  ${OUT_DIR}/

Set each GitHub secret by copying its file to the clipboard, then ⌘V into the
secret box (Settings → Secrets and variables → Actions). Run these one at a time:

  pbcopy < "${OUT_DIR}/APPLE_SIGNING_IDENTITY.txt"      # → APPLE_SIGNING_IDENTITY
  pbcopy < "${OUT_DIR}/APPLE_CERTIFICATE_PASSWORD.txt"  # → APPLE_CERTIFICATE_PASSWORD
  pbcopy < "${OUT_DIR}/KEYCHAIN_PASSWORD.txt"           # → KEYCHAIN_PASSWORD
  pbcopy < "${OUT_DIR}/APPLE_CERTIFICATE.txt"           # → APPLE_CERTIFICATE

⚠️  All four MUST come from THIS run. Don't reuse old values — the password and
    certificate are a matched pair and won't import if mixed.

Optional — sign LOCAL builds too:
  cp "$WORKDIR/cert.p12" ~/textmacro-cert.p12
  security import ~/textmacro-cert.p12 -P "\$(cat "${OUT_DIR}/APPLE_CERTIFICATE_PASSWORD.txt")" -T /usr/bin/codesign
  APPLE_SIGNING_IDENTITY="${IDENTITY}" npm run tauri build

When everything is set, delete the local copies:
  rm -rf "${OUT_DIR}" "$WORKDIR"
EOF
