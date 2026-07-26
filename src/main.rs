use chrono::{Datelike, NaiveDate, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct GithubUser {
    public_repos: u32,
    followers: u32,
}

#[derive(Deserialize, Debug)]
struct GithubRepo {
    name: String,
    stargazers_count: u32,
    fork: bool,
}

#[derive(Deserialize, Debug)]
struct GraphQlResponse {
    data: Option<GraphQlData>,
}

#[derive(Deserialize, Debug)]
struct GraphQlData {
    user: Option<GraphQlUser>,
}

#[derive(Deserialize, Debug)]
struct GraphQlUser {
    #[serde(rename = "contributionsCollection")]
    contributions_collection: ContributionsCollection,
}

#[derive(Deserialize, Debug)]
struct ContributionsCollection {
    #[serde(rename = "totalCommitContributions")]
    total_commits: u32,
    #[serde(rename = "totalPullRequestContributions")]
    total_prs: u32,
}

struct ProfileConfig {
    username: &'static str,
    birth_date: NaiveDate,
    email: &'static str,
    telegram: &'static str,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            username: "ssnddsgnr",
            birth_date: NaiveDate::from_ymd_opt(1999, 8, 24).unwrap(),
            email: "me@chyng.one",
            telegram: "chyngalgan",
        }
    }
}

fn calculate_uptime(birth_date: NaiveDate) -> String {
    let now = Utc::now().date_naive();

    let mut years = now.year() - birth_date.year();
    let mut months = now.month() as i32 - birth_date.month() as i32;
    let mut days = now.day() as i32 - birth_date.day() as i32;

    if days < 0 {
        months -= 1;
        days += 30;
    }
    if months < 0 {
        years -= 1;
        months += 12;
    }

    format!("{} yrs, {:02} mos, {:02} days", years, months, days)
}

fn make_ascii_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProfileConfig::default();
    println!("🚀 Generating profile README for {}...", config.username);

    let token = std::env::var("PROFILE_UPDATE_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("rust-profile-generator"),
    );
    if !token.is_empty() {
        if let Ok(val) = HeaderValue::from_str(&format!("bearer {}", token)) {
            headers.insert(AUTHORIZATION, val);
        }
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // 1. Достём мою информацию:
    let user_url = format!("https://api.github.com/users/{}", config.username);
    let user_res = client.get(&user_url).send().await?;
    let (public_repos_count, followers_count) = if user_res.status().is_success() {
        let user_data: GithubUser = user_res.json().await?;
        (user_data.public_repos, user_data.followers)
    } else {
        (0, 0)
    };

    // 2. Читаем репозитории:
    let repos_url = if !token.is_empty() {
        "https://api.github.com/user/repos?per_page=100&type=owner".to_string()
    } else {
        format!(
            "https://api.github.com/users/{}/repos?per_page=100",
            config.username
        )
    };

    let repos_res = client.get(&repos_url).send().await?;
    let mut total_stars = 0;
    let mut lang_bytes_map: HashMap<String, u64> = HashMap::new();

    if repos_res.status().is_success() {
        let repos: Vec<GithubRepo> = repos_res.json().await?;
        for repo in repos {
            if !repo.fork {
                total_stars += repo.stargazers_count;
                let lang_url = format!(
                    "https://api.github.com/repos/{}/{}/languages",
                    config.username, repo.name
                );
                if let Ok(l_res) = client.get(&lang_url).send().await {
                    if l_res.status().is_success() {
                        if let Ok(bytes_data) = l_res.json::<HashMap<String, u64>>().await {
                            for (lang, bytes) in bytes_data {
                                if lang == "Makefile" {
                                    continue;
                                }
                                *lang_bytes_map.entry(lang).or_insert(0) += bytes;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Достаём метрики через GraphQL
    let mut total_commits = 0;
    let mut total_prs = 0;

    if !token.is_empty() {
        let query = serde_json::json!({
            "query": format!(
                r#"query {{ user(login: "{}") {{ contributionsCollection {{ totalCommitContributions totalPullRequestContributions }} }} }}"#,
                config.username
            )
        });
        if let Ok(gql_res) = client
            .post("https://api.github.com/graphql")
            .json(&query)
            .send()
            .await
        {
            if gql_res.status().is_success() {
                if let Ok(gql_data) = gql_res.json::<GraphQlResponse>().await {
                    if let Some(user) = gql_data.data.and_then(|d| d.user) {
                        total_commits = user.contributions_collection.total_commits;
                        total_prs = user.contributions_collection.total_prs;
                    }
                }
            }
        }
    }

    // 4. Собираем метрику:
    let m1_c1 = format!("   - Public Repositories: {}", public_repos_count);
    let m1_c2 = format!("Pull Requests: ..... {}", total_prs);
    let m1_c3 = format!("Total Commits: .... {}", total_commits);

    let m2_c1 = format!("   - Account Followers: . {}", followers_count);
    let m2_c2 = format!("Total Stars: ....... {}", total_stars);
    let m2_c3 = "Active Project: ... DataCopter".to_string();

    let metrics_line1 = format!("{: <43}{: <38}{}", m1_c1, m1_c2, m1_c3);
    let metrics_line2 = format!("{: <43}{: <38}{}", m2_c1, m2_c2, m2_c3);

    let mut metrics_block = format!(
        "1. Live GitHub Metrics -------------------------------------------------------------------------------------------------\n{}\n{}\n",
        metrics_line1, metrics_line2
    );

    let total_lang_bytes: u64 = lang_bytes_map.values().sum();
    let mut sorted_langs: Vec<(String, u64)> = lang_bytes_map.into_iter().collect();
    sorted_langs.sort_by(|a, b| b.1.cmp(&a.1));

    if total_lang_bytes > 0 {
        metrics_block.push_str("2. Top Languages (Code Volume) -----------------------------------------------------------------------------------------\n");
        for (lang, bytes) in sorted_langs.iter().take(5) {
            let percentage = (*bytes as f64 / total_lang_bytes as f64) * 100.0;
            let bar = make_ascii_bar(percentage, 70);
            metrics_block.push_str(&format!("   - {:<12} {} {:>5.1}%\n", lang, bar, percentage));
        }
    }

    // 5. Читаем заготовки, интегрируем возраст:
    let uptime = calculate_uptime(config.birth_date);

    let profile_template = std::fs::read_to_string("templates/profile.md")
        .unwrap_or_else(|_| include_str!("../templates/profile.md").to_string());
    let profile_content = profile_template.replace("{uptime}", &uptime);

    let wallets_content = std::fs::read_to_string("templates/wallets.md")
        .unwrap_or_else(|_| include_str!("../templates/wallets.md").to_string());

    // 6. Генерим финальный README.md:
    let readme_content = format!(
        r#"<pre>
{}
{}
{}
</pre>

<p align="center">
  <a href="https://t.me/{telegram}" target="_blank"><img src="https://img.shields.io/badge/Telegram-26A69A?style=for-the-badge&logo=telegram&logoColor=white"/></a>
  <a href="mailto:{email}"><img src="https://img.shields.io/badge/Email-EA4335?style=for-the-badge&logo=gmail&logoColor=white"/></a>
  <a href="https://github.com/{username}"><img src="https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white"/></a>
</p>
"#,
        profile_content.trim(),
        metrics_block.trim(),
        wallets_content.trim(),
        telegram = config.telegram,
        email = config.email,
        username = config.username,
    );

    let mut file = File::create("README.md")?;
    file.write_all(readme_content.as_bytes())?;

    println!("✅ README.md generated successfully!");
    Ok(())
}
