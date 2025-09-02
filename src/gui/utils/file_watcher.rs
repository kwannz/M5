use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs;

pub struct FileWatcher {
    watched_files: Vec<WatchedFile>,
    check_interval: Duration,
    last_check: Instant,
}

struct WatchedFile {
    path: PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watched_files: Vec::new(),
            check_interval: Duration::from_secs(1),
            last_check: Instant::now(),
        }
    }
    
    pub fn watch(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.watched_files.push(WatchedFile {
            path,
            last_modified: None,
        });
    }
    
    pub async fn check_changes(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        if now.duration_since(self.last_check) < self.check_interval {
            return Vec::new();
        }
        
        self.last_check = now;
        let mut changed_files = Vec::new();
        
        for watched_file in &mut self.watched_files {
            if let Ok(metadata) = fs::metadata(&watched_file.path).await {
                if let Ok(modified) = metadata.modified() {
                    let file_changed = match watched_file.last_modified {
                        Some(last_modified) => modified > last_modified,
                        None => true, // First check
                    };
                    
                    if file_changed {
                        watched_file.last_modified = Some(modified);
                        changed_files.push(watched_file.path.clone());
                    }
                }
            }
        }
        
        changed_files
    }
    
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        let mut watcher = Self::new();
        
        // Watch common DeskAgent files
        watcher.watch("progress/sprint-01.progress.json");
        watcher.watch("status/REPORT.md");
        watcher.watch("reviews/AI_REVIEW.md");
        watcher.watch("plans/sprint-01.plan.json");
        watcher.watch("routing/log.jsonl");
        
        watcher
    }
}