## 設定ファイルについて


### 設定ファイルについて

本アプリケーションは YAML 形式の設定ファイルを読み込みます。  
ファイル名は `{実行ファイル名}.config.yaml` です（例: 実行ファイルが `myapp` の場合は `myapp.config.yaml`）。

#### 設定ファイルの読み込みと優先順位

設定は以下の順に読み込まれ、**後に読み込まれた設定が前の設定を上書き**します。

1. **システム全体用ディレクトリ**  
   `/etc/{exe_basename}/{exe_basename}.config.yaml`  
   例: `/etc/myapp/myapp.config.yaml`

2. **実行ファイルと同じディレクトリ**  
   `${実行ファイルのあるディレクトリ}/{exe_basename}.config.yaml`  
   例: `/usr/local/bin/myapp.config.yaml`

3. **カレントディレクトリ**  
   `./{exe_basename}.config.yaml`  
   例: `./myapp.config.yaml`

> 読み込まれなかった項目はデフォルト値が使用されます。

4. **CLI 引数**  
   最終的に CLI 引数は最優先で設定を上書きします。

> 上記の順に存在チェックされ、見つかった設定は順に読み込まれ、後に読み込まれた設定が前の設定を上書きします。
> 見つからなかった項目はデフォルト設定が使用されます。
> 最終的に CLI 引数は最優先で設定を上書きします。

---
以下は各セクションの設定項目です。

#### server
- **host**: サーバーをバインドするアドレスとポート (例: `"127.0.0.1:8080"`)
- **cors**: CORS を許可するオリジンのリスト (空配列なら許可なし)
- **static**: 静的ファイル配信ディレクトリ。未指定なら無効 (`null`)

#### content
- **markdown_dir**: Markdown の保存先 (例: `"output/markdown"`)
- **html_dir**: HTML 出力先 (例: `"output/public_html"`)
- **title_max_len**: タイトル最大文字数 (範囲 80–240)
- **body_max_len**: 本文最大文字数 (範囲 100–30000)
- **tag_max_len**: タグ最大文字数 (範囲 8–32)
- **caregory_max_len**: カテゴリー最大文字数 (範囲 8–32)
- **max_tags**: 最大タグ数 (範囲 1–100)
- **max_categories**: 最大カテゴリ数 (範囲 1–5)
- **template_dir**: テンプレート格納ディレクトリ
- **template_content**: コンテンツ用テンプレート (例: `"content.html"`)
- **template_index**: トップページ用テンプレート (例: `"index.html"`)
- **template_list**: 一覧ページ用テンプレート (例: `"list.html"`)

#### search
- **dictionary_dir**: 辞書ディレクトリ (例: `"data/dictionary"`)
- **index_dir**: インデックス保存先 (例: `"output/.index"`)
- **index_limit**: インデックス登録件数上限 (範囲 100–10000)
- **search_limit**: 検索結果の最大件数 (範囲 100–10000)
- **memory_budget_in_bytes**: Tantivy のメモリ予算 (範囲 10M–99999999)

#### security
- **issuer**: JWT の発行者名（省略時は実行ファイル名）
- **secret**: JWT シークレットキー（省略時は UUID v4 が自動生成される）
- **expire**: JWT の有効期限（秒数, 範囲 180–7776000）
- **user_file**: ユーザー情報ファイル (例: `"data/user.dat"`)

#### log
- **level**: ログレベル（例: `"debug"`, `"info"`）。省略または `null` で未設定。

---

#### CLI による設定上書き

以下の設定は CLI オプションで上書き可能です。  
CLI で指定した値は常に設定ファイルより優先されます。

| 設定項目 | CLI オプション | 説明 |
|----------|----------------|------|
| `server.host` | `--host <HOST>` | サーバーのホスト:ポート |
| `server.cors` | `--cors <LIST>` | CORS 許可リスト |
| `server.cors` | `--no-cors` | CORS 設定をクリア |
| `server.static_dir` | `--static-dir <DIR>` | 静的ファイル配信ディレクトリ |
| `server.static_dir` | `--no-static` | 静的ファイル配信を無効化 |
| `log.level` | `--log-level <LEVEL>` | ログレベル |
| `log.level` | `--no-log` | ログ出力を無効化 |

> CLI で上書き可能な項目は上記の通りで、その他の設定は設定ファイルまたはデフォルト値が使用されます。
#### 設定ファイルの例

```yaml
server:
  host: "0.0.0.0:3000"
  cors: []
  static: null

content:
  markdown_dir: "output/markdown"
  html_dir: "output/public_html"
  title_max_len: 80
  body_max_len: 5000
  tag_max_len: 16
  category_max_len: 16
  max_tags: 6
  max_categories: 3
  template_dir: "data/templates"
  template_content: "content.html"
  template_index: "index.html"
  template_list: "list.html"

search:
  dictionary_dir: "data/dictionary"
  index_dir: "output/.index"
  index_limit: 3000
  search_limit: 1000
  memory_budget_in_bytes: 50000000

c:
  issuer: "myapp"
  secret: "550e8400-e29b-41d4-a716-446655440000"
  expire: 86400
  user_file: "data/security/user.dat"

log:
  level: null
```

### ビルド
```
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project messense/rust-musl-cross:aarch64-musl cargo build --release
cp target/aarch64-unknown-linux-musl/release/ssg-mng ./bin/
```

### 実行
```
docker run --rm -it --mount type=bind,source="$(pwd)",target=/project -w /project --network shared_devcontainer_net -p 3333:3000 gcr.io/distroless/static-debian12 /project/bin/ssg-mng
```

### swagger-ui
```
docker run --rm -d -p 8080:8080 -e API_URL=openapi.yaml --name swagger-ui -v "$(pwd)"/swagger/openapi.yaml:/usr/share/nginx/html/openapi.yaml swaggerapi/swagger-ui 
```
```swagger-ui``` からリクエストを投げる場合、CORSに ```http://localhost:8080``` を設定する必要がある