# コンセプト
それぞれの団体，サーバで独立稼働するTimes拡散DiscordBot

# 必要になりそうなユーザコマンド
- UTshere その後に続くメッセージ内容を拡散する
- UTregtimes 実行したチャンネルを，そのユーザの拡散先として登録する
- UTunregtimes 実行したチャンネルを，そのユーザの拡散先から削除する
- UTserverlist チャンネル登録可能なサーバを表示する
- UTsharelist 登録されている拡散先を表示する
- UTping そのユーザの拡散先に対して，宛先を指定して拡散が可能かどうかを確認する
- UTpingA そのユーザの拡散先に対して，すべてのサーバに拡散が可能かどうかを確認する（全サーバ）

# 必要になりそうな運営コマンド
- UTregserver 指定したサーバを，拡散可能なサーバとして登録する
- UTunregserver 指定したサーバを，拡散可能なサーバから削除する

# KVSに格納するためのキー名とバリューのデータ型
- "webhook_urls": Vec<String>
- "master_webhook_urls": Vec<HashMap<>>

# bot間通信
bot間の通信が必要になるので，どのように実装するか．  
特定のチャンネルを，bot間の通信用，ログ用チャンネルとして設定するのが良さそう．
`{src: "server", cmd_kind: "ログなのかbot間通信なのか", cmd: "通信コマンド"}`

そのチャンネルをそのサーバのbotが監視して，bot間通信を行う．
そのチャンネルのwebhookをマスターwebhookとして登録して，拡散可能サーバの登録にはこのwebhookを人力で登録する．


どのようにして指定のチャンネルを監視するか
イベントハンドラに`message`を登録して，`message.channel.id`で判定する．でいけそう

いけた

# bot間通信のデータ構造
jsonをメッセージに投げ，それをパースする．  

{
    "src": "server",
    "dst": "server",
    "cmd_kind": "log",
    "cmd": "log message"
    "ttl": 4
}

チャンネルへの書き込みを監視する都合，無限ループの発生を防ぐために，ttlを設定する．
# グローバルデータ
`client.data`にRwLockでアクセスするといけそう  
そもそも変数に書き込みしないから，`Arc<>`だけで十分だ


# メモ
サーバの識別に，webhookに紐づいたサーバ識別名を使う．
サーバ自身が設定して，他の拡散可能サーバとして登録する時に取得して設定される．

一気に双方向で登録したいな  
チャンネル登録時，両サーバが拡散可能サーバとして登録してあれば，メンバーが片方のサーバから拡散Timesを設定したときに，双方向の拡散設定をする．

~~ KVSで, Vecへのデータ追加は`marge`メソッドで行けそう  
削除処理は少々手間かも ~~
追加も削除も自前の関数を使う


- botのユーザid 1147398117717704714
- webhookを介したbotのユーザid 1148062413812404244


CREATE TABLE IF NOT EXISTS serverwebhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    servername    TEXT    UNIQUE      NOT NULL,
    guildid    TEXT          NOT NULL,
    webhookurl     TEXT                NOT NULL,
)


CREATE TABLE IF NOT EXISTS privatewebhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    servername    TEXT          NOT NULL,
    userid    TEXT          NOT NULL,
    webhookurl     TEXT                NOT NULL
)

# bot間通信の流れ
## マスターwebhookの登録
サーバA，サーバB
- AがBのマスターwebhookの登録を行う
- AはBにマスターwebhookの登録通知を行う
- Bはそれを受け取り，サーバー名とGUILD_IDを返送する
- Aはそれを受け取り，サーバー名とGUILD_IDを登録する

おなじ名前のサーバがあるとめんどいな
やめよう 手動で登録することにする

## メンバーwebhookの登録
サーバA，サーバB
- A拡散元チャンネルからコマンドを実行し，がBのチャンネルidを指定してを拡散先として設定する
- チャンネルidと，拡散元チャンネルのwebhookAをBのマスターwebhookに送信する
{a_member_webhook, b_channel_id}
- Bはそれを受け取り，チャンネルidからwebhookBを作成して返送する
- BはAを拡散可能登録してあれば，webhookAを，Bを拡散元として拡散先として登録する
- AはBにメンバーwebhookの登録通知を行う
    - botLogチャンネルへの通知，メンバーチャンネルへの通知
- もしも双方向にマスターwebhookが登録されていれば，Bから
Aへのメンバーwebhookの登録を行う
## 改
### マスターwebhookの登録
サーバA, サーバB 
- Aは，Bの`master_webhook`, `guild_id`, `servername`を登録する
- あとから偽装されていないことを人間が確認できるようにするためである


サーバA,自身のマスターwebhook作成
- `ut_set_masterwebhook_and_serverdata`コマンドで作成する
- いずれ運営のみが実行できるようにパスワード機能をつけたい
- `ut_set_masterwebhook`コマンドを実行すると，webhookの入力を要求し，webhookからchannel_idを作成し，それをマスターwebhookとして登録する
- 実行時に`servername`, `master_webhook_url`を入力し，登録する
- 実行結果として，`servername`, `guild_id`, `master_webhook_url`を返す

サーバ情報取得
- 実行結果として，`servername`, `guild_id`, `master_webhook_url`を返す

### メンバーwebhookの登録
両サーバが，互いに拡散可能サーバとして登録されている前提  
でない場合は手動登録で対応する

互いに登録してなくても，1方向で可能ではないか？ botが導入されている前提とはいえ．


サーバAは，サーバBのマスターwebhookを登録済みであるものとする．

最終的に必要なデータ
両者のwebhook
- それぞれのサーバで，どれが自分のTimesのチャンネルなのか登録する．その際に`webhook`は作成されている
- `webhook`は`member_id`に紐づいている
- 拡散可能サーバに登録されているすべてのサーバに，拡散登録リクエストを送る
- `{a_member_id, a_master_webhook, a_channel_id, a_servername, a_guild_id}`
- `a_channel_id`, `a_servername, a_guild_id`は，あとからaサーバが偽装されていないことを確認できるようにするため． 
- 拡散登録リクエストを受け取ったbサーバは，`a_member_id`と紐づいている`webhook`を拡散登録リプライする.
- 存在しなければ何も返送しない
- `{a_member_id, b_servername, b_guild_id, b_channel_id, b_webhook}`
- `b_servername`はbサーバ側で登録したbサーバのサーバ名
- aサーバで登録されているbサーバの名前とは違う可能性がある

- 同じ名前のサーバ，団体が存在する可能性もあるため，衝突を許容することにした.

- リプライを受け取ったaサーバは，受け取った`webhook`を登録する
- 


<!-- - Aチャンネルのwebhookは存在するか？
- 条件: webhookの名前が`UT-{userid}`
    - する それを取得
    - しない AチャンネルIDから作成
- `{A-webhook, B_channel_id, A-servername}`をB-マスターwebhookに送信
- BはA-webhookを拡散先に登録
- BはチャンネルIDからB-webhookを作成
- BはA-マスターwebhookにB-webhookを返送
- Aは受け取ったB-webhookを登録 -->

登録時のデータ
{
    b_servername,
    b_guild_id,
    b_member_webhook,
}

サーバーネームをどうするか

# 欠点
- マスターwebhookをなんらかの形で公開する必要がある
- 第三者がスパムを送り付けてくるかもしれない
- マスターwebhookはBot専用のチャンネルなので，被害は少なめになるだろうと予測しているとはいえ...

# 対策
- bot間通信の暗号化
- なんかlinuxで鍵を作成するやつあったな

### まずは手動登録を実装しよう
- 手動登録
- 削除
- 共有
- ping
- list




### 自身のサーバのマスターwebhookやbotComチャンネルが変更された場合の動き
`after`を使って，特定のコマンド実行されたときにDBから再取得する
む，この`after`にdb処理をまとめてもいいのでは？
すべてをまとめるのは現実的じゃないな
必要な処理はまとめよう

## マスターwebhook周りの処理作成
## channel_idとかがi64だとなんかうまくいかないので修正


# テストデータ
test

1150053746156507267

https://discord.com/api/webhooks/1150053774518403073/LEEaORLKsL2T0U9kMBjlsxhcmRvbKIHUPNncqdGfZ_ZmXThz9-aL0lZz0U-KrtK5XtmH


1150053692540723281

https://discord.com/api/webhooks/1151880824547975170/FR6T_U_ghGVXleL6buQeKIgBIsKJD9Xlr3SoxwK8kf-BDEV5ZQUpF-lpa_Pa9_s_dI7J

# 発生したgitエラー
object file .git/objects/ad/10d7e89c9a717582ff5c6dca6ef8582db494d2 is empty