.PHONY: dev build test lint clean install

install:
	npm ci

dev:
	npm run dev

build:
	npm run build

test:
	@echo "No JavaScript test script configured; see README Verification for Rust checks."
	@exit 1

lint:
	@echo "No lint script configured; npm run build performs the TypeScript check."
	@exit 1

clean:
	rm -rf node_modules dist .next .turbo
