// レスポンスを受け取って，それを処理する
// poiseのContextが使えないことに注意
// 代わりにpoiseのFrameworkContext構造体を使う

use super::{TimesSettingCommunicatorResult, UbiquitimesResReceiver};
use crate::{
    bot_message::ResponseMessage, bot_message_communicator::is_response_from_sent_guild,
    other_server::OtherTimes, other_server_repository::OtherTimesRepository,
};

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
        let guild_name = match is_response_from_sent_guild {
            Some(guild_name) => guild_name,
            None => {
                return Err(
                    anyhow::anyhow!("レスポンスを受け取ったサーバは送信記録にありません").into(),
                )
            }
        };

        // レスポンスを処理
        // 相手のサーバを拡散先サーバとして登録
        // または，拡散先サーバの情報を更新

        // OtherTimes構造体を作成
        let other_times = OtherTimes::new(
            res.times_setting_response.req_src_member_id,
            &guild_name,
            res.times_setting_response.req_dst_guild_id,
            res.times_setting_response.req_dst_channel_id,
            &res.times_setting_response.req_dst_webhook_url,
        );

        // DBに登録
        let other_times_repository = framwework.user_data.other_times_repository.clone();
        other_times_repository.upsert(other_times).await?;

        Ok(())
    }
}
