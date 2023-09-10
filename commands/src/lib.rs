use std::sync::Arc;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::*;

mod webhook;

// Dbのラッパー
struct UtDb;

// // TypemapKeyを実装することで、Contextに格納できるようになる
// impl TypeMapKey for UtDb {
//     type Value = Arc<sled::Db>;
// }

#[command]
pub async fn rg_webhook(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
