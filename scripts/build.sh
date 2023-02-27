# exit when any command fails
set -e

# to request sudo in the beginning
sudo uname -r

cd ..
rm -f ./Cargo.lock
rm -rf ./target

rustup update
rustup default stable

cargo build --release

cd ./scripts
chmod +x ./run.sh
./run.sh

