use actix_web::{web, App, HttpResponse, HttpServer, Responder, Error};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use ignore::WalkBuilder;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct CodeQueryRequest {
    repository_url: String,
}

#[derive(Debug, Serialize)]
struct CodeAnalysisResponse {
    total_lines: usize,
    keyword_counts: HashMap<String, usize>,
    // Add more analysis data here
}

async fn analyze_code(req: web::Json<CodeQueryRequest>) -> Result<HttpResponse, Error> {
    let repo_url = &req.repository_url;

    // Basic implementation - replace with actual cloning/fetching
    // and proper error handling
    let temp_dir = std::env::temp_dir().join("codequery_temp");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }

    std::fs::create_dir_all(&temp_dir).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Placeholder: Assume files are already in temp_dir
    // In reality, clone the git repository to temp_dir
    // Example using reqwest (needs error handling, proper git clone lib):
    // let output = Command::new("git").arg("clone").arg(repo_url).arg(&temp_dir).output().expect("Failed to execute git clone");
    // if !output.status.success() {
    //     eprintln!("Error cloning repository: {:?}", String::from_utf8_lossy(&output.stderr));
    //     return Err(actix_web::error::ErrorInternalServerError("Failed to clone repository"));
    // }

    let analysis_result = perform_code_analysis(&temp_dir).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    std::fs::remove_dir_all(&temp_dir).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(analysis_result))
}

fn perform_code_analysis(repo_path: &Path) -> Result<CodeAnalysisResponse, std::io::Error> {
    let mut total_lines = 0;
    let mut keyword_counts: HashMap<String, usize> = HashMap::new();
    let keywords = vec!["fn", "let", "if", "else", "for", "while", "return", "struct", "enum", "impl"];

    let walker = WalkBuilder::new(repo_path).build();

    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "rs" || ext == "py" || ext == "js" || ext == "java" || ext == "go" {

                            let file = File::open(entry.path())?;
                            let reader = BufReader::new(file);

                            for line_result in reader.lines() {
                                let line = line_result?;
                                total_lines += 1;

                                for keyword in &keywords {
                                    if line.contains(keyword) {
                                        *keyword_counts.entry(keyword.to_string()).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    Ok(CodeAnalysisResponse {
        total_lines,
        keyword_counts,
    })
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Service is healthy")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/analyze", web::post().to(analyze_code))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
