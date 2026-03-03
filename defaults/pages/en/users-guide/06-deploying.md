---
title: Deploying
order: 6
---

## Deploying

After building your site with `docs-gen build`, you have a directory of static HTML files ready to deploy anywhere.

### GitHub Pages

There are two ways to deploy to GitHub Pages.

#### Option A: Deploy from a branch

Build locally, commit the output, and let GitHub serve it directly.

1. Set `hostname` and `base_path` in `config.toml`:

   ```toml
   [site]
   hostname = "https://username.github.io"
   base_path = "/my-repo"
   ```

2. Build and push:

   ```bash
   docs-gen build . docs
   git add docs
   git commit -m "Build docs"
   git push
   ```

3. Go to your repository's **Settings > Pages**.
4. Set **Source** to **Deploy from a branch**.
5. Choose your branch (e.g. `main`) and folder `/docs`, then click **Save**.

Your site will be available at `https://username.github.io/my-repo/`.

#### Option B: Build with GitHub Actions

Use this if you prefer not to commit build artifacts. A workflow builds and deploys on every push.

1. Set `hostname` and `base_path` in `config.toml` (same as Option A).

2. Add `.github/workflows/docs.yml` to your repository:

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

3. Go to your repository's **Settings > Pages**.
4. Set **Source** to **GitHub Actions**.

> If you use a custom domain (e.g. `docs.example.com`), set `hostname` to that domain and `base_path = ""`.

### Other Hosting Services

docs-gen outputs plain static HTML with no server-side requirements. The `docs/` directory can be deployed to any static hosting service — Netlify, Vercel, Cloudflare Pages, S3, or your own web server.

For services that build from source (Cloudflare Pages, Netlify, etc.), set the build command to `cargo install docs-gen && docs-gen build . docs` and the output directory to `docs`.

Just ensure `hostname` and `base_path` in `config.toml` match your final URL. For services that serve from the root of a domain, set `base_path = ""`.

Next: [Creating Themes](../07-creating-themes/)
