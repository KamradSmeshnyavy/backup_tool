#!/bin/bash
set -e
LATEST_KEY=$(ls -1t backups/*.key | head -1)
LATEST_ENC=$(ls -1t backups/*.tar.gz.enc | head -1)
echo "Restoring from: $LATEST_KEY and $LATEST_ENC"
mkdir -p restored
./target/release/backup_tool restore \
  --config config.toml \
  --secret-key recipient_secret.key \
  --enc-key "$LATEST_KEY" \
  --input "$LATEST_ENC" \
  --output restored
diff -rq testdata restored && echo "Success: restored files match original."
