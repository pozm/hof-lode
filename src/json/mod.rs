use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "FreeCompany")]
    pub free_company: FreeCompany,
    #[serde(rename = "FreeCompanyMembers")]
    pub free_company_members: Vec<FreeCompanyMember>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeCompany {
    #[serde(rename = "Active")]
    pub active: String,
    #[serde(rename = "ActiveMemberCount")]
    pub active_member_count: i64,
    #[serde(rename = "Crest")]
    pub crest: Vec<String>,
    #[serde(rename = "DC")]
    pub dc: String,
    #[serde(rename = "Estate")]
    pub estate: Estate,
    #[serde(rename = "Focus")]
    pub focus: Vec<Focu>,
    #[serde(rename = "Formed")]
    pub formed: i64,
    #[serde(rename = "GrandCompany")]
    pub grand_company: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ParseDate")]
    pub parse_date: i64,
    #[serde(rename = "Rank")]
    pub rank: i64,
    #[serde(rename = "Ranking")]
    pub ranking: Ranking,
    #[serde(rename = "Recruitment")]
    pub recruitment: String,
    #[serde(rename = "Reputation")]
    pub reputation: Vec<Reputation>,
    #[serde(rename = "Seeking")]
    pub seeking: Vec<Seeking>,
    #[serde(rename = "Server")]
    pub server: String,
    #[serde(rename = "Slogan")]
    pub slogan: String,
    #[serde(rename = "Tag")]
    pub tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estate {
    #[serde(rename = "Greeting")]
    pub greeting: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Plot")]
    pub plot: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Focu {
    #[serde(rename = "Icon")]
    pub icon: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ranking {
    #[serde(rename = "Monthly")]
    pub monthly: i64,
    #[serde(rename = "Weekly")]
    pub weekly: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reputation {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Progress")]
    pub progress: i64,
    #[serde(rename = "Rank")]
    pub rank: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seeking {
    #[serde(rename = "Icon")]
    pub icon: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeCompanyMember {
    #[serde(rename = "Avatar")]
    pub avatar: String,
    #[serde(rename = "FeastMatches")]
    pub feast_matches: i64,
    #[serde(rename = "ID")]
    pub id: i64,
    #[serde(rename = "Lang")]
    pub lang: Value,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Rank")]
    pub rank: String,
    #[serde(rename = "RankIcon")]
    pub rank_icon: String,
    #[serde(rename = "Server")]
    pub server: String,
}
