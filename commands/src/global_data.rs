use anyhow::Error;

use poise::serenity_prelude::RwLock;

use std::sync::Arc;

use std::collections::{HashMap, HashSet};

use sled::Db;

use crate::other_server_repository::{SledOtherServerRepository, SledOtherTimesRepository};
use crate::own_server_repository::{SledOwnServerRepository, SledOwnTimesRepository};
use crate::sign::{keys_gen::RsaKeyGenerator, UbiquitimesPrivateKey, UbiquitimesPublicKey};

// Types used by all command functions
// すべてのコマンド関数で使用される型
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
pub struct Data {
    pub connection: Arc<Db>,
    pub master_webhook_url: RwLock<String>,
    // いちいちデータベースにアクセスするのは非効率なので
    // キャッシュのような役割として，ここに保持する
    pub own_server_cache: RwLock<Option<crate::own_server::OwnServer>>,
    // 秘密鍵はここにはのせない
    pub public_key_pem_hashmap: RwLock<HashMap<u64, String>>,
    // ユーザidと送信先ギルドid
    pub sent_member_and_guild_ids: RwLock<HashMap<u64, RwLock<HashSet<u64>>>>,
    // トレイトを受け取るようにしたいけど，うまくいかないから
    // 具体的な型を指定する
    // ていうか，sledは排他制御してるんだっけ
    // してなかったらMutexで囲む
    // データ保存にかかわるオブジェクト
    pub own_server_repository: Arc<SledOwnServerRepository>,
    pub own_times_repository: Arc<SledOwnTimesRepository>,
    pub other_server_repository: Arc<SledOtherServerRepository>,
    pub other_times_repository: Arc<SledOtherTimesRepository>,
    // 鍵に関わるオブジェクト
    pub ubiquitimes_signer: Arc<UbiquitimesPrivateKey>,
    pub ubiquitimes_verifier: Arc<UbiquitimesPublicKey>,
    pub ubiquitimes_keygenerator: Arc<RsaKeyGenerator>,
}
