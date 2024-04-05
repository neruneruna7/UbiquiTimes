use super::*;

// イベントハンドラ
// serenityの，EventHadlerトレイトを実装して実現していたものと同等と推測
pub async fn event_handler(
    _ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        FullEvent::Message { new_message } => {
            info!("new message: {:?}", new_message);

            let ca_driver = _data.ca_driver.clone();
            let own_times_repository = _data.own_times_repository.clone();
            let own_guild_id = new_message.guild_id.unwrap().get();

            let req_receiver = PoiseWebhookReqReceiver::new(ca_driver, own_times_repository);
            req_receiver
                .times_setting_receive_and_response(new_message, own_guild_id)
                .await?;

            let other_times_repository = _data.other_times_repository.clone();
            let sent_member_and_guild_ids = _data.sent_member_and_guild_ids.clone();

            let res_receiver = PoiseWebhookResReceiver::new(other_times_repository);
            res_receiver
                .times_setting_response_receive(new_message, sent_member_and_guild_ids)
                .await?;
        }
        _ => {}
    }
    Ok(())
}
