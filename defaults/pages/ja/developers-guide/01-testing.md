---
title: テスト
order: 1
---

## テスト

### ユニットテスト

```bash
cargo test
```

### エンドツーエンドテスト

E2e テストは [thirtyfour](https://crates.io/crates/thirtyfour)（Rust 用 WebDriver クライアント）と Firefox ヘッドレスモード（geckodriver）を使います。

**前提条件:**

- Firefox がインストールされていること
- geckodriver: `brew install geckodriver`

```bash
cargo test --test e2e -- --ignored --test-threads=1
```

テストハーネスが geckodriver と `docs-gen serve` をそれぞれポート 4444 と 8123 で自動起動します。ポート競合を避けるため、テストはシングルスレッド（`--test-threads=1`）で実行してください。
