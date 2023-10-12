use crate::*;


use crate::sign::key_gen::*;

use anyhow::Context as anyhowContext;
use anyhow::{anyhow, Result};

use rsa::pkcs8::der::zeroize::Zeroizing;



/// bot導入後，最初に実行してください
///
/// 自身のサーバのマスターwebhook，サーバ情報を登録します
/// 返信として，他のサーバが自身のサーバを拡散可能先として登録する際にコピペ可能なテキストを返します．
#[poise::command(prefix_command, track_edits, aliases("UtOwnServerData"), slash_command)]
pub async fn ut_set_own_masterhook(
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

    let register_tmplate_str = format!(
        "/ut_set_other_masterhook server_name:{} master_webhook_url:{} guild_id:{} public_key_pem:{}",
        server_name, master_webhook_url, guild_id, keys_pem.public_key_pem
    );
    // format!("server_data: ```\n server_name: {},\n guild_id: {},\n master_channel_id: {},\n master_webhook_url: {}```", server_name, guild_id, master_channel_id, master_webhook_url)

    ctx.say(register_tmplate_str).await?;

    loged(&ctx, "サーバ情報を登録しました").await?;

    Ok(())
}

/// trueなら鍵を作り直す
/// falseなら鍵をDBから取得する
async fn get_keys_pem(ctx: Context<'_>, is_new_key: bool) -> Result<KeyPair_pem> {
    if is_new_key {
        let (private_key, public_key) = generate_keypair();
        Ok(keypair_to_pem(&private_key, &public_key))
    } else {
        let server_data = crate::db_query::own_server_data::select_own_server_data(
            &ctx.data().connection,
            ctx.guild_id()
                .ok_or(anyhow!("guildidを取得できませんでした"))?
                .0,
        )
        .await
        .context("鍵を取得できません. trueを指定してください")?;
        Ok(KeyPair_pem {
            private_key_pem: Zeroizing::new(server_data.private_key_pem),
            public_key_pem: server_data.public_key_pem,
        })
    }
}
