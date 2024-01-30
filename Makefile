default:
	build deploy

build:
	cargo build --release

deploy:
	cp ./target/release/ls-proxy /usr/local/bin	
