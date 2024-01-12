default:
	build deploy

build:
	cargo build --release

deploy:
	cp ./target/release/lsp-proxy /usr/local/bin	
