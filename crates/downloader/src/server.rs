#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Server {
    En,
    Cn,
    Bl,
    Jp,
    Kr,
    Tw,
}

impl Server {
    pub fn as_str(&self) -> &'static str {
        match self {
            Server::En => "EN (Yostar)",
            Server::Cn => "CN (Hypergryph)",
            Server::Bl => "CN (Bilibili)",
            Server::Jp => "JP (Yostar)",
            Server::Kr => "KR (Yostar)",
            Server::Tw => "TW (Hypergryph)",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Server::En => "en",
            Server::Cn => "cn",
            Server::Bl => "bl",
            Server::Jp => "jp",
            Server::Kr => "kr",
            Server::Tw => "tw",
        }
    }

    pub fn cdn_base_url(&self) -> &'static str {
        match self {
            Server::En => "https://ark-us-static-online.yo-star.com/assetbundle",
            Server::Cn | Server::Bl => "https://ak.hycdn.cn/assetbundle",
            Server::Jp => "https://ark-jp-static-online.yo-star.com/assetbundle",
            Server::Kr => "https://ark-kr-static-online-1300509597.yo-star.com/assetbundle",
            Server::Tw => "https://ak-tw.hg-cdn.com/assetbundle",
        }
    }

    pub fn version_url(&self) -> &'static str {
        match self {
            Server::En => {
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version"
            }
            Server::Cn => "https://ak-conf.hypergryph.com/config/prod/official/Android/version",
            Server::Bl => "https://ak-conf.hypergryph.com/config/prod/b/Android/version",
            Server::Tw => "https://ak-conf-tw.gryphline.com/config/prod/official/Android/version",
            Server::Jp => {
                "https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/version"
            }
            Server::Kr => {
                "https://ark-kr-static-online-1300509597.yo-star.com/assetbundle/official/Android/version"
            }
        }
    }

    pub fn asset_tag(&self) -> &'static str {
        match self {
            Server::Bl => "bilibili",
            _ => "official",
        }
    }

    pub fn asset_url(&self, res_version: &str, file_name: &str) -> String {
        format!(
            "{}/{}/Android/assets/{}/{}",
            self.cdn_base_url(),
            self.asset_tag(),
            res_version,
            file_name
        )
    }

    pub fn hot_update_url(&self, res_version: &str) -> String {
        self.asset_url(res_version, "hot_update_list.json")
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
