set -euxo pipefail

main() {
    # Prune for crates and check them
    find . -type f -name Cargo.toml -execdir cargo build ';'
}

main
