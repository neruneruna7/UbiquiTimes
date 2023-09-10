# コンセプト
それぞれの団体，サーバで独立稼働するTimes拡散DiscordBot

# 必要になりそうなユーザコマンド
- UTshere その後に続くメッセージ内容を拡散する
- UTregtimes 実行したチャンネルを，そのユーザの拡散先として登録する
- UTunregtimes 実行したチャンネルを，そのユーザの拡散先から削除する
- UTlist 登録されている拡散先を表示する
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