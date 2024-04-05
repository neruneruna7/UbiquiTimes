# UbiquiTimes
Timesを複数サーバで共有する，実験的に作成したDiscord Bot．

# 概要
Timesを複数サーバで共有するDiscord Botである．
正確に言うならば，Discordのサーバーにおいて，特定のチャンネルをユーザー自身のTimesと指定し，任意のサーバー，チャンネルで書き込んだ内容を他のサーバーのTimes指定したチャンネルにも書き込むものである．

名前の由来としては，Ubiquitous(遍在する)とTimes(ソフトウェアエンジニア系のコミュニティにおいてよくある，内部Twitterのようなものだと思っている．由来は知らない)を組み合わせたものである．

また通常のDiscord Botとは異なる点がある．多くのDiscord Botでは，１つの稼働しているBotを，複数のDiscordサーバーに導入するという形式だと思われるが，Ubiquitimesは，１つのBotにつき，１つのDiscordサーバーという形になっている．

<!-- | 多くのbotの形式 | UbiquiTimesの形式 |
|----------|----------|
| ![1つのbotを複数のサーバに導入しているイメージ図](/images/ubiquitimes-art-1/normal_bot.png =300x)   | ![１つのbotにつき，１つのサーバ上で稼働しているイメージ図](/images/ubiquitimes-art-1/ubiquitimes_bot.png =300x)   | -->

# 目標
- Timesに書いた内容を複数のサーバへ拡散することができる
- １つのサーバにつき１つのbotという形式である
- 他のアプリにも対応できるようにする
  - そのために，特定のアプリごとへの依存性が低い手段を用いて，拡散を行う


# 動かすには
- 最新版のRustのインストール
- cargo-shuttleのインストール
- このリポジトリをクローン
- DiscordのBotを作成し，トークンを取得
- トークンを`ubiquitimes\shuttle\Secrets.toml`に配置
- `cargo shuttle run`を実行
  - または，
  - `cargo shuttle login`
  - `cargo shuttle project start`
  - `cargo shuttle deploy`
  - を実行

# アーキテクチャ
特定のアーキテクチャを目指したものではない．現時点で自分ができる範囲でやっている．
俗にいう，クリーンアーキテクチャ（を謳う情報）からいくらか参考にしている．