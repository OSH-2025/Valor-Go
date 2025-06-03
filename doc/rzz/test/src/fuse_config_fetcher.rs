// use flatbuffers::FlatBufferBuilder;
// use crate::mgmtd_client::MgmtdClient;
// use hostname;

// pub struct FuseConfigFetcher;

// impl FuseConfigFetcher {
//     pub async fn complete_app_info<'a>(&self, app_info: &mut FlatBufferBuilder<'a>) -> Result<(), String> {
//         let hostname = match hostname::get() {
//             Ok(h) => h.to_string_lossy().into_owned(),
//             Err(e) => return Err(e.to_string()),
//         };

//         let client = MgmtdClient::new();
//         client.init().await?;

//         let tags = client.get_universal_tags(&hostname, app_info).await?;
//         app_info.finish(tags, None);

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_complete_app_info() {
//         let fetcher = FuseConfigFetcher;
//         let mut app_info = FlatBufferBuilder::new();
//         assert!(fetcher.complete_app_info(&mut app_info).await.is_ok());
//     }
// }

use crate::mgmtd_client::MgmtdClient;
use flatbuffers::FlatBufferBuilder;
use hostname;
use std::error::Error;

pub struct FuseConfigFetcher {
    mgmtd_client: MgmtdClient,
}

impl FuseConfigFetcher {
    pub fn new(mgmtd_service_url: &str) -> Self {
        FuseConfigFetcher {
            mgmtd_client: MgmtdClient::new(mgmtd_service_url),
        }
    }

    pub async fn complete_app_info<'a>(
        &self,
        app_info: &mut FlatBufferBuilder<'a>,
    ) -> Result<(), Box<dyn Error>> {
        // 1. 获取主机名
        let hostname = hostname::get()?
            .to_string_lossy()
            .into_owned();

        // 2. 初始化客户端
        self.mgmtd_client.init().await?;

        // 3. 获取标签并写入 FlatBuffer
        let tags = self.mgmtd_client.get_universal_tags(&hostname, app_info).await?;
        app_info.finish(tags, None);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock; // 用于模拟 HTTP 服务

    #[tokio::test]
    async fn test_complete_app_info() {
        // 1. 启动模拟服务器
        let mock_server = mockito::Server::new();
        let _m = mock("GET", "/api/v1/tags?hostname=test-host")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"tags": ["tag1", "tag2"]}"#)
            .create();

        // 2. 测试逻辑
        let fetcher = FuseConfigFetcher::new(&mock_server.url());
        let mut builder = FlatBufferBuilder::new();
        let result = fetcher.complete_app_info(&mut builder).await;

        // 3. 验证结果
        assert!(result.is_ok());
        let data = builder.finished_data();
        assert!(data.len() > 0);
    }
}