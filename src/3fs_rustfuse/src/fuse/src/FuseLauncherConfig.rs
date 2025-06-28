use serde::Serialize;

#[derive(Debug, Clone, serde::Deserialize, Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct IBDeviceConfig {
    pub config_details: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ClientConfig {
    pub config_details: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MgmtdClientForClientConfig {
    pub config_details: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FuseLauncherConfig {
    pub cluster_id: String,
    pub ib_devices: IBDeviceConfig,
    pub client: ClientConfig,
    pub mgmtd_client: MgmtdClientForClientConfig,
    pub mountpoint: String,
    pub allow_other: bool,
    pub token_file: String,
}

impl Default for FuseLauncherConfig {
    fn default() -> Self {
        Self {
            cluster_id: String::new(),
            ib_devices: IBDeviceConfig::default(),
            client: ClientConfig::default(),
            mgmtd_client: MgmtdClientForClientConfig::default(),
            mountpoint: String::new(),
            allow_other: true,
            token_file: String::new(),
        }
    }
}

impl FuseLauncherConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn init(&mut self, _file_path: &str, dump: bool, updates: &[KeyValue]) {
        for update in updates {
            match update.key.as_str() {
                "cluster_id" => self.cluster_id = update.value.clone(),
                "mountpoint" => self.mountpoint = update.value.clone(),
                "token_file" => self.token_file = update.value.clone(),
                "allow_other" => self.allow_other = update.value.parse().unwrap_or(self.allow_other),
                "ib_devices.config_details" => self.ib_devices.config_details = update.value.clone(),
                "client.config_details" => self.client.config_details = update.value.clone(),
                "mgmtd_client.config_details" => self.mgmtd_client.config_details = update.value.clone(),
                _ => println!("Unknown config key in FuseLauncherConfig::init: {}", update.key),
            }
        }

        if dump {
            println!("Dumping config (FuseLauncherConfig::init): {:#?}", self);
        }
    }
}