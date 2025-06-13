use actix_web::{web, HttpResponse, Responder, Error};
use serde::{Deserialize, Serialize};
use reqwest;
use std::fs::File;
use flate2::read::GzDecoder;
use tar::Archive;
use ignore::gitignore::GitignoreBuilder;
use ignore::WalkBuilder;

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub repo_url: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub results: Vec<String>,
}

async fn download_and_extract_repo(repo_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(repo_url).await?;
    let content = response.bytes().await?;

    let gz_decoder = GzDecoder::new(&content[..]);
    let mut archive = Archive::new(gz_decoder);

    let extract_path = format!("./tmp/{}", chrono::Utc::now().timestamp_nanos());
    std::fs::create_dir_all(&extract_path)?; // Ensure the directory exists

    archive.unpack(&extract_path)?;  // Unpack into the created directory

    Ok(extract_path)
}

async fn search_code(query: &str, repo_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    let mut ignore_builder = GitignoreBuilder::new(repo_path);
    if ignore_builder.add(".gitignore").is_err() {
        println!("Warning: Could not read .gitignore");
    }
    let ignore = ignore_builder.build()?;

    let walker = WalkBuilder::new(repo_path)
        .hidden(false) // Don't ignore hidden files
        .git_ignore(false) // Use our custom ignore
        .build();

    for result in walker {
        let entry = result?;
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            if ignore.matched(entry.path(), false).is_ignore() {
                continue;
            }

            let file_content = std::fs::read_to_string(entry.path())?;
            if file_content.contains(query) {
                results.push(entry.path().display().to_string());
            }
        }
    }

    Ok(results)
}

pub async fn handle_query(req: web::Json<QueryRequest>) -> Result<impl Responder, Error> {
    println!("Received query: {}", &req.query);
    println!("Received repo_url: {}", &req.repo_url);

    let repo_path_result = download_and_extract_repo(&req.repo_url).await;

    match repo_path_result {
        Ok(repo_path) => {
            let search_results_result = search_code(&req.query, &repo_path).await;

            match search_results_result {
                Ok(search_results) => {
                    // Clean up the temporary directory
                    if let Err(e) = std::fs::remove_dir_all(&repo_path) {
                        eprintln!("Error cleaning up temporary directory: {}", e);
                    }
                    Ok(HttpResponse::Ok().json(QueryResponse { results: search_results }))
                }
                Err(e) => {
                     // Clean up the temporary directory even if search fails
                    if let Err(cleanup_err) = std::fs::remove_dir_all(&repo_path) {
                        eprintln!("Error cleaning up temporary directory: {}", cleanup_err);
                    }
                    Err(actix_web::error::ErrorInternalServerError(format!("Search failed: {}", e)))
                }
            }
        }
        Err(e) => {
            Err(actix_web::error::ErrorBadRequest(format!("Download and extract failed: {}", e)))
        }
    }
}
