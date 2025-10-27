use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use tokio::sync::Semaphore;
use lazy_static::lazy_static;
use dashmap::DashMap;

lazy_static! {
    static ref DOI_CACHE: DashMap<String, bool> = DashMap::new();
}

pub async fn validate_dois(dois: &[String]) -> f32 {
    if dois.is_empty() { return 0.0; }
    
    let client = Client::new();
    let sem = Arc::new(Semaphore::new(10));
    
    let tasks: Vec<_> = dois.iter().map(|doi| {
        let client = client.clone();
        let sem = sem.clone();
        let doi = doi.clone();
        
        async move {
            if let Some(cached) = DOI_CACHE.get(&doi) {
                return *cached;
            }
            
            let _permit = sem.acquire().await.unwrap();
            let valid = check_crossref(&client, &doi).await
                || check_semantic_scholar(&client, &doi).await;
            
            DOI_CACHE.insert(doi.clone(), valid);
            crate::filter::cache::save_doi_cache(&doi, valid);
            valid
        }
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    results.iter().filter(|&&v| v).count() as f32 / dois.len() as f32
}

async fn check_crossref(client: &Client, doi: &str) -> bool {
    let url = format!("https://api.crossref.org/works/{}", doi);
    client.get(&url)
        .timeout(Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn check_semantic_scholar(client: &Client, doi: &str) -> bool {
    let url = format!("https://api.semanticscholar.org/v1/paper/{}", doi);
    client.get(&url)
        .timeout(Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}


