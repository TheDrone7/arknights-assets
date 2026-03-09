use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub res_version: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotUpdateList {
    pub pack_infos: Vec<PackInfo>,
    pub ab_infos: Vec<AbInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub name: String,
    pub total_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbInfo {
    pub name: String,
    pub total_size: u32,
    pub ab_size: u32,
    pub md5: String,
    pub pid: Option<String>,
}
