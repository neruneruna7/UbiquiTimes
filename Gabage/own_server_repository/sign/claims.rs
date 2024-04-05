pub(crate) async fn register_public_key_ctx_data(
    guild_id: u64,
    public_key_pem: String,
    ctx: &Context<'_>,
) -> anyhow::Result<()> {
    let mut public_key_pem_hashmap = ctx.data().public_key_pem_hashmap.write().await;

    public_key_pem_hashmap.insert(guild_id, public_key_pem);
    Ok(())
}
