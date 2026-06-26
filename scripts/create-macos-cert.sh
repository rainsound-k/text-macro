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
P12_PASS="$(openssl rand -base64 18)"
KEYCHAIN_PW="$(openssl rand -base64 18)"

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

B64="$(base64 < "$WORKDIR/cert.p12")"

cat <<EOF

✅  Self-signed code-signing certificate created: "${IDENTITY}"

Register these as GitHub repository secrets
(Settings → Secrets and variables → Actions → New repository secret):

  APPLE_SIGNING_IDENTITY        ${IDENTITY}
  APPLE_CERTIFICATE_PASSWORD    ${P12_PASS}
  KEYCHAIN_PASSWORD             ${KEYCHAIN_PW}
  APPLE_CERTIFICATE             ← the base64 block below (copy all lines)

----- BEGIN APPLE_CERTIFICATE (base64) -----
${B64}
----- END APPLE_CERTIFICATE -----

Optional — sign LOCAL builds too:
  cp "$WORKDIR/cert.p12" ~/textmacro-cert.p12
  security import ~/textmacro-cert.p12 -P "${P12_PASS}" -T /usr/bin/codesign
  APPLE_SIGNING_IDENTITY="${IDENTITY}" npm run tauri build

Temp files: $WORKDIR
Delete them when done:  rm -rf "$WORKDIR"
EOF
