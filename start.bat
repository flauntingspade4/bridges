cd rust
cargo build --release
cargo test --release --quiet
cd ..

pip install -r requirements.txt

python python
