---
title: テーマのカスタマイズ
order: 4
---

## テーマのカスタマイズ

テーマの色やスタイルを変更するには、テーマをプロジェクトにインストールして CSS を編集します。

### テーマをインストールする

`docs-gen init` で作成されたサイトは、ビルトインテーマをそのまま使います。テーマをカスタマイズしたい場合は、明示的にインストールします:

```bash
docs-gen theme install default my-docs
```

CSS とハイライト設定がプロジェクトにコピーされ、自由に編集できます:

```text
styles/default/
├── config.toml       # シンタックスハイライト設定
└── static/
    └── css/main.css  # すべてのスタイル（ライト/ダークテーマ、レイアウト）
```

> **注意:** インストールしなければ、docs-gen の更新時にテーマの改善が自動で反映されます。インストール後はローカルコピーが優先されるため、将来のアップデートを取り込むにはテーマを再インストールし、カスタマイズを手動で再適用してください。

カスタマイズ不要なファイルは削除すれば、ビルトインのデフォルトにフォールバックします。テンプレートや JavaScript も変更したい場合は `--with-base` オプションを使います（詳細は[テーマの構造と作成](../07-creating-themes/)を参照）。

### スタイル

すべてのスタイルは `styles/default/static/css/main.css` に入っています。スタイルシートは `[data-theme="light"]` セレクタでダークモードとライトモードの両方をサポートします。デフォルトはダークモードです。

色を変えるには、CSSカスタムプロパティを検索するか、該当するセレクタを直接編集してください。主なエリアはこちらです:

- **ヘッダー** — `.header`, `.header-nav`
- **サイドバー** — `.sidebar`, `.nav-section`, `.nav-list`
- **コンテンツ** — `.content`, `article`
- **コードブロック** — `pre`, `code`
- **検索モーダル** — `.search-overlay`, `.search-modal`

### シンタックスハイライト

コードブロックのハイライトは `styles/default/config.toml` で設定します:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

ダークテーマ: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

ライトテーマ: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

> 両方を設定すると、docs-gen は各コードブロックをダーク用・ライト用の2つの HTML として出力し、CSS でアクティブなテーマに応じて一方だけを表示します。

### テーマを切り替える

別のビルトインテーマに切り替えるには、`config.toml` の `theme` を変更するだけです（[設定](../03-configuration/)を参照）。

`config.toml` を編集せずにテーマを試すには、`--theme` フラグを使います:

```bash
docs-gen serve my-docs --theme monotone
```

次へ: [サイトのチェック](../05-checking-your-site/)
