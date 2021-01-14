cd rust
cargo build --release
cargo test --release --quiet
cd ..

python main.py
