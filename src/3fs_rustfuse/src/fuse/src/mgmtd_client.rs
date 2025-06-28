// use flatbuffers::{FlatBufferBuilder, WIPOffset};

// pub struct MgmtdClient;

// impl MgmtdClient {
//     pub fn new() -> Self {
//         MgmtdClient
//     }

//     pub async fn init(&self) -> Result<(), String> {
//         // TODO: 实现初始化逻辑
//         Ok(())
//     }

//     pub async fn get_universal_tags<'a>(&self, _hostname: &str, builder: &mut FlatBufferBuilder<'a>) -> Result<WIPOffset<&'a str>, String> {
//         // TODO: 实现获取标签的逻辑
//         // 这里我们创建一个简单的字符串作为示例
//         let tags = builder.create_string("example_tags");
//         Ok(tags)
//     }
// } 

use reqwest::Client; // 用于 HTTP 请求
use serde::{Deserialize, Serialize};
use std::error::Error;
use flatbuffers::{FlatBufferBuilder, WIPOffset};

#[derive(Debug, Serialize, Deserialize)]
struct UniversalTagsResponse {
    tags: Vec<String>,
}

pub struct MgmtdClient {
    pub http_client: Client,
    pub mgmtd_service_url: String, // 管理服务的地址，例如 "http://mgmtd-service:8080"
}

impl MgmtdClient {
    pub fn new(mgmtd_service_url: &str) -> Self {
        MgmtdClient {
            http_client: Client::new(),
            mgmtd_service_url: mgmtd_service_url.to_string(),
        }
    }

    pub async fn init(&self) -> Result<(), Box<dyn Error>> {
        // 这里可以添加初始化逻辑（如健康检查）
        Ok(())
    }

    pub async fn get_universal_tags<'a>(
        &self,
        hostname: &str,
        builder: &mut FlatBufferBuilder<'a>,
    ) -> Result<WIPOffset<&'a str>, Box<dyn Error>> {
        // 1. 发送 HTTP 请求到管理服务
        let url = format!(
            "{}/api/v1/tags?hostname={}",
            self.mgmtd_service_url, hostname
        );
        let response = self
            .http_client
            .get(&url)
            .send()
            .await?
            .json::<UniversalTagsResponse>()
            .await?;

        // 2. 将标签列表序列化为字符串（用逗号分隔）
        let tags_str = response.tags.join(",");
        
        // 3. 写入 FlatBuffer
        let tags_offset = builder.create_string(&tags_str);
        Ok(tags_offset)
    }
}