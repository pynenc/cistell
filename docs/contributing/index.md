# Contributing to Cistell

The project is in early development and not yet open for external contributions. Once the API stabilises, this guide will be updated with contribution instructions.

In the meantime, feel free to open issues or participate in discussions on the repository.

```{toctree}
:hidden:
:maxdepth: 2
:caption: Detailed Use Cases

./docs
```

## Setting Up the Development Environment

1. **Fork and clone** the repository:

   ```bash
   git clone https://github.com/pynenc/cistell.git
   cd cistell
   ```

2. **Install uv** (dependency manager): see <https://docs.astral.sh/uv/getting-started/installation/\>.

3. **Install dependencies**:

   ```bash
   uv sync
   ```

4. **Install Rust toolchain** (needed for the native extension): see <https://rustup.rs/\>.

5. **Build the native extension** (development mode):

   ```bash
   maturin develop
   ```

6. **Install pre-commit hooks**:

   ```bash
   uv run pre-commit install
   ```

7. **Run tests**:

   ```bash
   uv run pytest tests/
   cargo test --workspace --exclude cistell-py
   ```

## Running Tests with Coverage

```bash
uv run coverage run -m pytest
uv run coverage report
```
