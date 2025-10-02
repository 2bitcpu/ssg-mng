# 設定ファイルについて

本アプリケーションは YAML 形式の設定ファイルを読み込みます。  
ファイル名は `{実行ファイル名}.config.yaml` です（例: 実行ファイルが `ssg-mng` の場合は `ssg-mng.config.yaml`）。

#### 設定ファイルの読み込みと優先順位

設定は以下の順に読み込まれ、**後に読み込まれた設定が前の設定を上書き**します。

1. **システム全体用ディレクトリ**  
   `/etc/{exe_basename}/{exe_basename}.config.yaml`  
   例: `/etc/ssg-mng/ssg-mng.config.yaml`

2. **実行ファイルと同じディレクトリ**  
   `${実行ファイルのあるディレクトリ}/{exe_basename}.config.yaml`  
   例: `/usr/local/bin/ssg-mng.config.yaml`

3. **カレントディレクトリ**  
   `./{exe_basename}.config.yaml`  
   例: `./ssg-mng.config.yaml`

上記パスで見つかった設定ファイルは順番に読み込まれ、後に読み込まれた設定が前の設定を上書きします。設定ファイルで指定されなかった項目にはデフォルト値が適用されます。
最終的に、後述するコマンドライン (CLI) 引数で指定された設定が、設定ファイルの内容をさらに上書きします。

---

すべての設定項目は省略可能です。省略された項目には、以下に示される規定値が自動的に適用されます。

#### server
- **host**: サーバーをバインドするアドレスとポート (規定値: `"0.0.0.0:3000"`)
- **cors**: CORS を許可するオリジンのリスト (規定値: `[]`)
- **static**: 静的ファイル配信ディレクトリ (規定値: `null`)

#### content
- **markdown_dir**: Markdown の保存先 (規定値: `"output/markdown"`)
- **html_dir**: HTML 出力先 (規定値: `"output/public_html"`)
- **title_max_len**: タイトル最大文字数 (規定値: `80`, 範囲: 80–240)
- **description_max_len**: 説明文の最大文字数 (規定値: `300`, 範囲: 100-1000)
- **body_max_len**: 本文最大文字数 (規定値: `5000`, 範囲: 1000–30000)
- **tag_max_len**: タグ最大文字数 (規定値: `16`, 範囲: 8–32)
- **category_max_len**: カテゴリー最大文字数 (規定値: `16`, 範囲: 8–32)
- **max_tags**: 最大タグ数 (規定値: `6`, 範囲: 1–100)
- **max_categories**: 最大カテゴリ数 (規定値: `3`, 範囲: 1–5)
- **template_dir**: テンプレート格納ディレクトリ (規定値: `"data/templates"`)
- **template_content**: コンテンツ用テンプレート (規定値: `"content.html"`)
- **template_index**: トップページ用テンプレート (規定値: `"index.html"`)
- **template_list**: 一覧ページ用テンプレート (規定値: `"list.html"`)
- **template_recent**: 最近の記事一覧用テンプレート (規定値: `"recent.html"`)

#### search
- **dictionary_dir**: 辞書ディレクトリ (規定値: `"data/dictionary"`)
- **index_dir**: インデックス保存先 (規定値: `"output/.index"`)
- **index_limit**: インデックス登録件数上限 (規定値: `3000`, 範囲: 100–10000)
- **search_limit**: 検索結果の最大件数 (規定値: `1000`, 範囲: 100–10000)
- **memory_budget_in_bytes**: Tantivy のメモリ予算 (規定値: `50000000`, 範囲: 10M–99.9M)

#### security
- **issuer**: JWT の発行者名 (規定値: 実行ファイル名)
- **secret**: JWT シークレットキー (規定値: 自動生成されるUUID v4)
- **expire**: JWT の有効期限（秒数, 規定値: `86400` (1日), 範囲: 180–7776000)
- **lock_threshold**: アカウントロックされる試行回数 (規定値: `3`, 範囲: 1-10)
- **lock_seconds**: アカウントロック時間（秒数, 規定値: `3600` (1時間), 範囲: 60-86400)
- **update_interval**: ユーザーファイル更新間隔（秒数, 規定値: `1`, 範囲: 1-60)
- **allow_signup**: ユーザー登録を許可するかどうか (規定値: `false`)
- **user_file**: ユーザー情報ファイル (規定値: `"data/security/user.dat"`)

#### log
- **level**: ログレベル（例: `"info"`, `"debug"`）。(規定値: `null` (ログ出力なし))

---

#### 設定ファイルの例

```yaml
# 以下は、すべての設定項目を規定値で記述した例です。
# 実際には、変更したい項目のみを記述すれば問題ありません。
server:
  host: "0.0.0.0:3000"
  cors: []
  static: null

content:
  markdown_dir: "output/markdown"
  html_dir: "output/public_html"
  title_max_len: 80
  description_max_len: 300
  body_max_len: 5000
  tag_max_len: 16
  category_max_len: 16
  max_tags: 6
  max_categories: 3
  template_dir: "data/templates"
  template_content: "content.html"
  template_index: "index.html"
  template_list: "list.html"
  template_recent: "recent.html"

search:
  dictionary_dir: "data/dictionary"
  index_dir: "output/.index"
  index_limit: 3000
  search_limit: 1000
  memory_budget_in_bytes: 50000000

security:
  issuer: "ssg-mng" # 規定値は実行ファイル名
  secret: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" # 規定値は実行時に自動生成されるUUID
  expire: 86400
  lock_threshold: 3
  lock_seconds: 3600
  update_interval: 1
  allow_signup: false
  user_file: "data/security/user.dat"

log:
  level: null
```

---

# コマンドライン (CLI) オプション

設定ファイルの内容は、以下のコマンドラインオプションで上書きすることができます。

| オプション | 対応する設定 | 説明 |
|:---|:---|:---|
| `--host <HOST>` | `server.host` | サーバーのホストとポートを指定します。例: `127.0.0.1:8080` |
| `--cors <ORIGIN>...` | `server.cors` | CORSを許可するオリジンを複数指定できます。例: `--cors http://localhost:8080` |
| `--no-cors` | `server.cors` | CORS設定をすべてクリアします。 |
| `--static-dir <DIR>` | `server.static` | 静的ファイルを配信するディレクトリを指定します。 |
| `--no-static` | `server.static` | 静的ファイルの配信を無効化します。 |
| `--allow-signup` | `security.allow_signup` | 新規ユーザーの登録を許可します (`true` に設定)。 |
| `--no-allow-signup` | `security.allow_signup` | 新規ユーザーの登録を禁止します (`false` に設定)。 |
| `--log-level <LEVEL>` | `log.level` | ログレベルを指定します。例: `info`, `debug`, `trace` |
| `--no-log` | `log.level` | ログ出力を無効化します (`null` に設定)。 |

> `--allow-signup` と `--no-allow-signup` が同時に指定された場合、安全のため `--no-allow-signup` (登録禁止) が優先されます。
