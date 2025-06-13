use actix_web::{web, App, HttpResponse, HttpServer, Responder, Error};
use serde::{Deserialize, Serialize};
use std::fs::File;
use flate2::read::GzDecoder;
use tar::Archive;
use std::path::Path;
use ignore::WalkBuilder;
use std::fs;
use std::io::Read;

#[derive(Deserialize)]
struct SearchQuery {
    repo_url: String,
    query: String,
}

#[derive(Serialize)]
struct SearchResult {
    file_path: String,
    line_number: usize,
    line: String,
}

async fn search(query: web::Json<SearchQuery>) -> Result<HttpResponse, Error> {
    println!("Received search request: repo_url={}, query={}", query.repo_url, query.query);

    let repo_url = &query.repo_url;
    let search_query = &query.query;

    // 1. Download the repository as a tar.gz archive.
    let archive_path = "repo.tar.gz";
    match download_repo(repo_url, archive_path).await {
        Ok(_) => println!("Repository downloaded successfully."),
        Err(e) => {
            eprintln!("Error downloading repository: {}", e);
            return Ok(HttpResponse::InternalServerError().body(format!("Failed to download repository: {}", e)));
        }
    };

    // 2. Extract the archive to a temporary directory.
    let extract_path = "temp_repo";
    match extract_archive(archive_path, extract_path) {
        Ok(_) => println!("Repository extracted successfully."),
        Err(e) => {
            eprintln!("Error extracting repository: {}", e);
            return Ok(HttpResponse::InternalServerError().body(format!("Failed to extract repository: {}", e)));
        }
    };

    // 3. Search for the query within the extracted files.
    let search_results = match search_files(extract_path, search_query) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error searching files: {}", e);
            return Ok(HttpResponse::InternalServerError().body(format!("Failed to search files: {}", e)));
        }
    };

    // 4. Clean up the temporary directory and archive.
    fs::remove_dir_all(extract_path).unwrap_or_else(|e| eprintln!("Failed to remove temp dir: {}", e));
    fs::remove_file(archive_path).unwrap_or_else(|e| eprintln!("Failed to remove archive: {}", e));

    // 5. Return the search results.
    Ok(HttpResponse::Ok().json(search_results))
}

async fn download_repo(repo_url: &str, archive_path: &str) -> Result<(), reqwest::Error> {
    let response = reqwest::get(repo_url).await?;
    let mut file = File::create(archive_path).expect("Failed to create archive file");
    let mut content =  std::io::Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file).expect("Failed to copy content to file");
    Ok(())
}

fn extract_archive(archive_path: &str, extract_path: &str) -> Result<(), std::io::Error> {
    let file = File::open(archive_path)?; 
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    archive.unpack(extract_path)?; 
    Ok(())
}

fn search_files(root_path: &str, query: &str) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results: Vec<SearchResult> = Vec::new();

    for result in WalkBuilder::new(root_path).build() {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.is_file() {
                    if let Some(file_path) = path.to_str() {
                        if let Ok(file_content) = fs::read_to_string(path) {
                            for (line_number, line) in file_content.lines().enumerate() {
                                if line.contains(query) {
                                    results.push(SearchResult {
                                        file_path: file_path.to_string(),
                                        line_number: line_number + 1,
                                        line: line.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }

    Ok(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/search", web::post().to(search))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
