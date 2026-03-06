---
title: テーマの作成
order: 7
---

## テーマの作成

> このページは、新しいビルトインテーマを追加したい **docs-gen コントリビューター**向けです。サイトの見た目をカスタマイズしたいだけなら、[テーマのカスタマイズ](../04-customizing-themes/)を参照してください。

docs-gen はコンパイル時に `defaults/themes/` からテーマを自動検出します。新しいビルトインテーマを追加するには、`defaults/themes/<name>/` に次の構造でディレクトリを作ります:

```
defaults/themes/<name>/
├── config.toml          # ハイライトテーマ設定
├── templates/
│   ├── base.html
│   ├── page.html
│   └── portal.html
└── static/
    ├── css/main.css
    ├── js/main.js
    └── favicon.svg
```

コードの変更は不要です。再コンパイルすればすぐにテーマが使えるようになります。

### テーマの `config.toml`

各テーマには独自の `config.toml` があり、シンタックスハイライトを設定します:

```toml
[highlight]
dark_theme = "base16-eighties.dark"   # ダークモード用テーマ
light_theme = "InspiredGitHub"        # ライトモード用テーマ（省略可）
```

| キー | デフォルト | 説明 |
|-----|----------|------|
| `dark_theme` | `base16-eighties.dark` | ダークモード用テーマ |
| `light_theme` | `InspiredGitHub` | ライトモード用テーマ。ダークとライト両方のコードブロックがレンダリングされ、CSSで切り替わる。 |

ハイライトテーマの一覧は[テーマのカスタマイズ — シンタックスハイライト](../04-customizing-themes/#syntax-highlighting)を参照してください。

### テンプレート

テンプレートには [Tera](https://keats.github.io/tera/) テンプレートエンジンを使います。テーマには3つのテンプレートが必要です:

- **base.html** — 共通 HTML シェル（ヘッダー、フッター、スクリプト）。他のテンプレートはすべてこれを継承します。
- **page.html** — サイドバーナビゲーション付きのコンテンツページ。
- **portal.html** — ホームページレイアウト（全幅、サイドバーなし）。

テンプレート変数とスタイリングの詳細は[テーマのカスタマイズ](../04-customizing-themes/)を参照してください。

### 静的アセット

`static/` ディレクトリには CSS、JavaScript、アイコンを配置します。ビルド時にそのまま出力先にコピーされます。最低限、以下のファイルが必要です:

- `css/main.css` — ダーク/ライトモード対応を含むすべてのスタイル
- `js/main.js` — 検索、テーマ切り替え、言語切り替え
- `favicon.svg` — サイトアイコン
