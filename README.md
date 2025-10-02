# ssg-mng

`ssg-mng` は、静的サイトジェネレーター (SSG) で利用する Markdown コンテンツを管理するためのバックエンドAPIサーバーです。
Web API を通じて記事の作成、編集、削除、検索などの操作を提供し、コンテンツ管理を効率化することを目的としています。

## 主な機能

- **コンテンツ管理**: Markdown 形式の記事の CRUD (作成, 読み取り, 更新, 削除) 機能。
- **日本語対応の検索**: `Tantivy` と日本語トークナイザーを組み合わせ、ユーザー辞書も利用可能な高精度な全文検索を実現。
- **認証**: JWT を利用したユーザー認証機能。
- **柔軟な設定**: YAML 形式の設定ファイルとコマンドラインオプションによる詳細な設定。
- **静的ファイル配信**: ビルド済みの HTML や画像などの静的ファイルを配信する機能。
- **API ドキュメント**: `Swagger (OpenAPI)` による API 仕様の提供。

## 技術スタック

- **言語**: Rust (stable)
- **Webフレームワーク**: Axum
- **検索エンジン**: Tantivy
- **トークナイザー**: `lindera-tantivy` (日本語形態素解析)
- **テンプレートエンジン**: Tera

## ビルドと実行

### ローカル環境

Rust のビルド環境がセットアップされている場合は、`cargo` コマンドで直接ビルド・実行できます。

```bash
# ビルド
cargo build --release

# 実行 (デバッグログを有効にする例)
./target/release/ssg-mng --log-level debug
```

### Docker 環境

ローカルにRustのビルド環境がない場合は、Dockerを使用してビルド・実行することができます。

**ビルド (クロスコンパイル)**
```bash
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project messense/rust-musl-cross:aarch64-musl cargo build --release
cp target/aarch64-unknown-linux-musl/release/ssg-mng ./bin/
```

### 実行
```
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project --network shared_devcontainer_net -p 3333:3000 gcr.io/distroless/static-debian12 /project/bin/ssg-mng
```

## フロントエンド開発

`swagger-ui`で使える`openapi.yaml`を用意しています。

### swagger-ui
```
docker run --rm -d -p 8888:8080 -e API_URL=openapi.yaml --name swagger-ui -v "$(pwd)"/swagger/openapi.yaml:/usr/share/nginx/html/openapi.yaml swaggerapi/swagger-ui 
```
```swagger-ui``` からリクエストを投げる場合、CORSに ```http://localhost:8888``` を設定する必要があります
