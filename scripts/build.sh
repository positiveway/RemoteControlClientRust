# exit when any command fails
set -e

rustup update
rustup default stable

cd ..
rm -rf ./target
cargo build --release

cd ./scripts
chmod +x ./run.sh
./run.sh

