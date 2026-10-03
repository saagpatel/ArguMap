.PHONY: dev build test lint clean install

install:
	npm ci

dev:
	npm run dev

build:
	npm run build

test:
	cargo test --locked --manifest-path src-tauri/Cargo.toml

lint:
	@echo "No lint script configured; npm run build performs the TypeScript check."
	@exit 1

clean:
	rm -rf node_modules dist .next .turbo
