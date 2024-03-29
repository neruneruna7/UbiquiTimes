use anyhow::Error;

use tokio::sync::RwLock;

use std::sync::Arc;

use std::collections::HashMap;

use crate::bot_message_communicator::HashKey;
use crate::other_server_repository::{SledOtherServerRepository, SledOtherTimesRepository};
use crate::own_server_repository::{SledOwnServerRepository, SledOwnTimesRepository};
use crate::sign::keys_gen::RsaKeyGenerator;

// Types used by all command functions
// すべてのコマンド関数で使用される型
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
// すべてのコマンド関数に渡されるカスタム ユーザー データ
#[derive(Debug)]
pub struct Data {
    // ここにキャッシュと明示したデータを作るのはおかしいと思ったので，それらを削除

    // ユーザidと送信先ギルドid
    pub sent_member_and_guild_ids: RwLock<HashMap<HashKey, String>>,
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
    pub ubiquitimes_keygenerator: Arc<RsaKeyGenerator>,
}
