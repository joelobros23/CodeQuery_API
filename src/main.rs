use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::process::Command;
use std::env;

#[derive(Deserialize)]
struct CloneRequest {
    repo_url: String,
    target_dir: String, // Optional: Allow specifying the target directory
}

async fn clone_repo(req: web::Json<CloneRequest>) -> Result<HttpResponse> {
    let repo_url = &req.repo_url;
    let target_dir = &req.target_dir;

    // Ensure the target directory exists.  Crucial for security.
    if !Path::new(target_dir).exists() {
        if let Err(e) = fs::create_dir_all(target_dir) {
            eprintln!("Failed to create directory: {:?}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    // Sanitize the target directory to prevent directory traversal attacks.
    let current_dir = env::current_dir().unwrap();
    let absolute_target_path = current_dir.join(target_dir).canonicalize().map_err(|e| {
        eprintln!("Failed to canonicalize path: {:?}", e);
        actix_web::error::ErrorInternalServerError("Invalid target directory")
    })?;

    if !absolute_target_path.starts_with(current_dir) {
        eprintln!("Target directory is outside the current working directory.");
        return Ok(HttpResponse::BadRequest().body("Invalid target directory"));
    }

    println!("Cloning {} to {}", repo_url, target_dir);

    let mut cmd = Command::new("git");
    cmd.arg("clone").arg(repo_url).arg(absolute_target_path);

    let output = cmd.output().await.map_err(|e| {
        eprintln!("Failed to execute git clone: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to clone repository")
    })?;

    if output.status.success() {
        println!("Repository cloned successfully");
        Ok(HttpResponse::Ok().body("Repository cloned successfully"))
    } else {
        eprintln!("Git clone failed: {:?}", String::from_utf8_lossy(&output.stderr));
        Ok(HttpResponse::InternalServerError().body(format!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr))))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/clone", web::post().to(clone_repo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
