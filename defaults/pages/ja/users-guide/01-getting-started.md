---
title: はじめよう
order: 1
---

## はじめよう

1分もかからずにドキュメントサイトを作れます。

### 1. プロジェクトを初期化する

```bash
docs-gen init my-docs
```

次のような構造が作られます:

```
my-docs/
├── config.toml              # サイト設定
└── pages/
    ├── en/                      # 英語ページ
    │   ├── index.md             # ホームページ
    │   ├── users-guide/         # セクション（独自のサイドバーを持つ）
    │   │   ├── index.md
    │   │   └── *.md
    │   └── developers-guide/    # コントリビューター向けセクション
    │       ├── index.md
    │       └── *.md
    └── ja/                      # 日本語ページ（en/ と同じ構造）
```

テーマはあとからカスタマイズできます。詳しくは[テーマのカスタマイズ](../04-customizing-themes/)をどうぞ。

### 2. ローカルでプレビューする

```bash
docs-gen serve my-docs --open
```

`--open` フラグをつけると、デフォルトブラウザが自動で開きます。ローカルサーバーが `http://localhost:8080` で起動し、ライブリロードが有効になります。

> Markdown ファイル、`config.toml`、テンプレートを保存するたびに、ブラウザが自動でリロードします。手動でリフレッシュする必要はありません。

### 3. 本番用にビルドする

```bash
docs-gen build my-docs docs
```

`docs/` ディレクトリに静的 HTML が生成されます。あとはデプロイするだけです。

> **ヒント:** 先に `cd my-docs` しておけば、すべてのコマンドでディレクトリ引数を省略できます:
>
> ```bash
> mkdir my-docs && cd my-docs
> docs-gen init                 # docs-gen init my-docs と同じ
> docs-gen serve --open         # docs-gen serve my-docs --open と同じ
> docs-gen build . docs         # docs-gen build my-docs docs と同じ
> ```

次へ: [ページを書く](../02-writing-pages/)
