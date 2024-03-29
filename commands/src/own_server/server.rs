use crate::*;

use crate::own_server::OwnServerData;
use crate::sign::keys_gen::*;

use anyhow::Context as anyhowContext;
use anyhow::{anyhow, Result};

use rsa::pkcs8::der::zeroize::Zeroizing;

///他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返す関数．
fn get_register_tmplate_str(
    server_name: &str,
    master_webhook_url: &str,
    guild_id: u64,
    public_key_pem: &str,
) -> String {
    format!(
        "/ut_set_other_server_data server_name:{} master_webhook_url:{} guild_id:{} public_key_pem:{}",
        server_name, master_webhook_url, guild_id, public_key_pem
    )
}

/// bot導入後，最初に実行してください
///
/// 自身のサーバのマスターwebhook，サーバ情報を登録します
/// 返信として，他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返します．
/// デバッグビルドだと鍵作成の処理時間が長いため，返信が来ないことがあります
// おそらくどこかに時間を設定するところがあるはず...
#[poise::command(prefix_command, track_edits, aliases("UtOwnServerData"), slash_command)]
pub async fn ut_set_own_server_data(
    ctx: Context<'_>,
    #[description = "本サーバのサーバ名"] server_name: String,
    #[description = "本サーバのマスターwebhook URL"] master_webhook_url: String,
    #[description = "署名鍵を作り直しますか？（初回はTrueにしてください）"] is_new_key: bool,
) -> Result<()> {
    let master_webhook = Webhook::from_url(ctx, &master_webhook_url).await?;
    let master_channel_id = master_webhook
        .channel_id
        .ok_or(anyhow!("webhookからチャンネルidを取得できませんでした"))?
        .0;

    let guild_id = ctx
        .guild_id()
        .ok_or(anyhow!("guild_idが取得できませんでした"))?
        .0;

    let keys_pem = get_keys_pem(ctx, is_new_key).await?;

    let server_data = OwnServerData::new(
        guild_id,
        &server_name,
        master_channel_id,
        &master_webhook_url,
        &keys_pem.private_key_pem,
        &keys_pem.public_key_pem,
    );

    upsert_own_server_data(&ctx, server_data).await?;

    let register_tmplate_str = get_register_tmplate_str(
        &server_name,
        &master_webhook_url,
        guild_id,
        &keys_pem.public_key_pem,
    );
    // format!("server_data: ```\n server_name: {},\n guild_id: {},\n master_channel_id: {},\n master_webhook_url: {}```", server_name, guild_id, master_channel_id, master_webhook_url)

    ctx.say(register_tmplate_str).await?;

    logged(&ctx, "サーバ情報を登録しました").await?;

    Ok(())
}

/// このサーバの情報を返します
///
/// 他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返します．
#[poise::command(
    prefix_command,
    track_edits,
    aliases("UtGetOwnServerData"),
    slash_command
)]
pub async fn ut_get_own_server_data(ctx: Context<'_>) -> Result<()> {
    // dbから取得

    let db = ctx.data().connection.clone();
    let own_server_data = OwnServerData::db_read(db.as_ref())?.context("鍵を取得できません")?;

    // テンプレート文字列を作成
    let register_tmplate_str = get_register_tmplate_str(
        &own_server_data.server_name,
        &own_server_data.master_webhook_url,
        own_server_data.guild_id,
        &own_server_data.public_key_pem,
    );

    // 返信
    ctx.say(register_tmplate_str).await?;
    Ok(())
}

/// trueなら鍵を作り直す
/// falseなら鍵をDBから取得する
async fn get_keys_pem(ctx: Context<'_>, is_new_key: bool) -> Result<KeyPairPem> {
    if is_new_key {
        let (private_key, public_key) = generate_keypair();
        Ok(keypair_to_pem(&private_key, &public_key))
    } else {
        let db = ctx.data().connection.clone();
        let own_server_data = OwnServerData::db_read(db.as_ref())?
            .context("鍵を取得できません. trueを指定してください")?;

        Ok(KeyPairPem {
            private_key_pem: Zeroizing::new(own_server_data.private_key_pem),
            public_key_pem: own_server_data.public_key_pem,
        })
    }
}
