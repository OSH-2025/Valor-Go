use flatbuffers::FlatBufferBuilder;
use crate::mgmtd_client::MgmtdClient;
use hostname;

pub struct FuseConfigFetcher;

impl FuseConfigFetcher {
    pub async fn complete_app_info<'a>(&self, app_info: &mut FlatBufferBuilder<'a>) -> Result<(), String> {
        let hostname = match hostname::get() {
            Ok(h) => h.to_string_lossy().into_owned(),
            Err(e) => return Err(e.to_string()),
        };

        let client = MgmtdClient::new();
        client.init().await?;

        let tags = client.get_universal_tags(&hostname, app_info).await?;
        app_info.finish(tags, None);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_app_info() {
        let fetcher = FuseConfigFetcher;
        let mut app_info = FlatBufferBuilder::new();
        assert!(fetcher.complete_app_info(&mut app_info).await.is_ok());
    }
}