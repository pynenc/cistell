.PHONY: install
install: ## Install dependencies, build the Python extension, and set up pre-commit hooks
	@echo "🚀 Installing dependencies"
	@uv sync --all-extras
	@echo "🚀 Building and installing cistell-py in develop mode"
	@uv run maturin develop --uv --release -m crates/cistell-py/Cargo.toml
	@echo "🚀 Installing pre-commit hooks"
	@uv run pre-commit install

.PHONY: check
check: ## Run all code quality checks (pre-commit)
	@echo "🚀 Running all checks via pre-commit"
	@uv run pre-commit run --all-files

.PHONY: build
build: clean-build ## Build the Python wheel and sdist
	@echo "🚀 Creating wheel file"
	@uvx maturin build --release -m crates/cistell-py/Cargo.toml --out dist --sdist

.PHONY: build-rust
build-rust: ## Build all Rust crates (excludes cistell-py cdylib)
	@echo "🚀 Building Rust workspace"
	@cargo build --workspace --exclude cistell-py

.PHONY: develop
develop: ## Build and install cistell-py in develop mode
	@echo "🚀 Building and installing package in develop mode"
	@uv run maturin develop --uv --release -m crates/cistell-py/Cargo.toml

.PHONY: test-python
test-python: develop ## Run Python tests with pytest
	@echo "🚀 Testing Python: Running pytest"
	@uv run pytest tests/ --cov --cov-config=pyproject.toml --cov-report=xml

.PHONY: test-rust
test-rust: ## Run Rust tests
	@echo "🚀 Testing Rust: Running cargo test"
	@cargo test --workspace --exclude cistell-py

.PHONY: test
test: test-rust test-python ## Run all tests (Rust + Python)

.PHONY: clean-build
clean-build: ## Clean build artifacts
	@echo "🚀 Removing build artifacts"
	@rm -rf dist build
	@cargo clean

.PHONY: clean
clean: ## Clean coverage data
	@echo "Cleaning previous coverage data and HTML reports..."
	@rm -f .coverage .coverage.*
	@rm -rf htmlcov

.PHONY: publish-python
publish-python: build ## Publish the Python package to PyPI
	@echo "🚀 Publishing Python package to PyPI"
	@uvx twine upload dist/*

.PHONY: publish-rust
publish-rust: ## Publish Rust crates to crates.io (in dependency order)
	@echo "🚀 Publishing Rust crates to crates.io"
	cargo publish -p cistell-macros
	sleep 30
	cargo publish -p cistell-core

.PHONY: docs-install
docs-install: ## Install documentation dependencies (uv docs group)
	@echo "🚀 Installing documentation dependencies"
	@uv sync --group docs

.PHONY: docs-render
docs-render: docs-install ## Render documentation to docs/_build/html
	@echo "🚀 Building cistell documentation"
	@rm -rf docs/_build
	@uv run --group docs python -m sphinx -W -b html docs/ docs/_build/html
	@echo "📖 Docs built — open docs/_build/html/index.html"

.PHONY: docs
docs: docs-render ## Build documentation

.PHONY: docs-build
docs-build: docs-render ## Build documentation (compat alias)

.PHONY: docs-serve
docs-serve: docs-render ## Serve documentation locally
	@echo "🚀 Serving docs at http://localhost:8080"
	@uv run --group docs python -m http.server 8080 --directory docs/_build/html

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
