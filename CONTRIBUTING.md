# Contributing to Devflow

First off, thank you for considering contributing to Devflow! It's people like you that make Devflow such a great tool.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as it might be a known problem. When you are creating a bug report, please include as many details as possible according to the issue template.

### Suggesting Enhancements

We welcome feature requests! Please use the enhancement template to describe your idea.

### Pull Requests

1.  Fork the repo and create your branch from `dev`.
2.  If you've added code that should be tested, add tests.
3.  If you've changed APIs, update the documentation.
4.  Ensure the test suite passes (`make verify`).
5.  Make sure your code lints and is formatted (`make dev`).

## Development Process

We use a standard Rust development workflow. The `Makefile` in the root directory provides several helpers:

*   `make setup`: Prepare the environment.
*   `make dev`: Format, lint, and test.
*   `make verify`: Comprehensive local verification.

## Questions?

Feel free to open an issue with the `question` label.
