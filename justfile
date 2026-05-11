build:
  cargo build --release --features capi

test:
  cargo test

lint:
  cargo clippy --all-features -- -Dwarnings
