use tokio::task::spawn_blocking;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub struct FuseConfigFetcher;

impl FuseConfigFetcher {
    pub async fn complete_app_info(&self, app_info: &mut flatbuffers::FlatBufferBuilder) -> Result<(), String> {
        let hostname = match sys_resource::hostname(true) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        let client = MgmtdClient::new();
        client.init().await?;

        let tags = client.get_universal_tags(&hostname).await?;
        app_info.finish(&tags, None);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_app_info() {
        let fetcher = FuseConfigFetcher;
        let mut app_info = flatbuffers::FlatBufferBuilder::new();
        assert!(fetcher.complete_app_info(&mut app_info).await.is_ok());
    }
}