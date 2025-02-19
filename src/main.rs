//! Git Repository File Concatenator
//! This tool takes any Git repository (local path or remote URL) and creates a single Markdown file
//! containing the repository structure and all file contents.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::error::Error;
use serde::{Serialize, Deserialize};
use tempfile::TempDir;

/// Represents a file or directory in the repository structure
#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    #[serde(rename = "type")]
    entry_type: String,      // "file" or "directory"
    name: String,            // Name of the file or directory
    path: String,            // Relative path from repository root
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,       // File size in bytes (None for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<FileEntry>>,  // Subdirectories and files (None for files)
}

/// Main processor struct that handles all file operations
struct FileProcessor {
    ignore_dirs: HashSet<String>,       // Directories to ignore (e.g., .git, node_modules)
    ignore_files: HashSet<String>,      // Files to ignore (e.g., .DS_Store)
    ignore_extensions: HashSet<String>, // File extensions to ignore (e.g., .exe, .dll)
}

impl FileProcessor {
    /// Creates a new FileProcessor with default ignore lists
    fn new() -> Self {
        // Initialize directories to ignore
        let mut ignore_dirs = HashSet::new();
        ignore_dirs.insert(".git".to_string());
        ignore_dirs.insert("node_modules".to_string());
        ignore_dirs.insert("target".to_string());
        ignore_dirs.insert("dist".to_string());
        ignore_dirs.insert("build".to_string());

        // Initialize files to ignore
        let mut ignore_files = HashSet::new();
        ignore_files.insert(".DS_Store".to_string());
        ignore_files.insert("yarn.lock".to_string());

        // Initialize file extensions to ignore
        let mut ignore_extensions = HashSet::new();
        for ext in [
            // Binaries
            "exe", "dll", "so", "dylib",
            // Images
            "png", "jpg", "jpeg", "gif", "ico", "bmp", "tiff", "webp",
            // Archives
            "zip", "rar", "7z", "tar", "gz", "bz2",
            // Other binary formats
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
            "class", "pyc", "pyo", "pyd",
            // Audio/Video
            "mp3", "mp4", "wav", "avi", "mov", "flv", "mkv",
            // Database
            "db", "sqlite", "sqlite3",
        ] {
            ignore_extensions.insert(ext.to_string());
        }

        Self {
            ignore_dirs,
            ignore_files,
            ignore_extensions,
        }
    }

    /// Recursively builds the file structure starting from the given directory
    fn get_file_structure(&self, dir: &Path, base_path: &Path) -> Result<Vec<FileEntry>, Box<dyn Error>> {
        let mut structure = Vec::new();
        let entries = fs::read_dir(dir)?;

        // Process each entry in the directory
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let relative_path = base_path.join(&name);

            if path.is_dir() {
                // Process directory if it's not in ignore list
                if !self.ignore_dirs.contains(&name) {
                    let children = self.get_file_structure(&path, &relative_path)?;
                    if !children.is_empty() {
                        structure.push(FileEntry {
                            entry_type: "directory".to_string(),
                            name,
                            path: relative_path.to_string_lossy().into_owned(),
                            size: None,
                            children: Some(children),
                        });
                    }
                }
            } else {
                // Process file if it's not in ignore list
                if !self.should_ignore_file(&name) {
                    structure.push(FileEntry {
                        entry_type: "file".to_string(),
                        name,
                        path: relative_path.to_string_lossy().into_owned(),
                        size: Some(entry.metadata()?.len()),
                        children: None,
                    });
                }
            }
        }

        Ok(structure)
    }

    /// Checks if a file should be ignored based on its name or extension
    fn should_ignore_file(&self, filename: &str) -> bool {
        // Check if the file is in the ignore list
        if self.ignore_files.contains(filename) {
            return true;
        }

        // Check if the file extension is in the ignore list
        if let Some(extension) = Path::new(filename).extension() {
            self.ignore_extensions.contains(&extension.to_string_lossy().to_string())
        } else {
            false
        }
    }

    /// Determines the programming language based on file extension
    fn get_language_from_ext(&self, filepath: &Path) -> String {
        let extension = filepath
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Map file extensions to their corresponding language for syntax highlighting
        match extension.as_str() {
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "py" => "python",
            "rb" => "ruby",
            "java" => "java",
            "cs" => "csharp",
            "cpp" | "hpp" => "cpp",
            "c" | "h" => "c",
            "rs" => "rust",
            "go" => "go",
            "php" => "php",
            "html" => "html",
            "css" => "css",
            "scss" => "scss",
            "md" => "markdown",
            "json" => "json",
            "xml" => "xml",
            "yaml" | "yml" => "yaml",
            "sh" | "bash" => "bash",
            "sql" => "sql",
            "kt" => "kotlin",
            "swift" => "swift",
            "r" => "r",
            "lua" => "lua",
            "pl" | "perl" => "perl",
            "dart" => "dart",
            "ex" | "exs" => "elixir",
            "erl" => "erlang",
            "fs" | "fsx" => "fsharp",
            "hs" => "haskell",
            "scala" => "scala",
            "toml" => "toml",
            _ => "",
        }.to_string()
    }

    /// Generates the complete markdown document for the repository
    fn generate_markdown(&self, repo_path: &str) -> Result<String, Box<dyn Error>> {
        // Handle both local paths and remote repositories
        let temp_dir;
        let repo_dir = if repo_path.starts_with("http") || repo_path.starts_with("git@") || repo_path.starts_with("ssh://") {
            // Clone remote repository to temporary directory
            temp_dir = TempDir::new()?;
            println!("Cloning repository to {:?}...", temp_dir.path());

            // Build git command with appropriate flags
            let mut git_cmd = Command::new("git");
            git_cmd.args(&["clone"]);

            // Add SSH specific flags if using SSH
            if repo_path.starts_with("git@") || repo_path.starts_with("ssh://") {
                git_cmd.args(&["-c", "core.sshCommand=ssh -o StrictHostKeyChecking=accept-new"]);
            }

            // Add repository URL and target directory
            git_cmd.args(&[repo_path, &temp_dir.path().to_string_lossy()]);

            // Execute the command
            let output = git_cmd.output()?;

            // Print any error messages from git
            if !output.stderr.is_empty() {
                eprintln!("Git output: {}", String::from_utf8_lossy(&output.stderr));
            }
            temp_dir.path().to_path_buf()
        } else {
            PathBuf::from(repo_path)
        };

        // Generate repository structure
        let structure = self.get_file_structure(&repo_dir, Path::new(""))?;

        // Create markdown document
        let mut markdown = String::from("# Repository Structure\n\n```json\n");
        markdown.push_str(&serde_json::to_string_pretty(&structure)?);
        markdown.push_str("\n```\n\n# File Contents\n\n");

        // Process all files and add their contents to the markdown
        self.process_files(&structure, &repo_dir, &mut markdown)?;

        Ok(markdown)
    }

    /// Recursively processes files and adds their contents to the markdown document
    fn process_files(&self, entries: &[FileEntry], base_dir: &Path, markdown: &mut String) -> Result<(), Box<dyn Error>> {
        for entry in entries {
            if entry.entry_type == "directory" {
                // Recursively process directory contents
                if let Some(ref children) = entry.children {
                    self.process_files(children, base_dir, markdown)?;
                }
            } else {
                // Process file contents
                let full_path = base_dir.join(&entry.path);

                // Try to read the file content, handle non-UTF8 files
                let content = match fs::read_to_string(&full_path) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Warning: Unable to read {} as UTF-8 text: {}", entry.path, e);
                        String::from("[Binary or non-UTF8 file content skipped]")
                    }
                };

                // Add file header and content to markdown
                markdown.push_str(&format!("## {}\n\n", entry.path));
                let lang = self.get_language_from_ext(&full_path);
                markdown.push_str(&format!("```{}\n", lang));
                markdown.push_str(&content);
                markdown.push_str("\n```\n\n");
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <repository-path-or-url>", args[0]);
        std::process::exit(1);
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all("./output")?;

    // Extract repository name from path or URL
    let repo_name = if args[1].ends_with('/') {
        args[1].trim_end_matches('/')
    } else {
        &args[1]
    };

    // Handle different URL formats
    let repo_name = if repo_name.starts_with("git@") {
        // SSH format: git@host:user/repo.git
        repo_name.split(':').last().unwrap_or(repo_name)
    } else {
        // HTTPS or local path format
        repo_name.split('/').last().unwrap_or(repo_name)
    };

    // Clean up the name
    let repo_name = repo_name
        .strip_suffix(".git")
        .unwrap_or(repo_name)
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_', "-");

    if repo_name.is_empty() {
        "repository".to_string()
    } else {
        repo_name.to_string()
    }

    // Process repository and generate markdown
    let processor = FileProcessor::new();
    let markdown = processor.generate_markdown(&args[1])?;

    // Create output file path
    let output_path = format!("./output/{}.md", repo_name);
    fs::write(&output_path, markdown)?;
    println!("Successfully generated {}", output_path);

    Ok(())
}
