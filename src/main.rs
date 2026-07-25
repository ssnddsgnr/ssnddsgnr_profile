use chrono::{Datelike, NaiveDate, Utc};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct GithubUser {
    public_repos: u32,
    followers: u32,
}

#[derive(Deserialize, Debug)]
struct GithubRepo {
    stargazers_count: u32,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProfileConfig::default();
    println!("🚀 Generating profile README for {}...", config.username);

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("rust-profile-generator"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // 1. Fetch GitHub stats
    let user_url = format!("https://api.github.com/users/{}", config.username);
    let user_res = client.get(&user_url).send().await?;
    let (repos_count, followers_count) = if user_res.status().is_success() {
        let user_data: GithubUser = user_res.json().await?;
        (user_data.public_repos, user_data.followers)
    } else {
        (0, 0)
    };

    // 2. Fetch repos to calculate stars
    let repos_url = format!(
        "https://api.github.com/users/{}/repos?per_page=100",
        config.username
    );
    let repos_res = client.get(&repos_url).send().await?;
    let total_stars: u32 = if repos_res.status().is_success() {
        let repos: Vec<GithubRepo> = repos_res.json().await?;
        repos.iter().map(|r| r.stargazers_count).sum()
    } else {
        0
    };

    let uptime = calculate_uptime(config.birth_date);

    // 3. ASCII Portrait (Exact 40-char width, 23 lines)
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

    // Merge grid: 40 chars + 3 spaces + Right Column
    let mut top_grid = String::new();
    for i in 0..23 {
        let left_line = ascii_portrait
            .get(i)
            .copied()
            .unwrap_or("                                        ");
        let right_line = right_column.get(i).map_or("", |s| s.as_str());
        top_grid.push_str(&format!("{:40}   {}\n", left_line, right_line));
    }

    // Build complete README
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
   - Public Repositories: {repos_count:<18} Total Stars: ...... {total_stars}
   - Account Followers: . {followers_count:<18} Active Project: ... DataCopter
================================================================================
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
        top_grid = top_grid.trim_end(),
        repos_count = repos_count,
        total_stars = total_stars,
        followers_count = followers_count,
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
