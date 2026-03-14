---
title: デプロイ
order: 6
---

## デプロイ

`docs-gen build` でサイトをビルドすると、静的 HTML ファイルのディレクトリができます。どこにでもデプロイできます。

### GitHub Pages

GitHub Pages へのデプロイには2つの方法があります。

#### 方法A: ブランチからデプロイ

ローカルでビルドし、出力をコミットして、GitHub に直接配信させます。

1. `config.toml` で `hostname` と `base_path` を設定します（詳細は[設定 — デプロイシナリオ](../03-configuration/#deployment-scenarios)を参照）:

   ```toml
   [site]
   hostname = "https://username.github.io"
   base_path = "/my-repo"
   ```

2. ビルドしてプッシュします:

   ```bash
   docs-gen build . docs
   git add docs
   git commit -m "Build docs"
   git push
   ```

3. リポジトリの **Settings > Pages** を開きます。
4. **Source** を **Deploy from a branch** にします。
5. ブランチ（例: `main`）とフォルダ `/docs` を選んで **Save** をクリックします。

`https://username.github.io/my-repo/` でサイトが公開されます。

#### 方法B: GitHub Actions でビルド

ビルド成果物をコミットしたくない場合はこちらを使います。プッシュのたびにワークフローがビルドとデプロイを行います。

1. `config.toml` で `hostname` と `base_path` を設定します（上の方法Aと同じ）。

2. リポジトリに `.github/workflows/docs.yml` を追加します:

   ```yaml
   name: Docs
   on:
     push:
       branches: [main]
   permissions:
     contents: read
     pages: write
     id-token: write
   concurrency:
     group: ${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: true
   jobs:
     deploy:
       runs-on: ubuntu-latest
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       steps:
         - uses: actions/checkout@v4
         - uses: Swatinem/rust-cache@v2
         - name: Install docs-gen
           run: cargo install docs-gen
         - name: Build
           run: docs-gen build . docs
         - uses: actions/configure-pages@v5
         - uses: actions/upload-pages-artifact@v3
           with:
             path: docs
         - id: deployment
           uses: actions/deploy-pages@v4
   ```

3. リポジトリの **Settings > Pages** を開きます。
4. **Source** を **GitHub Actions** にします。

> カスタムドメイン（例: `docs.example.com`）を使う場合は、`hostname` をそのドメインにし、`base_path = ""` にしてください。

### その他のホスティングサービス

docs-gen はサーバーサイドの要件がない純粋な静的 HTML を出力します。`docs/` ディレクトリは Netlify、Vercel、Cloudflare Pages、S3、自前のWebサーバーなど、どの静的ホスティングサービスにもデプロイできます。

ソースからビルドするサービス（Cloudflare Pages、Netlify など）では、ビルドコマンドを `cargo install docs-gen && docs-gen build . docs` に、出力ディレクトリを `docs` に設定してください。

`config.toml` の `hostname` と `base_path` が最終的なURLと一致していることを確認しましょう。ドメインのルートから配信するサービスでは `base_path = ""` にします。

次へ: [テーマの作成](../07-creating-themes/)
