---
title: 設定
order: 3
---

## 設定

サイトの設定はすべてプロジェクトルートの `config.toml` に書きます。

### 基本構造

```toml
[system]
theme = "default"
langs = ["en", "ja"]

[site]
title = "My Project"
version = "1.0.0"
hostname = "https://example.github.io"
base_path = "/my-project"
footer_message = "© 2026 My Project."
```

### デプロイシナリオ

`hostname` と `base_path` の設定がURLの生成方法を決めます。

#### ローカル開発

ローカル開発では `hostname` の設定は不要です。`base_path` を空にするだけで動きます:

```toml
[site]
base_path = ""
```

これがデフォルトの状態です。`docs-gen serve` はそのまま動作します。

#### カスタムドメイン

サイトをドメインのルートから配信する場合（例: `https://docs.example.com/`）:

```toml
[site]
hostname = "https://docs.example.com"
base_path = ""
```

#### サブディレクトリ（GitHub Pages）

リポジトリ名の配下でホスティングする場合（例: `https://user.github.io/my-project/`）:

```toml
[site]
hostname = "https://user.github.io"
base_path = "/my-project"
```

### ナビゲーションボタン

`[[nav]]` エントリでサイトヘッダーにボタンを追加できます:

```toml
[[nav]]
label = "Guide"
path = "guide/"          # 内部パス（言語ごとに解決される）

[[nav]]
label = "GitHub"
url = "https://github.com/your/repo"   # 外部URL
```

### 多言語サポート

言語を追加するには、`langs` にリストします。最初のエントリがデフォルト言語です:

```toml
[system]
langs = ["en", "ja"]
```

そのあと `pages/ja/` の下に対応するページディレクトリを作ります。

> 複数の言語を設定すると、ヘッダーに言語切り替えボタンが自動で表示されます。

#### 単一言語サイト

単一言語のサイトでは、`langs` にエントリを1つだけ設定します:

```toml
[system]
langs = ["en"]
```

単一言語モードでは、URLに言語プレフィックスが付きません（例: `/en/guide/` ではなく `/guide/`）。言語切り替えボタンも非表示になります。

### リファレンス

#### `[system]`

| キー | 必須 | 説明 |
|-----|------|------|
| `theme` | いいえ | ビルトインテーマ名（デフォルト: `"default"`） |
| `langs` | はい | 言語コードのリスト。最初のエントリがデフォルト。 |

#### `[site]`

| キー | 必須 | 説明 |
|-----|------|------|
| `title` | はい | ヘッダーに表示するサイトタイトル |
| `version` | いいえ | ヘッダーに表示するバージョン文字列 |
| `hostname` | いいえ | ベースとなるホスト名（上の[デプロイシナリオ](#デプロイシナリオ)と下の[SEO](#seo)を参照） |
| `base_path` | いいえ | URLのパスプレフィックス（上の[デプロイシナリオ](#デプロイシナリオ)を参照） |
| `footer_message` | いいえ | フッターテキスト |

#### `[[nav]]` — ヘッダーボタン

| キー | 必須 | 説明 |
|-----|------|------|
| `label` | はい | ボタンのラベルテキスト |
| `path` | いいえ | `<lang>/` からの相対的な内部セクションパス |
| `url` | いいえ | 外部URL（`path` より優先される） |
| `icon_svg` | いいえ | インラインSVGアイコンのマークアップ |

### SEO

`hostname` を設定すると、docs-gen は SEO メタデータを自動生成します:

- **Canonical URL** — 各ページに `<link rel="canonical">` タグを出力し、重複コンテンツの問題を防ぎます。
- **hreflang タグ** — 多言語サイトでは、各ページに全言語と `x-default` の `<link rel="alternate" hreflang="...">` タグを出力します。これにより検索エンジンは `/en/` と `/ja/` のページが同一コンテンツの言語バリエーションであると認識できます。
- **sitemap.xml** — サイトルートに全ページの URL を含むサイトマップを生成します。多言語サイトでは各言語の `xhtml:link` alternate も含まれます。

これらの機能は `hostname` を設定するだけで有効になり、追加の設定は不要です。

次へ: [テーマのカスタマイズ](../04-customizing-themes/)
