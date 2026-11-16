.PHONY: up down test test-integration build lint fmt

## Bring up Postgres, Anvil, and the bundler
up:
	docker compose up -d

## Tear down all containers and volumes
down:
	docker compose down -v

## Run the hermetic unit tier (no external deps required)
test:
	go test ./services/walletd/...
	cargo test --manifest-path services/signerd/Cargo.toml
	cd contracts/evm && forge test
	cd services/agent && python -m pytest -m "not integration"

## Run the tagged integration tier (requires `make up` first)
test-integration:
	go test -tags integration ./services/walletd/...
	cd services/agent && python -m pytest -m integration

## Build all services
build:
	go build ./services/walletd/cmd/walletd
	cargo build --manifest-path services/signerd/Cargo.toml --release
	cd contracts/evm && forge build

## Lint all services
lint:
	go vet ./services/walletd/...
	cargo clippy --manifest-path services/signerd/Cargo.toml -- -D warnings
	cd contracts/evm && forge build
	cd services/agent && ruff check src/

## Format all services
fmt:
	gofmt -w services/walletd/
	cargo fmt --manifest-path services/signerd/Cargo.toml
	cd services/agent && ruff format src/
