// レスポンスを受け取って，それを処理する
// poiseのContextが使えないことに注意
// 代わりにpoiseのFrameworkContext構造体を使う

use super::{TimesSettingCommunicatorResult, UbiquitimesResReceiver};
use crate::{bot_message::ResponseMessage, bot_message_communicator::is_response_from_sent_guild};

pub struct WebhookResReceiver;

impl UbiquitimesResReceiver for WebhookResReceiver {
    async fn times_setting_response_receive(
        &self,
        framwework: poise::FrameworkContext<'_, crate::global_data::Data, anyhow::Error>,
        res: ResponseMessage,
    ) -> TimesSettingCommunicatorResult<()> {
        // 送信記録にあるサーバからのレスポンスかどうかを判定する
        // ない場合はエラーを返す
        let is_response_from_sent_guild = is_response_from_sent_guild(framwework, &res).await?;
        if !is_response_from_sent_guild {
            return Err(anyhow::anyhow!("This response is not from sent guild").into());
        }

        // レスポンスを処理
        // 相手のサーバを拡散先サーバとして登録

        todo!()
    }
}
