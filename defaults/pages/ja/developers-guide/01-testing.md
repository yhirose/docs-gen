---
title: 開発ツール
order: 1
---

## 開発ツール

docs-gen の開発には [just](https://github.com/casey/just) コマンドランナーを使います。`just` を実行すると利用可能なコマンドの一覧が表示されます。

### 前提条件

- **Rust** — `rustup` でインストール
- **just** — `cargo install just` または `brew install just`
- **Firefox + geckodriver** — E2E テスト用。`brew install geckodriver`

### コマンド

#### just install

docs-gen をローカルにインストールします:

```bash
just install    # cargo install --path .
```

他のコマンド（`just build`, `just serve` など）は `docs-gen` バイナリを使うため、変更後は先にこれを実行してください。

#### just test

ユニットテストと E2E テストをまとめて実行します:

```bash
just test
```

内部では以下が順に実行されます:

1. `cargo test --release` — ユニットテスト
2. `cargo test --release --test e2e -- --ignored --test-threads=1` — E2E テスト

E2E テストは [thirtyfour](https://crates.io/crates/thirtyfour)（Rust 用 WebDriver クライアント）と Firefox ヘッドレスモードを使います。テストハーネスが geckodriver と `docs-gen serve` をそれぞれポート 4444 と 8123 で自動起動します。ポート競合を避けるため、シングルスレッドで実行されます。

#### just build

docs-gen 自身のドキュメントサイト（`docs-src/`）をビルドします:

```bash
just build              # default テーマでビルド
just build monotone     # テーマを指定
```

#### just serve

ドキュメントサイトをライブリロード付きで起動し、ブラウザを開きます:

```bash
just serve              # default テーマで起動
just serve monotone     # テーマを指定
```

#### just check

ドキュメントサイトのソースを検証します（壊れたリンク、order の問題など）:

```bash
just check
```

#### just clean

ビルド出力（`docs*/`）を削除します:

```bash
just clean
```

#### just release

新しいバージョンをリリースします:

```bash
just release 0.4.0
```

リリーススクリプト（`scripts/release`）が以下を行います:

1. バージョン形式の検証と未コミットの変更がないことを確認
2. main ブランチの最新 CI がパスしていることを確認
3. 新バージョンが現在より大きいことを確認
4. `Cargo.toml` と `defaults/config.toml` のバージョンを更新
5. コミット、タグ付け、プッシュ

プッシュ後、Publish ワークフロー（`publish.yml`）が CI を再実行し、パスすれば crates.io への公開と GitHub Release の作成を行います。
