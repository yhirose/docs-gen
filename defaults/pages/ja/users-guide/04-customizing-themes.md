---
title: テーマのカスタマイズ
order: 4
---

## テーマのカスタマイズ

docs-gen にはビルトインテーマが同梱されていますが、テーマをプロジェクトにインストールしてファイルを編集すれば、見た目を自由にカスタマイズできます。

### テーマをインストールする

テーマファイルは `init` 時にはコピーされません。テーマをカスタマイズするには、明示的にインストールします:

```bash
docs-gen theme install default my-docs
```

`my-docs/themes/default/` にテーマファイルがコピーされるので、自由に編集できます。

> インストールされた `config.toml` には、インストール時の docs-gen バージョンが記録されます。

インストール後のテーマディレクトリ構造はこうなります:

```
themes/default/
├── config.toml              # シンタックスハイライト設定
├── templates/
│   ├── base.html            # 共通レイアウト（ヘッダー、フッター、スクリプト）
│   ├── page.html            # サイドバー付きのコンテンツページ
│   └── portal.html          # ホームページ（全幅、サイドバーなし）
└── static/
    ├── css/main.css          # すべてのスタイル（ライト/ダークテーマ、レイアウト）
    ├── js/main.js            # 検索、テーマ切り替え、言語切り替え
    └── favicon.svg           # サイトアイコン
```

### テンプレート

テンプレートには [Tera](https://keats.github.io/tera/) テンプレートエンジン（Jinja2 に似ています）を使います。テンプレートは3種類あります:

- **base.html** — 共通の HTML シェル。ヘッダー、フッター、検索モーダル、テーマ検出スクリプトを含みます。他のテンプレートはすべてこれを継承します。
- **page.html** — コンテンツページ用。`base.html` を継承し、セクションナビゲーション付きのサイドバーを追加します。
- **portal.html** — ホームページ用（言語ルートの `index.md`）。`base.html` を継承し、全幅レイアウトでサイドバーはありません。

#### テンプレート変数

| 変数 | 説明 |
|-----|------|
| `site.title` | `config.toml` のサイトタイトル |
| `site.version` | バージョン文字列 |
| `site.base_url` | 完全なベースURL（`hostname` + `base_path`） |
| `site.base_path` | URLのベースパス |
| `site.footer_message` | フッターHTML |
| `site.nav` | ナビゲーションエントリ（各エントリは `label`、`path` または `url` を持つ） |
| `site.langs` | 設定済み言語のリスト |
| `site.single_lang` | 言語が1つだけのとき `true` |
| `page.title` | 現在のページタイトル |
| `page.status` | ページステータス（例: `"draft"`） |
| `lang` | 現在の言語コード |
| `lang_prefix` | 言語パスプレフィックス（例: `"/en"`）。単一言語サイトでは空 |
| `content` | レンダリング済みページHTML。エスケープせずに出力するには `\| safe` を使う |
| `nav` | サイドバーナビゲーションツリー（`page.html` 内） |

### スタイル

すべてのスタイルは `static/css/main.css` に入っています。スタイルシートは `[data-theme="light"]` セレクタでダークモードとライトモードの両方をサポートします。デフォルトはダークモードです。

色を変えるには、CSSカスタムプロパティを検索するか、該当するセレクタを直接編集してください。主なエリアはこちらです:

- **ヘッダー** — `.header`, `.header-nav`
- **サイドバー** — `.sidebar`, `.nav-section`, `.nav-list`
- **コンテンツ** — `.content`, `article`
- **コードブロック** — `pre`, `code`
- **検索モーダル** — `.search-overlay`, `.search-modal`

### シンタックスハイライト

コードブロックのハイライトはテーマの `config.toml` で設定します:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

ダークテーマ: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

ライトテーマ: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

> 両方を設定すると、docs-gen は各コードブロックを2バージョンレンダリングし、アクティブなテーマに応じて CSS で切り替えます。

### JavaScript

`static/js/main.js` がインタラクティブ機能を担当します:

- **テーマ切り替え** — ダークモードとライトモードを切り替え、設定を `localStorage` に保存します
- **言語切り替え** — 複数言語が設定されていると自動で表示されます
- **検索** — 検索ボタンまたは `⌘K` / `Ctrl+K` で起動するクライアントサイド全文検索

### テーマを切り替える

別のビルトインテーマを使うには、プロジェクトの `config.toml` で `theme` の値を変更し（[設定](../03-configuration/)を参照）、インストールします:

```bash
docs-gen theme install monotone my-docs
```

`config.toml` を編集せずにテーマを試すには、`--theme` フラグを使います:

```bash
docs-gen serve my-docs --theme monotone
```

次へ: [サイトのチェック](../05-checking-your-site/)
