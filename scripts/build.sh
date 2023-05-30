build_clean=true
use_polonius=true

# exit when any command fails
set -e

# to request sudo in the beginning
sudo uname -r

rust_version="stable"
if $use_polonius
then
  rust_version="nightly"
fi

rustup install $rust_version
rustup default $rust_version
rustup update

cd ..

if $build_clean
then
rm -f ./Cargo.lock
rm -rf ./target
cargo clean
fi

cd ./src
CARGO_INCREMENTAL=0
RUSTFLAGS="-Ctarget-cpu=native"

if $use_polonius
then
  RUSTFLAGS="-Z polonius $RUSTFLAGS"
fi

echo $RUSTFLAGS

# release build
cargo +$rust_version rustc --release -- $RUSTFLAGS

# polonius debug build
#cargo +nightly rustc -- -Z polonius

# polonius release build
#cargo +nightly rustc --release -- -Z polonius

cd ../scripts
chmod +x ./run.sh
./run.sh
