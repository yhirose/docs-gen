---
title: テーマを作る
order: 7
---

## テーマを作る

既存のテーマをベースに、独自のテーマを作成できます。まず手順を紹介し、その後テーマの内部構造を解説します。

### 手順

ここでは、組織共通の Base（`corporate`）を作り、それを元に独自の Style（`my-theme`）を作成する流れを紹介します。

#### 1. プロジェクトを作成し、テーマをインストールする

ベースにしたいテーマを `--with-base` 付きでインストールし、Base と Style の全ファイルをプロジェクトにコピーします:

```bash
docs-gen init my-theme-project
docs-gen theme install default my-theme-project --with-base
```

プロジェクトはこのようになります。

```text
# Base: 共通レイアウト基盤
bases/standard/
├── templates/
│   ├── base.html              # 共通 HTML シェル（ヘッダー、フッター、スクリプト）
│   ├── page.html              # サイドバー付きコンテンツページ
│   └── portal.html            # ホームページ（全幅、サイドバーなし）
└── static/
    ├── js/main.js             # 検索、テーマ切り替え、言語切り替え
    └── favicon.svg            # サイトアイコン

# Style: テーマ固有のファイル
styles/default/
├── config.toml                # シンタックスハイライト設定
├── DESIGN.md                  # 設計意図（人間と AI エージェント向け）
└── static/
    └── css/main.css           # すべてのスタイル
```

#### 2. テーマに名前を付ける

コピーされた `styles/default/` を `styles/my-theme/` のようにリネームし、プロジェクトの `config.toml` でテーマ名を合わせます:

```toml
# my-theme-project/config.toml
[system]
theme = "my-theme"
```

Base もリネームできます。例えば `bases/standard/` を `bases/corporate/` にリネームした場合、各 Style の `config.toml` で Base 名を合わせます:

```toml
# styles/my-theme/config.toml
[system]
base = "corporate"
```

リネーム後の構造です:

```text
bases/corporate/
├── templates/
│   ├── base.html
│   ├── page.html
│   └── portal.html
└── static/
    ├── js/main.js
    └── favicon.svg

styles/my-theme/
├── config.toml
├── DESIGN.md
└── static/
    └── css/main.css
```

#### 3. ファイルを編集する

プロジェクトにコピーされたファイルを編集します:

- `styles/my-theme/static/css/main.css` — 色、フォント、レイアウトを変更
- `styles/my-theme/config.toml` — シンタックスハイライトの設定を変更
- `styles/my-theme/DESIGN.md` — 変更内容に合わせて設計意図を更新
- `bases/corporate/templates/*.html` — HTML 構造を変更
- `bases/corporate/static/js/main.js` — 動的機能を変更

#### 4. テストする

`docs-gen serve` はファイルの変更を検知して自動でリロードします:

```bash
docs-gen serve my-theme-project
```

#### 5. 不要なファイルを削除する

変更しなかったファイルは削除できます。削除したファイルはビルトインのデフォルトにフォールバックします。

### テーマの構造: Base + Style

テーマは **Base** と **Style** の2つで構成されています:

- **Base** — テンプレート（HTML）、JavaScript、アイコンなど、レイアウトの共通基盤
- **Style** — CSS やハイライト設定など、テーマ固有のファイル

複数のテーマが同じ Base を共有できます。例えば `default` と `monotone` は CSS だけが異なり、Base は共通です。どの Base を使うかは、Style の `config.toml` で指定します:

```toml
# styles/default/config.toml
[system]
base = "standard"    # この Style は standard Base を使う
```

### Base

Base にはテンプレートと JavaScript が含まれています:

- **テンプレート** — ヘッダー、サイドバー、フッターなど、ページの HTML 構造を定義
- **JavaScript** — テンプレートと連携して、検索・テーマ切り替え・言語切り替えなどの動的機能を提供

```text
bases/standard/
├── templates/
│   ├── base.html              # 共通 HTML シェル（ヘッダー、フッター、スクリプト）
│   ├── page.html              # サイドバー付きコンテンツページ
│   └── portal.html            # ホームページ（全幅、サイドバーなし）
└── static/
    ├── js/main.js             # 検索、テーマ切り替え、言語切り替え
    └── favicon.svg            # サイトアイコン
```

#### テンプレート

`bases/standard/templates/` に3つのテンプレートがあります（文法は [Tera](https://keats.github.io/tera/) を参照）:

- **base.html** — 共通の HTML シェル。ヘッダー、フッター、検索モーダル、テーマ検出スクリプトを含みます。他のテンプレートはすべてこれを継承します。
- **page.html** — コンテンツページ用。`base.html` を継承し、セクションナビゲーション付きのサイドバーを追加します。
- **portal.html** — ホームページ用（言語ルートの `index.md`）。`base.html` を継承し、全幅レイアウトでサイドバーはありません。

テンプレートを編集する際は、JavaScript が参照する HTML 要素やクラス名を変えないよう注意してください。

#### テンプレート変数

docs-gen がビルド時にテンプレートに渡す変数です。`{{ site.title }}` のように参照します。値はプロジェクトの `config.toml` やビルド処理から生成されます:

| 変数 | 説明 |
| --- | --- |
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

### Style

Style にはテーマの見た目を決めるファイルが含まれています:

- **CSS** — 色、フォント、レイアウトの余白などすべてのビジュアルスタイル
- **config.toml** — 使用する Base の指定と、シンタックスハイライトの設定

```text
styles/my-theme/
├── config.toml       # [system] base = "standard" + ハイライト設定
├── DESIGN.md         # 設計意図（任意、記述を推奨）
└── static/
    └── css/main.css  # すべてのスタイル
```

#### ライト/ダークモード

CSS はダークモードがデフォルトです。ライトモード用のスタイルは `[data-theme="light"]` セレクタで定義します:

```css
/* ダークモード（デフォルト） */
:root {
    --bg-color: #1a1a2e;
}

/* ライトモード */
[data-theme="light"] {
    --bg-color: #ffffff;
}
```

#### シンタックスハイライト

コードブロックのハイライトテーマは `config.toml` で設定します:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

ダークテーマ: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

ライトテーマ: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

両方を設定すると、docs-gen は各コードブロックをダーク用・ライト用の2つの HTML として出力し、CSS でアクティブなテーマに応じて一方だけを表示します。

#### DESIGN.md を書く（推奨）

独自のテーマを作る際は、`styles/<your-theme>/DESIGN.md` を追加することを推奨します。これはテーマの設計意図を記述したプレーンテキスト文書で、ビジュアル哲学、各 CSS トークンの意味、編集時に守るべきルールなどを書きます。

書くことは任意で、ビルドには一切影響しませんが、次の3つの場面で役立ちます:

1. **将来の自分** — 数ヶ月後にテーマを再び触るときに、なぜその色やレイアウトを選んだかを思い出すため。
2. **コラボレーターやフォークする人** — テーマのビジュアルアイデンティティを壊さずに拡張してもらうため。
3. **AI コーディングエージェント** — Claude Code や Cursor のようなツールが自動的に `DESIGN.md` を読み、設計に沿った編集を行うため。

参考として `styles/default/DESIGN.md` と `styles/monotone/DESIGN.md` を見てください。フォーマットは強制されていません — テーマに合わせて自由に構成してよいですが、これらのファイルは記述する価値のある領域（設計思想、カラートークン、レイアウト、タイポグラフィ、コンポーネント、Do/Don't）をカバーしています。
