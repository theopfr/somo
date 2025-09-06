# Contributing
Thanks for considering to contribute to ``somo`` 🎉 !

## Git conventions:
There are currently no strict rules for branch naming, commit style, or similar conventions. Changelog entries are maintained by the core maintainer, so you don't need to worry about updating them in your PRs.

## Setup:
Make sure you have the Rust toolchain installed. You can get it from [rustup.rs](https://rustup.rs/).

Then, fork the repository on GitHub and clone your fork:#
```bash
git clone git@github.com:{your-username}/somo.git
cd somo

# Build
cargo build

# Run
cargo run -- --some-flag
```

## Linting:
For CI to pass, the code must be formatted and linted. Please run the following when opening a PR:
```bash
cargo fmt
cargo clippy --fix
```

## Testing:
When adding or modifying a feature, you are strongly encouraged to include **unit tests** and possiblty **integration tests**.

Run all tests with:
```bash
cargo test
```

### Unit tests:
Unit tests live in the same file as the code they test, usually in a #[cfg(test)] module at the bottom.

Run only unit tests with:
```bash
cargo test --bin somo
```

### Integration tests (Linux only):
Integration tests run the somo binary and check its raw stdout.
Instead of using your host's /proc filesystem, somo reads from a mocked procfs located at ``tests/mock/proc/``. This mock directory contains dummy netcat processes, which makes the tests deterministic and reproducible.

##### Updating the mock data:
If you need to modify the mocked processes, edit ``tests/setup/init_processes.sh``. Then regenerate the mock procfs with:
```bash
bash tests/setup/generate_mock_procfs.sh
```

This script starts a Docker container, launches isolated network-bound processes inside it, and copies the relevant /proc files back into tests/mock/proc/ on the host.

Run only integration tests with:
```bash
cargo test --test integration_tests
```