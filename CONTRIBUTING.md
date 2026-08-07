# Contributing to LiqVid

First off, thank you for considering contributing to LiqVid! It's people like you that make LiqVid a great tool for Computational Fluid Dynamics.

## How Can I Contribute?

### Reporting Bugs
If you find a bug in the solver or UI, please create an issue using the Bug Report template. Be sure to include:
- A clear and descriptive title.
- Steps to reproduce the bug.
- Information about your environment (OS, Rust version, etc.).
- Expected vs. actual behavior.

### Suggesting Enhancements
Have an idea for a new feature (e.g., a new turbulence model, integration, or UI improvement)? Open an issue using the Feature Request template. Provide as much detail as possible about the use case and how the enhancement would work.

### Submitting Pull Requests
1. Fork the repository and create your branch from `main`.
2. Make sure your code follows standard Rust formatting by running `cargo fmt`.
3. Ensure all tests and builds pass by running `cargo test` and `cargo build`.
4. If you've added new features, please add corresponding tests where applicable.
5. Update the documentation (`README.md` or other docs) if you are changing user-facing functionality.
6. Open a Pull Request with a clear title and description of your changes.

## Development Setup

To set up the project locally for development:

1. Clone your fork of the repo:
   ```bash
   git clone https://github.com/YOUR_USERNAME/LiqVid.git
   cd LiqVid
   ```
2. Build the project:
   ```bash
   cargo build
   ```
3. Run the tests:
   ```bash
   cargo test
   ```

We use standard Rust tooling, so `rust-analyzer` with your favorite editor (VSCode, Neovim) is highly recommended.

Thank you for contributing to LiqVid!
