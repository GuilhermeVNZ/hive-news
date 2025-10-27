use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use tokio::sync::Semaphore;

pub async fn validate_authors(authors: &[String]) -> f32 {
    if authors.is_empty() { return 0.0; }
    
    let client = Client::new();
    let sem = Arc::new(Semaphore::new(5));
    
    let tasks: Vec<_> = authors.iter().map(|author| {
        let client = client.clone();
        let sem = sem.clone();
        let author = author.clone();
        
        async move {
            let _permit = sem.acquire().await.unwrap();
            check_orcid(&client, &author).await
                || check_semantic_scholar_author(&client, &author).await
        }
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    results.iter().filter(|&&v| v).count() as f32 / authors.len() as f32
}

async fn check_orcid(client: &Client, author: &str) -> bool {
    let url = format!("https://pub.orcid.org/v3.0/search?q={}", urlencoding::encode(author));
    client.get(&url)
        .timeout(Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn check_semantic_scholar_author(client: &Client, author: &str) -> bool {
    let url = format!("https://api.semanticscholar.org/v1/author/search?query={}", urlencoding::encode(author));
    client.get(&url)
        .timeout(Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}


