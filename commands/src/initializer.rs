// Bot起動時に実行される処理を記述するファイル
//
// やること：
// - DBに自分のサーバのデータがあれば，それをメモリ上に読み込む（アクセスしやすくするため）
//     - ウェルカムメッセージを表示する（応用）
// - なければ，データを登録するように促すメッセージを表示する

use crate::{global_data::Context, own_server_repository::OwnServerRepository};

use crate::own_server::OwnServer;

pub async fn get_or_init_own_server_data(
    ctx: &Context<'_>,
    _own_server_data: OwnServer,
) -> anyhow::Result<()> {
    let own_server_repository = ctx.data().own_server_repository.clone();
    let own_server_data = own_server_repository.get().await;

    // if let Ok(own_server_data) = own_server_data {
    //     let mut own_server_cache = ctx.data().own_server_cache.write().await;
    //     *own_server_cache = Some(own_server_data);
    // } else {
    //     // データの登録を促すメッセージを表示する
    // }

    Ok(())
}
