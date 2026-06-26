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

# base64 with NO line wrapping, so it's a single clean blob (no stray newlines
# or marker lines that could get pasted into the secret by mistake).
B64="$(base64 < "$WORKDIR/cert.p12" | tr -d '\n')"

# Save it to a file and copy it straight to the clipboard so there's nothing to
# hand-select. The earlier "decode base64" failures came from copying the
# surrounding ----- marker lines; this avoids that entirely.
OUT_DIR="${HOME}/textmacro-signing"
mkdir -p "$OUT_DIR"
printf '%s' "$B64" > "$OUT_DIR/APPLE_CERTIFICATE.txt"
printf '%s' "$B64" | pbcopy 2>/dev/null && COPIED="yes" || COPIED="no"

cat <<EOF

✅  Self-signed code-signing certificate created: "${IDENTITY}"

Register these 4 GitHub repository secrets
(Settings → Secrets and variables → Actions → New repository secret):

  1. APPLE_SIGNING_IDENTITY        ${IDENTITY}
  2. APPLE_CERTIFICATE_PASSWORD    ${P12_PASS}
  3. KEYCHAIN_PASSWORD             ${KEYCHAIN_PW}
  4. APPLE_CERTIFICATE             (base64 — see below)

  ⚠️  For APPLE_CERTIFICATE, paste ONLY the base64 text. Do NOT include any
      "-----" lines. To avoid mistakes it has been:
        • copied to your clipboard: ${COPIED}  (just ⌘V into the secret box)
        • saved to: ${OUT_DIR}/APPLE_CERTIFICATE.txt
          (or run:  pbcopy < "${OUT_DIR}/APPLE_CERTIFICATE.txt" )

Optional — sign LOCAL builds too:
  cp "$WORKDIR/cert.p12" ~/textmacro-cert.p12
  security import ~/textmacro-cert.p12 -P "${P12_PASS}" -T /usr/bin/codesign
  APPLE_SIGNING_IDENTITY="${IDENTITY}" npm run tauri build

Temp files: $WORKDIR  (delete when done:  rm -rf "$WORKDIR")
EOF
