---
title: Testing
order: 1
---

## Testing

### Unit Tests

```bash
cargo test
```

### End-to-End Tests

E2e tests use [thirtyfour](https://crates.io/crates/thirtyfour) (Rust WebDriver client) with Firefox headless via geckodriver.

**Prerequisites:**

- Firefox installed
- geckodriver: `brew install geckodriver`

```bash
cargo test --test e2e -- --ignored --test-threads=1
```

The test harness automatically starts geckodriver and `docs-gen serve` on ports 4444 and 8123 respectively. Tests must run single-threaded (`--test-threads=1`) to avoid port conflicts.
