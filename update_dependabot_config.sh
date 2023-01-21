#!/usr/bin/env bash
cat <<EOF > .github/dependabot.yml
# file automatically generated using update_dependabot.sh, re-run the script to update this file!
version: 2
updates:
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
EOF

export LC_ALL=C
for fld in $(find . -type f -name Cargo.toml | sort | sed 's#/Cargo.toml##' | sed 's#./#/#') ; do
    cat <<EOF >> .github/dependabot.yml
  - package-ecosystem: "cargo"
    directory: "${fld}"
    schedule:
      interval: "weekly"
    rebase-strategy: "disabled"
EOF
done
