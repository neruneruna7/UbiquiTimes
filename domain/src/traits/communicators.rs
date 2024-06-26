use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tracing::info;

use crate::models::{
    communication::{ResponseMessage, TimesSettingRequest},
    guild_data::OwnGuild,
};

// できればpoiseへの依存がないトレイトを書きたい
pub trait UtReqSender {
    type Result<T>;
    fn times_setting_request_send(
        &self,
        own_guild: &OwnGuild,
        dst_guild_id: u64,
        dst_guild_name: &str,
        member_id: u64,
        times_setting_req: TimesSettingRequest,
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    ) -> impl std::future::Future<Output = Self::Result<()>> + Send;

    /// どのサーバに対して送信したかを記録する
    /// リクエストコマンド時に入力した識別用サーバー名も記録する必要が出てきた
    /// デフォルト実装があるので，実装しなくてもよい
    fn save_sent_guild_ids(
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
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

        let mut sent_member_and_guild_ids_lock = sent_member_and_guild_ids.lock().unwrap();
        sent_member_and_guild_ids_lock.insert(hash_key, dst_guild_name);
    }
}

pub trait UtReqReceiver {
    type Result<T>;
    // 使うライブラリによって，アプリから受け取ったメッセージの型は違うはず
    // そのため，それだけは関連型を使って実装時に指定できるようにしておく
    type NewMessage;

    fn times_setting_receive_and_response(
        &self,
        new_message: &Self::NewMessage,
        own_guild_id: u64,
    ) -> impl std::future::Future<Output = Self::Result<()>> + Send;
}

pub trait UtResReceiver {
    // Result型だとうまくいかなかった．ただ名前がResultなだけで，std::result::Resultとは関係ないので，
    // デフォルト実装の書くときにうまくいかないか，できても複雑になること思われる
    type Error;
    // 使うライブラリによって，アプリから受け取ったメッセージの型は違うはず
    // そのため，それだけは関連型を使って実装時に指定できるようにしておく
    type NewMessage;
    fn times_setting_response_receive(
        &self,
        new_message: &Self::NewMessage,
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// サーバからのレスポンスに対してリクエスト送信記録があるかどうか
    /// 返ってくるStringはサーバ名
    fn is_response_from_sent_guild(
        sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
        res: &ResponseMessage,
    ) -> Result<Option<String>, Self::Error> {
        let member_id = res.times_setting_response.req_src_member_id;
        let guild_id = res.dst_guild_id;

        let hash_key = HashKey::new(member_id, guild_id);

        let mut sent_member_and_guild_ids_lock = sent_member_and_guild_ids.lock().unwrap();

        // 該当データを取得
        let sent_guild_name = sent_member_and_guild_ids_lock.remove(&hash_key);

        info!(
            "hash_key: {:?}, server_name: {:?}",
            hash_key, sent_guild_name
        );

        Ok(sent_guild_name)
    }
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
