default:
	which cc || sudo apt-get -y install build-essential
	which cross || cargo install cross
	cross build --release --target arm-unknown-linux-musleabi
	ls -al ./target/arm-unknown-linux-musleabi/release/
podman:

run:
	cross run --target arm-unknown-linux-musleabi
