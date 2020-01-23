set -euxo pipefail

main() {
    rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabihf
}

main
