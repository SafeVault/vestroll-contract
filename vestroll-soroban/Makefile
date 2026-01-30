PARAMS=
CARGO_TEST_ARGS=

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

build: ## Build all contracts
	stellar contract build

test: ## Run all tests
	cargo test $(CARGO_TEST_ARGS)

fmt: ## Format code
	cargo fmt

clean: ## Clean build artifacts
	cargo clean

clippy: ## Run clippy for linting
	cargo clippy --all-targets -- -D warnings
