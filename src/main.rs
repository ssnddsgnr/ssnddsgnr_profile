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
    private: bool,
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
    btc_wallet: &'static str,
    eth_wallet: &'static str,
    sol_wallet: &'static str,
    usdt_wallet: &'static str,
    usdc_wallet: &'static str,
    gram_wallet: &'static str,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            username: "ssnddsgnr",
            birth_date: NaiveDate::from_ymd_opt(1999, 8, 24).unwrap(),
            email: "me@chyng.one",
            telegram: "chyngalgan",
            btc_wallet: "bc1qu4edhpk0wsdzh90aa6f488sr2ylr0rfrn37gt6",
            eth_wallet: "0x5B17758d7Eb8e14119f5Bdf6c2bBD4de786c5d08",
            sol_wallet: "HBNsS5XnFFWr7Tye5mGKC7nRUUn1kVCqmEjGtte55G23",
            usdt_wallet: "0x5B17758d7Eb8e14119f5Bdf6c2bBD4de786c5d08",
            usdc_wallet: "0x5B17758d7Eb8e14119f5Bdf6c2bBD4de786c5d08",
            gram_wallet: "UQCA8hNAxXSNf7M1rWbY4TUXWS6nDdZjNIY6zraubPSo9EYu",
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

    // 1. Публичное кол-во репозиториев и фолловеров
    let user_url = format!("https://api.github.com/users/{}", config.username);
    let user_res = client.get(&user_url).send().await?;
    let (public_repos_count, followers_count) = if user_res.status().is_success() {
        let user_data: GithubUser = user_res.json().await?;
        (user_data.public_repos, user_data.followers)
    } else {
        (0, 0)
    };

    // 2. Получаем ВСЕ репозитории (включая Private, если есть токен)
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
                                // 🚫 ИСКЛЮЧЕНИЕ: Игнорируем Makefile
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

    // 3. Сбор суммарных коммитов и PR через GraphQL
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

    // 4. Формирование динамического топа языков
    let total_lang_bytes: u64 = lang_bytes_map.values().sum();
    let mut sorted_langs: Vec<(String, u64)> = lang_bytes_map.into_iter().collect();
    sorted_langs.sort_by(|a, b| b.1.cmp(&a.1));

    let mut languages_block = String::new();
    if total_lang_bytes > 0 {
        languages_block.push_str(
            "3. Top Languages (Code Volume) -------------------------------------------------\n",
        );
        // Берём Топ-5 языков (автоматически подтянутся любые новые)
        for (lang, bytes) in sorted_langs.iter().take(5) {
            let percentage = (*bytes as f64 / total_lang_bytes as f64) * 100.0;
            let bar = make_ascii_bar(percentage, 22);
            let kb = *bytes as f64 / 1024.0;
            languages_block.push_str(&format!(
                "   - {:<12} {:>7.1} KB  {}  {:>5.1}%\n",
                lang, kb, bar, percentage
            ));
        }
    }

    let uptime = calculate_uptime(config.birth_date);

    // ASCII Portrait (Exact 40-char width, 23 lines)
    let ascii_portrait: Vec<&str> = vec![
        "ssnddsgnrkkkkkkkkkkk=7mkkkkkkkkkkkkkkkkk",
        "kkkkkkkkkkm567.2..........''4kchyngalgan",
        "kkkkkkkkkb'...[-........-.....-8kneovimk",
        "kkkkkm''.....'b...............-,,,mmkkkk",
        "kkkb,...,,,.........,......'...7k8k5kkkk",
        "kkk75'2'..,mkkktundukkkkmm,....-,,,.mkkk",
        "kb,,.aw'.seckachkkStalkerkk-k.1..jk73kkk",
        "kkm'2.'.kkkkkkkkkkkkkkkkkk6kkkkm.0k.kkkk",
        "kkkb.../k-......'4kkkk'.......'=k...kkkk",
        "kkkk,lb...........9km...........m.,2kkmk",
        "kkkb'kkkk.........damn.........mk.w.[kkk",
        "kkkmwb}bkl$=---mk.kkmmkkkdmmmmkkkk.bkkkk",
        "kkkkbm,4p.'kkkkk,..8kmkmkshitkkkbmmkkkkk",
        "kkkkkm1k4,..7=..4m==64AudiRS6kkmammkkkkk",
        "kkKyrgyzmk,......{kmmkmmmkkkkkm/Sosal?kk",
        "Rustkkkkkb'k,....,awtf?kkkkkm7.kFlutterk",
        "blyatkkkk=1..'..'kkkwkkkkm7,mkjkkkkkkkkk",
        "kkkkkkk9'.[.............xmmkkk.4pHondakk",
        "kkk62ek...'.........,mkkkkkkkk..'km5kkkk",
        "7.e................'kkAnimekkk...jkkk8s5",
        ",..............jkm,,mkdockerkb/...kk'8k3",
        ".........{,..k,.4kkkkkCraftkk'/b...b..'9",
        "....,kmb..kkm4kk,Financekk=2mm1b........",
    ];

    // Right column content (Exact 23 lines)
    let right_column = vec![
        format!("{}@localhost", config.username),
        "-------------------------------------".to_string(),
        "OS: ....... Linux / WSL2".to_string(),
        format!("Uptime: ... {}", uptime),
        "Host: ..... Brain Cells Ltd.".to_string(),
        "Shell: .... Zsh + Tmux".to_string(),
        "Editor: ... NeoVim (What is a mouse?)".to_string(),
        "".to_string(),
        "* Identity --------------------------".to_string(),
        "Role: ..... Systems & Backend Dev".to_string(),
        "Background: Fine Arts 🎨 (6y Design)".to_string(),
        "Passions: . Physics ⚛️, Auto 🏎️".to_string(),
        "".to_string(),
        "* Quick Stack -----------------------".to_string(),
        "Primary: .. Rust 🦀".to_string(),
        "Secondary:  JS 🟡, SQL 🗄️, Flutter 💙".to_string(),
        "".to_string(),
        "* Direct Contact --------------------".to_string(),
        format!(
            "Telegram: . <a href=\"https://t.me/{}\">@{}</a>",
            config.telegram, config.telegram
        ),
        format!(
            "Email: .... <a href=\"mailto:{}\">{}</a>",
            config.email, config.email
        ),
        "".to_string(),
        "* Current Status --------------------".to_string(),
        "Focus: .... on my goals".to_string(),
    ];

    // Merge grid
    let mut top_grid = String::new();
    for i in 0..23 {
        let left_line = ascii_portrait
            .get(i)
            .copied()
            .unwrap_or("                                        ");
        let right_line = right_column.get(i).map_or("", |s| s.as_str());
        top_grid.push_str(&format!("{:40}   {}\n", left_line, right_line));
    }

    let readme_content = format!(
        r#"<pre>
{top_grid}================================================================================
1. Engineering DNA & Philosophy ------------------------------------------------
   - Systems & Backend: Obsessed with Zero-Alloc performance, cache locality,
                        memory ordering & deterministic DBs.
   - Product & Design:  Fine Arts degree + 6 years in Graphic Design & Branding.
                        Eliminating cognitive load with frictionless UX.
   - Pragmatic Stack:   Not a Luddite — I actively leverage modern AI tooling
                        for speed, but trust only hard math & DB invariants.
   - Passions:          Classical physics, automotive engineering, car tuning &
                        refactoring working code.
2. Live GitHub Metrics ---------------------------------------------------------
   - Public Repositories: {repos_count:<12} Total Stars: ...... {total_stars}
   - Account Followers: . {followers_count:<12} Total Commits: .... {total_commits}
   - Pull Requests: ..... {total_prs:<12} Active Project: ... DataCopter
{languages_block}================================================================================
BTC:  <a href="https://mempool.space/address/{btc}">{btc}</a>        (Trust Wallet)
ETH:  <a href="https://etherscan.io/address/{eth}">{eth}</a>        (Trust Wallet)
SOL:  <a href="https://solscan.io/account/{sol}">{sol}</a>      (Trust Wallet)
USDT: <a href="https://etherscan.io/address/{usdt}">{usdt}</a>        (Trust Wallet)
USDC: <a href="https://etherscan.io/address/{usdc}">{usdc}</a>        (Trust Wallet)
GRAM: <a href="https://tonviewer.com/{gram}">{gram}</a>  (Telegram Wallet)
</pre>

<p align="center">
  <a href="https://t.me/{telegram}" target="_blank"><img src="https://img.shields.io/badge/Telegram-26A69A?style=for-the-badge&logo=telegram&logoColor=white"/></a>
  <a href="mailto:{email}"><img src="https://img.shields.io/badge/Email-EA4335?style=for-the-badge&logo=gmail&logoColor=white"/></a>
  <a href="https://github.com/{username}"><img src="https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white"/></a>
</p>
"#,
        top_grid = top_grid,
        repos_count = public_repos_count,
        total_stars = total_stars,
        followers_count = followers_count,
        total_commits = total_commits,
        total_prs = total_prs,
        languages_block = languages_block,
        btc = config.btc_wallet,
        eth = config.eth_wallet,
        sol = config.sol_wallet,
        usdt = config.usdt_wallet,
        usdc = config.usdc_wallet,
        gram = config.gram_wallet,
        telegram = config.telegram,
        email = config.email,
        username = config.username,
    );

    let mut file = File::create("README.md")?;
    file.write_all(readme_content.as_bytes())?;

    println!("✅ README.md generated successfully!");
    Ok(())
}
