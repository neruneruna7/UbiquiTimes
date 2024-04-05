use crate::global_data::Context;
use anyhow::Result;

pub mod setting_commands;
pub mod spreading_commands;

async fn command_check(ctx: &Context<'_>, enter_str: &str, sign_str: &str) -> Result<()> {
    let err_text = format!("{}と入力してください", sign_str);
    if enter_str != sign_str {
        ctx.say(&err_text).await?;
        return Err(anyhow::anyhow!(err_text));
    }

    Ok(())
}
