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
- Bはそれを受け取り，チャンネルidからwebhookBを作成して返送する
- BはAを拡散可能登録してあれば，webhookAを，Bを拡散元として拡散先として登録する
- AはBにメンバーwebhookの登録通知を行う
    - botLogチャンネルへの通知，メンバーチャンネルへの通知
- もしも双方向にマスターwebhookが登録されていれば，Bから
Aへのメンバーwebhookの登録を行う

### まずは手動登録を実装しよう


### 自身のサーバのマスターwebhookやbotComチャンネルが変更された場合の動き
`after`を使って，特定のコマンド実行されたときにDBから再取得する
む，この`after`にdb処理をまとめてもいいのでは？
すべてをまとめるのは現実的じゃないな
必要な処理はまとめよう
