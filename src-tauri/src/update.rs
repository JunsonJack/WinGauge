//! AtomGit 检查更新（手动）。
//!
//! API：`GET https://api.atomgit.com/api/v5/repos/:owner/:repo/releases/latest`
//! 文档：https://docs.atomgit.com/docs/apis/get-api-v-5-repos-owner-repo-releases-latest
//!
//! 仅比对版本并给出下载链接；安装由用户手动完成（打开浏览器下载 NSIS/绿色包）。

use serde::{Deserialize, Serialize};

/// AtomGit 空间与仓库（与 GitHub 同名；发布 Release 时请同步到 AtomGit）
pub const ATOMGIT_OWNER: &str = "JunsonJack";
pub const ATOMGIT_REPO: &str = "WinGauge";

const USER_AGENT: &str = concat!("WinGauge/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    /// 当前应用版本
    pub current_version: String,
    /// 远端最新 tag（去 v 前缀后的展示名）
    pub latest_version: Option<String>,
    /// 是否有更新
    pub update_available: bool,
    /// Release 标题
    pub release_name: Option<String>,
    /// Release 说明（可能很长）
    pub release_notes: Option<String>,
    /// Release 页面
    pub release_url: Option<String>,
    /// 优先下载地址（setup.exe / portable zip）
    pub download_url: Option<String>,
    /// 附件文件名
    pub download_name: Option<String>,
    /// 人类可读状态
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct AtomAsset {
    #[serde(default)]
    name: String,
    #[serde(default)]
    browser_download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AtomRelease {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<AtomAsset>,
}

/// `v0.1.0` / `V0.1.0` / `0.1.0` → `(0,1,0)`
pub fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let s = raw.trim().trim_start_matches(['v', 'V']);
    let core = s.split(['-', '+', ' ']).next().unwrap_or("");
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);
    Some((major, minor, patch))
}

pub fn is_newer(remote: &str, current: &str) -> bool {
    let (Some(r), Some(c)) = (parse_version(remote), parse_version(current)) else {
        return false;
    };
    r > c
}

fn latest_api_url() -> String {
    format!(
        "https://api.atomgit.com/api/v5/repos/{}/{}/releases/latest",
        ATOMGIT_OWNER, ATOMGIT_REPO
    )
}

fn release_page_url(tag: &str) -> String {
    format!(
        "https://www.atomgit.com/{}/{}/releases/tag/{}",
        ATOMGIT_OWNER, ATOMGIT_REPO, tag
    )
}

/// 在附件里优先挑安装包，其次绿色 zip
fn pick_download(assets: &[AtomAsset]) -> Option<(String, String)> {
    let mut setup = None;
    let mut portable = None;
    let mut any = None;
    for a in assets {
        let Some(url) = a.browser_download_url.as_ref().filter(|u| !u.is_empty()) else {
            continue;
        };
        let lower = a.name.to_ascii_lowercase();
        if lower.contains("setup") && lower.ends_with(".exe") {
            setup = Some((url.clone(), a.name.clone()));
        } else if lower.contains("portable") || lower.ends_with(".zip") {
            portable = Some((url.clone(), a.name.clone()));
        } else if any.is_none() {
            any = Some((url.clone(), a.name.clone()));
        }
    }
    setup.or(portable).or(any)
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

/// 拉取 AtomGit 最新 Release 并与当前版本比较。
pub async fn check_update(current_version: String) -> UpdateCheckResult {
    let current = current_version.trim_start_matches(['v', 'V']).to_string();
    let mut result = UpdateCheckResult {
        current_version: current.clone(),
        latest_version: None,
        update_available: false,
        release_name: None,
        release_notes: None,
        release_url: None,
        download_url: None,
        download_name: None,
        message: String::new(),
    };

    let http = match client() {
        Ok(c) => c,
        Err(e) => {
            result.message = format!("无法初始化网络客户端：{e}");
            return result;
        }
    };

    let resp = match http.get(latest_api_url()).send().await {
        Ok(r) => r,
        Err(e) => {
            result.message = format!("检查更新失败（网络）：{e}");
            return result;
        }
    };

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        result.message = format!(
            "AtomGit 上暂无 {ATOMGIT_OWNER}/{ATOMGIT_REPO} 的 Release，请先发布后再检查"
        );
        return result;
    }
    if !resp.status().is_success() {
        result.message = format!("检查更新失败：HTTP {}", resp.status());
        return result;
    }

    let release: AtomRelease = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            result.message = format!("解析 Release 失败：{e}");
            return result;
        }
    };

    let tag = release.tag_name.trim().to_string();
    let latest = tag.trim_start_matches(['v', 'V']).to_string();
    result.latest_version = Some(latest.clone());
    if !release.name.trim().is_empty() {
        result.release_name = Some(release.name.clone());
    }
    if !release.body.trim().is_empty() {
        result.release_notes = Some(release.body.clone());
    }
    result.release_url = Some(
        release
            .html_url
            .clone()
            .filter(|u| u.starts_with("http"))
            .unwrap_or_else(|| release_page_url(&tag)),
    );

    if release.prerelease {
        result.message = format!("最新为预发布 {tag}，未与稳定版比较");
        return result;
    }

    result.update_available = is_newer(&tag, &current);
    if let Some((url, name)) = pick_download(&release.assets) {
        result.download_url = Some(url);
        result.download_name = Some(name);
    }

    result.message = if result.update_available {
        format!("发现新版本 {tag}（当前 {current}）")
    } else {
        format!("已是最新（{current}）")
    };
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_versions() {
        assert_eq!(parse_version("v0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("V0.2.1-beta"), Some((0, 2, 1)));
        assert_eq!(parse_version("1.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("nope"), None);
    }

    #[test]
    fn newer_compare() {
        assert!(is_newer("v0.1.1", "0.1.0"));
        assert!(is_newer("0.2.0", "v0.1.9"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn pick_prefers_setup() {
        let assets = vec![
            AtomAsset {
                name: "a.zip".into(),
                browser_download_url: Some("https://x/a.zip".into()),
            },
            AtomAsset {
                name: "WinGauge_0.1.1_x64-setup.exe".into(),
                browser_download_url: Some("https://x/setup.exe".into()),
            },
        ];
        let (url, name) = pick_download(&assets).unwrap();
        assert!(name.contains("setup"));
        assert!(url.ends_with("setup.exe"));
    }
}
