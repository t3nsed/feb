use anyhow::{Context, Result};
use clap::Parser;
use git2::{Repository, Commit};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "REPO_PATH")]
    repo_path: PathBuf,
    
    #[arg(long, value_name = "API_KEY", env = "DEEPSEEK_API_KEY")]
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommitAnalysisRequest {
    code_diff: String,
    commit_message: String,
}

#[derive(Debug, Deserialize)]
struct CommitAnalysisResponse {
    performance_score: f64,
    maintainability_score: f64,
    explanation: String,
}

#[derive(Debug)]
struct CommitterStats {
    name: String,
    email: String,
    performance_scores: Vec<f64>,
    maintainability_scores: Vec<f64>,
}

struct GitAnalyzer {
    repo: Repository,
    client: Client,
    api_key: String,
}

impl GitAnalyzer {
    fn new(repo_path: PathBuf, api_key: String) -> Result<Self> {
        let repo = Repository::open(&repo_path)
            .context("Failed to open repository")?;
        let client = Client::new();
        
        Ok(Self {
            repo,
            client,
            api_key,
        })
    }

    async fn analyze_commit(&self, commit: &Commit<'_>) -> Result<CommitAnalysisResponse> {
        let parent = commit.parent(0).ok();
        let tree = commit.tree()?;
        let parent_tree = parent.as_ref().map(|c| c.tree()).transpose()?;

        let mut diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            None,
        )?;
        diff.find_similar(None)?;
        
        let diff_text = String::from_utf8_lossy(&diff.stats()?.to_buf(
            git2::DiffStatsFormat::FULL,
            80,
        )?).into_owned();

        let request = CommitAnalysisRequest {
            code_diff: diff_text,
            commit_message: commit.message().unwrap_or("").to_string(),
        };

        let response = self.client.post("https://api.deepseek.com/v1/analyze")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<CommitAnalysisResponse>()
            .await?;

        Ok(response)
    }

    async fn analyze_repository(&self) -> Result<HashMap<String, CommitterStats>> {
        let mut stats = HashMap::new();
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk {
            let commit = self.repo.find_commit(oid?)?;
            let author = commit.author();
            let email = author.email().unwrap_or("unknown").to_string();
            
            let analysis = self.analyze_commit(&commit).await?;
            
            stats.entry(email.clone())
                .or_insert_with(|| CommitterStats {
                    name: author.name().unwrap_or("unknown").to_string(),
                    email: email.clone(),
                    performance_scores: Vec::new(),
                    maintainability_scores: Vec::new(),
                });
            
            if let Some(stats) = stats.get_mut(&email) {
                stats.performance_scores.push(analysis.performance_score);
                stats.maintainability_scores.push(analysis.maintainability_score);
            }
        }

        Ok(stats)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let analyzer = GitAnalyzer::new(args.repo_path, args.api_key.unwrap())?;
    
    let stats = analyzer.analyze_repository().await?;
    
    for (_, committer) in stats {
        let avg_performance = committer.performance_scores.iter().sum::<f64>() / 
            committer.performance_scores.len() as f64;
        let avg_maintainability = committer.maintainability_scores.iter().sum::<f64>() / 
            committer.maintainability_scores.len() as f64;
            
        println!("Committer: {} <{}>", committer.name, committer.email);
        println!("  Performance Score: {:.2}", avg_performance);
        println!("  Maintainability Score: {:.2}", avg_maintainability);
        println!();
    }

    Ok(())
}
