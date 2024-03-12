use std::collections::HashMap;

use tracing::info;

use crate::models::{
    communication::{RequestMessage, ResponseMessage},
    guild_data::{OtherGuild, OwnGuild},
};

// できればpoiseへの依存がないトレイトを書きたい
pub trait UtReqSender {
    type Result<T>;
    async fn times_setting_request_send(
        &self,
        dst_guild: &OtherGuild,
        member_id: u64,
        req: RequestMessage,
        sent_member_and_guild_ids: &mut HashMap<HashKey, GuildName>,
    ) -> Self::Result<()>;

    /// どのサーバに対して送信したかを記録する
    /// リクエストコマンド時に入力した識別用サーバー名も記録する必要が出てきた
    /// デフォルト実装があるので，実装しなくてもよい
    async fn save_sent_guild_ids(
        sent_member_and_guild_ids: &mut HashMap<HashKey, GuildName>,
        member_id: u64,
        dst_guild_id: u64,
        dst_guild_name: String,
    ) {
        let hash_key = HashKey::new(member_id, dst_guild_id);

        info!(
            "hash_key: {:?}, server_name: {:?}",
            hash_key, dst_guild_name
        );

        // 一定時間後に削除するようにしたい

        sent_member_and_guild_ids.insert(hash_key, dst_guild_name);
    }
}

pub trait UtReqReceiver {
    type Result<T>;
    // 使うライブラリによって，アプリから受け取ったメッセージの型は違うはず
    // そのため，それだけは関連型を使って実装時に指定できるようにしておく
    type NewMessage;

    async fn times_setting_receive_and_response(
        &self,
        // リクエストを受け取って，それに対するレスポンスを返すため
        // リクエストを引数にとる
        new_message: Self::NewMessage,
        own_guild_id: u64,
    ) -> Self::Result<()>;
}

pub trait UtResReceiver {
    type Error;
    async fn times_setting_response_receive(&self, res: ResponseMessage)
        -> Result<(), Self::Error>;
}

/// ハッシュマップに送信記録を保存するとき，キーとして使うための構造体
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct HashKey {
    member_id: u64,
    guild_id: u64,
}

impl HashKey {
    fn new(member_id: u64, guild_id: u64) -> Self {
        Self {
            member_id,
            guild_id,
        }
    }
}

pub type GuildName = String;
