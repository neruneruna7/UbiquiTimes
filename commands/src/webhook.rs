use crate::*;


use std::collections::HashMap;

const WEBHOOKS_KEY: &str = "webhook_urls";

// 関数名はset_master_webhookから変更する必要がある
#[command]
async fn set_master_webhook(ctx: &Context, msg: &Message) -> CommandResult {
    let args = msg.content.split_whitespace().collect::<Vec<&str>>();
    if args.len() != 3 {
        msg.reply(
            ctx,
            "引数が足りません. サーバ識別名, webhookURLを入力してください",
        )
        .await?;
        return Ok(());
    }

    let server_name = args[1].to_string();
    let webhook_url = args[2].to_string();

    let server = {
        let mut server = HashMap::new();
        server.insert(server_name, webhook_url);
        server
    };

    // let data_read = ctx.data.read().await;
    // let db = data_read
    //     .get::<UtDb>()
    //     .expect("Expect UtDb in typemap")
    //     .clone();

    // db.ez_set(WEBHOOKS_KEY, &server)?;

    Ok(())
}
