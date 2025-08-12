.PHONY:
check-all:
	@echo "Cargo check..."
	@cargo check
	@echo "Format check..."
	@cargo fmt --check
	@echo "Clippy check..."
	@cargo clippy

.PHONY:
fix:
	@cargo fmt
	@cargo clippy --fix --allow-dirty --allow-staged

.PHONY:
test:
	@cargo test --all-features
