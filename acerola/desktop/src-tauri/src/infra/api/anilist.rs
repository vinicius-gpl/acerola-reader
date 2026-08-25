use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnilistResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnilistSearchData {
    #[serde(rename = "Page")]
    pub page: AnilistPage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnilistPage {
    pub media: Vec<AnilistMedia>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistMedia {
    pub id: i32,
    pub title: AnilistTitle,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub staff: Option<AnilistStaff>,
    #[serde(default)]
    pub cover_image: Option<AnilistCoverImage>,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub average_score: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistTitle {
    #[serde(default)]
    pub romaji: Option<String>,
    #[serde(default)]
    pub english: Option<String>,
    #[serde(default)]
    pub user_preferred: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistCoverImage {
    #[serde(default)]
    pub large: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistStaff {
    #[serde(default)]
    pub edges: Vec<AnilistStaffEdge>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistStaffEdge {
    #[serde(default)]
    pub role: String,
    pub node: AnilistStaffNode,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistStaffNode {
    pub name: AnilistStaffName,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnilistStaffName {
    #[serde(default)]
    pub full: String,
}

pub struct AnilistClient {
    client: reqwest::Client,
}

impl AnilistClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("AcerolaMangaApp/1.0 (Acerola Desktop)")
                .build()
                .unwrap(),
        }
    }

    // TODO: Validar de usar post ao invez de um cliente graphql aqui não é melhor.
    pub async fn search_manga_by_title(
        &self, title: &str,
    ) -> Result<AnilistResponse<AnilistSearchData>, String> {
        let query = r#"
            query MediaSearch($search: String) {
              Page(page: 1, perPage: 1) {
                media(search: $search, type: MANGA, sort: POPULARITY_DESC) {
                  id
                  title {
                    userPreferred
                    romaji
                    english
                  }
                  description(asHtml: false)
                  status
                  staff(sort: RELEVANCE) {
                    edges {
                      role
                      node {
                        name {
                          full
                        }
                      }
                    }
                  }
                  coverImage {
                    large
                  }
                  bannerImage
                  averageScore
                }
              }
            }
        "#;

        let body = json!({
            "query": query,
            "variables": {
                "search": title
            }
        });

        let response = self
            .client
            .post("https://graphql.anilist.co")
            .json(&body)
            .send()
            .await
            .map_err(|error| format!("Request failed: {}", error))?;

        let status = response.status();
        let response_text =
            response.text().await.map_err(|error| format!("Failed to read response: {}", error))?;

        if !status.is_success() {
            return Err(format!("HTTP error {}: {}", status, response_text));
        }

        serde_json::from_str(&response_text)
            .map_err(|error| format!("JSON decode failed: {}. Response: {}", error, response_text))
    }
}
