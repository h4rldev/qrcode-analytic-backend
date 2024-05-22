default: watch

watch:
  @cargo watch -x "run --release"

build:
  @cargo build --release
  @cp ./target/release/qrcode-analytic .
