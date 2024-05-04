use anyhow::Error;

use crate::{
    help_command::help,
    poise_commands::{
        setting_commands::{member_setting_commands, server_setting_commands},
        spreading_commands,
    },
};

use self::global_data::Data;

use super::*;

// use commands::help_command::help;
// use commands::poise_commands::setting_commands::{
//     member_setting_commands, server_setting_commands,
// };

// Shuttleとセルフホストの両方で使えるようにするため，切り出している
pub fn commands_vec() -> Vec<poise::Command<Data, Error>> {
    vec![
        help(),
        server_setting_commands::ut_initialize(),
        server_setting_commands::ut_get_own_server_data(),
        member_setting_commands::ut_times_set(),
        member_setting_commands::ut_times_show(),
        member_setting_commands::ut_times_unset(),
        member_setting_commands::ut_times_spread_setting(),
        member_setting_commands::ut_list(),
        member_setting_commands::ut_times_spread_unset(),
        spreading_commands::ut_times_release(),
        spreading_commands::hello(),
    ]
}
