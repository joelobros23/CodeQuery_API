# Project Plan: CodeQuery API

**Description:** A REST API built with Rust and Actix-web for performing static code analysis and searching code repositories. It allows users to upload code snippets or provide repository URLs to identify potential vulnerabilities, code smells, and search for specific patterns.


## Development Goals

- [ ] Set up the Actix-web server in src/main.rs with a basic health check endpoint.
- [ ] Create the `src/handlers.rs` module to manage request handling.
- [ ] Define API endpoints: `/api/analyze` (for code analysis) and `/api/search` (for code search) in `src/handlers.rs`.
- [ ] Implement the `/api/analyze` endpoint: accept code snippet (as string or file upload) or repository URL.
- [ ] Implement basic code analysis (e.g., counting lines of code, identifying common keywords, simple linting rules).
- [ ] Implement the `/api/search` endpoint: accept a repository URL and a search query.
- [ ] Clone the repository (if a URL is provided) using tokio::process or libgit2 (consider security implications).
- [ ] Recursively search through the cloned repository for the provided query using `ignore` crate to respect .gitignore files.
- [ ] Return the search results (file paths and matching lines) as a JSON response.
- [ ] Create `src/models.rs` to define data structures for request payloads and response formats (e.g., `AnalysisRequest`, `AnalysisResponse`, `SearchRequest`, `SearchResponse`).
- [ ] Implement error handling and logging for all API endpoints.
- [ ] Add a utility function in `src/utils.rs` to download and extract tar.gz files from repository URLs.
