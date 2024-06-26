use anyhow::Error;

use ca_driver::my_ca_driver::MyCaDriver;
use domain::traits::communicators::{GuildName, HashKey};
use signer_verifier::key_generator::RsaKeyGenerator;
use sled_repository::other_times_repository::SledOtherTimesRepository;
use sled_repository::own_guild_repository::SledOwnGuildRepository;
use sled_repository::own_times_repository::SledOwnTimesRepository;

use std::sync::{Arc, Mutex};

use std::collections::HashMap;

// Types used by all command functions
// すべてのコマンド関数で使用される型
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
#[derive(Debug)]
pub struct Data {
    // ここにキャッシュと明示したデータを作るのはおかしいと思ったので，それらを削除

    // ユーザidと送信先ギルドid
    pub sent_member_and_guild_ids: Arc<Mutex<HashMap<HashKey, GuildName>>>,
    // トレイトを受け取るようにしたいけど，うまくいかないから
    // 具体的な型を指定する
    // ていうか，sledは排他制御してるんだっけ
    // してなかったらMutexで囲む
    // データ保存にかかわるオブジェクト
    pub own_server_repository: Arc<SledOwnGuildRepository>,
    pub own_times_repository: Arc<SledOwnTimesRepository>,
    pub other_times_repository: Arc<SledOtherTimesRepository>,
    // 鍵に関わるオブジェクト
    pub ubiquitimes_keygenerator: Arc<RsaKeyGenerator>,

    pub ca_driver: Arc<MyCaDriver>,
}
