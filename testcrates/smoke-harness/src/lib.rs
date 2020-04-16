#[cfg(test)]
rye_harness::test_harness!();

#[cfg(test)]
mod tests {
    #[rye::test_main]
    async fn test_main(sess: &mut rye::Session<'_>) -> anyhow::Result<()> {
        sess.run().await?;
        Ok(())
    }

    #[rye::test]
    fn case1(_: &mut rye::Context<'_>) {}
}
