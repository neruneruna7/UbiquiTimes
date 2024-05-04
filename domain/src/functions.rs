/// Webhook名に，UbiquiTimesが作成したものだとわかるようにプレフィクスをつける
pub fn add_prefix_memberid(member_id: u64) -> String {
    format!("UT-{}", member_id)
}

/// UbiquiTimesが送信したものだとわかるように，プレフィクスを付加
pub fn add_prefix_username(username: &str) -> String {
    format!("UT-{}", username)
}
