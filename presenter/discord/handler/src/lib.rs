use anyhow::Error;

use domain::traits::communicators::*;
use message_communicator::request_receiver::PoiseWebhookReqReceiver;
use message_communicator::response_receiver::PoiseWebhookResReceiver;
use poise::serenity_prelude::{self as serenity, FullEvent};

use commands::global_data::Data;

use domain::tracing::info;

pub mod error_handler;
pub mod event_handler;

pub use error_handler::on_error;
pub use event_handler::event_handler;

/// poise公式リポジトリのサンプルコードの改造
/// コメントをグーグル翻訳にかけている

// Types used by all command functions
// すべてのコマンド関数で使用される型
// type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
