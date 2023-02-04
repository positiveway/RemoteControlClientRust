# exit when any command fails
set -e

cd ../target/release/
sudo nice -n -20 ./SocketControllerRust
