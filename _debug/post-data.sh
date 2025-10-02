#!/bin/bash

PWD=$(cd "$(dirname "$0")" && pwd)
JH="$PWD/jwt-header.txt"
CT="Content-Type: application/json"
readonly URL="http://localhost:3000/service/manage/content"

# curl -i -X POST -H "Content-Type: application/json" -d '{"account":"testuser","password":"P@55w0rd","confirmPassword":"P@55w0rd","email":"testuser@localhost"}' http://localhost:3000/service/manage/auth/signup
SIGNIN_URL="http://localhost:3000/service/manage/auth/signin"
ACCOUNT="testuser"
PASSWORD="P@55w0rd"

TOKEN=$(curl -s -X POST -H "$CT" \
  -d "{\"account\":\"$ACCOUNT\",\"password\":\"$PASSWORD\"}" \
  "$SIGNIN_URL" | jq -r '.token')
echo "$TOKEN"
echo "Authorization: Bearer $TOKEN" > "$JH"

# APIエンドポイントのURL

# JSONペイロードを生成する関数
generate_post_data() {
  # 引数としてループカウンタを受け取る
  local i=$1

  # テーマを決定 (20パターン)
  local theme=$((i % 20))

  local article_title
  local article_body
  local tags
  local categories
  local draft_status

  # 10件に1件を下書きにする
  if (( i % 10 == 0 )); then
    draft_status=true
  else
    draft_status=false
  fi

  # 日付を動的に生成 (i日前の日付)
  # GNU date (Linux)
  local dynamic_date
  if date --version >/dev/null 2>&1; then
      dynamic_date=$(date -u -d "-$i days" '+%Y-%m-%dT%H:%M:%SZ')
  # BSD date (macOS)
  else
      dynamic_date=$(date -u -v-${i}d '+%Y-%m-%dT%H:%M:%SZ')
  fi

  case $theme in
    0) # プログラミング (Rust)
      article_title="自動生成記事 ${i}：Rustでの非同期処理"
      article_body="これは${i}番目の記事です。Rustの非同期ランタイムTokioを使い、高効率なI/O処理を実装する方法を解説します。FutureやPinの概念にも触れ、実践的なコード例を提示します。"
      tags='["Rust", "非同期処理", "パフォーマンス"]'
      categories='["プログラミング", "バックエンド"]'
      ;;
    1) # プログラミング (Python)
      article_title="自動生成記事 ${i}：PythonとPandasでのデータ分析"
      article_body="これは${i}番目の記事です。PythonのライブラリPandasを使って、CSVデータを効率的に読み込み、集計・可視化する基本的な手法を紹介します。データサイエンスの第一歩です。"
      tags='["Python", "Pandas", "データ分析"]'
      categories='["プログラミング", "データサイエンス"]'
      ;;
    2) # プログラミング (Web)
      article_title="自動生成記事 ${i}：モダンなフロントエンド開発環境"
      article_body="これは${i}番目の記事です。ReactとViteを組み合わせた、高速で快適なフロントエンド開発環境の構築手順を説明します。TypeScriptの導入方法も合わせて解説します。"
      tags='["JavaScript", "React", "Vite", "TypeScript"]'
      categories='["プログラミング", "フロントエンド"]'
      ;;
    3) # プログラミング (DB)
      article_title="自動生成記事 ${i}：SQLアンチパターン"
      article_body="これは${i}番目の記事です。データベース設計やクエリで陥りがちなアンチパターンをいくつか紹介します。パフォーマンスのボトルネックを避けるためのヒントを提供します。"
      tags='["SQL", "データベース", "設計"]'
      categories='["プログラミング", "バックエンド"]'
      ;;
    4) # キャンプ (ソロ)
      article_title="自動生成記事 ${i}：週末のソロキャンプ体験"
      article_body="これは${i}番目の記事です。週末を利用してソロキャンプに行ってきました。自然の中で焚き火を眺めながら過ごす時間は最高のリフレッシュになります。おすすめのギアも紹介します。"
      tags='["キャンプ", "アウトドア", "ソロキャンプ"]'
      categories='["趣味", "ライフスタイル"]'
      ;;
    5) # キャンプ (ギア)
      article_title="自動生成記事 ${i}：最新軽量テントのレビュー"
      article_body="これは${i}番目の記事です。今年発売された最新の軽量テントを実際に使用してみました。設営のしやすさ、居住性、耐水圧など、様々な観点から詳しくレビューします。"
      tags='["キャンプ", "ギア", "テント"]'
      categories='["アウトドア", "レビュー"]'
      ;;
    6) # キャンプ (料理)
      article_title="自動生成記事 ${i}：ダッチオーブンで作る絶品キャンプ飯"
      article_body="これは${i}番目の記事です。ダッチオーブン一つで作れる、簡単で美味しいキャンプ料理のレシピを紹介します。ローストチキンは誰でも失敗なく作れます。"
      tags='["キャンプ飯", "料理", "ダッチオーブン"]'
      categories='["アウトドア", "グルメ"]'
      ;;
    7) # 釣り (海)
      article_title="自動生成記事 ${i}：港でのアジ釣り釣果"
      article_body="これは${i}番目の記事です。近所の港でサビキ釣りを試したところ、良型のアジが多数釣れました。仕掛けの工夫や、時合を読むコツについて考察します。釣った魚は美味しくいただきました。"
      tags='["釣り", "釣果報告", "サビキ釣り"]'
      categories='["趣味", "アウトドア"]'
      ;;
    8) # 釣り (川)
      article_title="自動生成記事 ${i}：渓流釣りの魅力と始め方"
      article_body="これは${i}番目の記事です。美しい自然に癒される渓流釣りの魅力について語ります。初心者向けの装備や、基本的な釣り方、マナーについて解説します。"
      tags='["釣り", "渓流釣り", "初心者"]'
      categories='["アウトドア", "入門"]'
      ;;
    9) # 釣り (ルアー)
      article_title="自動生成記事 ${i}：シーバスルアーの選び方"
      article_body="これは${i}番目の記事です。シーバスフィッシングで使うルアーの種類は多岐にわたります。ミノー、バイブレーション、シンペンなど、状況別の使い分けを解説します。"
      tags='["釣り", "ルアー", "シーバス"]'
      categories='["アウトドア", "テクニック"]'
      ;;
    *) # その他 (10-19)
      article_title="自動生成記事 ${i}：その他の趣味"
      article_body="これは${i}番目の記事です。料理、DIY、読書、映画、旅行、音楽など、様々な趣味に関する内容です。多様なデータパターンを生成するためのサンプル記事となります。"
      tags='["趣味", "ライフハック", "サンプル"]'
      categories='["その他", "コラム"]'
      ;;
  esac

  # IDをゼロパディングで生成
  local article_id
  article_id=$(printf "unique-article-id-%03d" "$i")

  # JSONデータを生成して返す
  cat <<EOF
{
  "matter": {
    "title": "$article_title",
    "description": "$article_title の概要です。", 
    "date": "$dynamic_date",
    "draft": $draft_status,
    "tags": $tags,
    "categories": $categories
  },
  "body": "$article_body"
}
EOF
}

for i in {1..300}; do
  # ログが多すぎるので、進捗表示はcurlに任せる
  JSON_DATA=$(generate_post_data "$i")

  # curlコマンドでJSONデータをPOSTする
  curl -s -o /dev/null -w "Article $i: HTTP Status %{http_code}\n" -X POST -H "$CT" -H @"$JH" -d "$JSON_DATA" "$URL"

  # サーバーへの負荷を軽減するために少し待機
  sleep 0.02
done

echo "All 300 requests have been sent."
