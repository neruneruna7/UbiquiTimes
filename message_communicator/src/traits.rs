use std::collections::HashMap;

use domain::{
    models::{
        communication::{RequestMessage, ResponseMessage},
        guild_data::OtherGuild,
    },
    traits::communicators::{GuildName, HashKey},
};

pub(crate) trait MessageSender {
    type Result<T>;
    async fn send_request(
        &self,
        dst_guild: &OtherGuild,
        member_id: u64,
        req: RequestMessage,
        sent_member_and_guild_ids: &mut HashMap<HashKey, GuildName>,
    ) -> Self::Result<()>;

    async fn send_response(&self, dst_guild: &OtherGuild, res: ResponseMessage)
        -> Self::Result<()>;
}

pub(crate) trait MessageReceiver {
    type Result<T>;
    async fn receive_request(&self, req: RequestMessage, own_guild_id: u64) -> Self::Result<()>;
    async fn receive_response(&self, res: ResponseMessage) -> Self::Result<()>;
}
