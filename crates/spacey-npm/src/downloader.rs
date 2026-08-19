//! Async multithreaded package downloader.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tracing::{debug, error, info, instrument, warn};

use crate::cache::PackageCache;
use crate::error::{Result, SnpmError};
use crate::integrity::IntegrityChecker;
use crate::registry::RegistryClient;

/// A package to download.
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Tarball URL
    pub tarball_url: String,
    /// Expected integrity hash
    pub integrity: Option<String>,
    /// Expected SHA-1 hash (legacy)
    pub shasum: Option<String>,
    /// Destination path
    pub dest_path: PathBuf,
}

/// Result of a download task.
#[derive(Debug)]
pub struct DownloadResult {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Path to the downloaded tarball
    pub tarball_path: PathBuf,
    /// Whether it was from cache
    pub from_cache: bool,
}

/// Statistics for download operations.
#[derive(Debug, Default)]
pub struct DownloadStats {
    pub total: AtomicUsize,
    pub downloaded: AtomicUsize,
    pub from_cache: AtomicUsize,
    pub bytes_downloaded: AtomicU64,
    pub failed: AtomicUsize,
}

impl DownloadStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_downloaded(&self) {
        self.downloaded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_hit(&self) {
        self.from_cache.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_failed(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.bytes_downloaded.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn summary(&self) -> String {
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        let from_cache = self.from_cache.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let bytes = self.bytes_downloaded.load(Ordering::Relaxed);
        format!(
            "Downloaded: {}, From cache: {}, Failed: {}, Total bytes: {}",
            downloaded,
            from_cache,
            failed,
            format_bytes(bytes)
        )
    }
}

/// Async multithreaded package downloader.
pub struct PackageDownloader {
    registry: RegistryClient,
    cache: PackageCache,
    concurrency: usize,
    integrity_checker: IntegrityChecker,
    stats: Arc<DownloadStats>,
    multi_progress: Arc<MultiProgress>,
    show_progress: bool,
}

impl PackageDownloader {
    pub fn new(
        registry: RegistryClient,
        cache: PackageCache,
        concurrency: usize,
        show_progress: bool,
    ) -> Self {
        Self {
            registry,
            cache,
            concurrency: concurrency.max(1),
            integrity_checker: IntegrityChecker::new(),
            stats: Arc::new(DownloadStats::new()),
            multi_progress: Arc::new(MultiProgress::new()),
            show_progress,
        }
    }

    pub fn stats(&self) -> &Arc<DownloadStats> {
        &self.stats
    }

    /// Get the registry client for direct downloads.
    pub fn downloader_client(&self) -> &RegistryClient {
        &self.registry
    }

    #[instrument(skip(self, tasks))]
    pub async fn download_packages(&self, tasks: Vec<DownloadTask>) -> Result<Vec<DownloadResult>> {
        if tasks.is_empty() {
            return Ok(vec![]);
        }

        self.stats.total.store(tasks.len(), Ordering::Relaxed);

        info!(
            "Downloading {} packages with {} concurrent connections",
            tasks.len(),
            self.concurrency
        );

        let progress = if self.show_progress {
            let pb = self
                .multi_progress
                .add(ProgressBar::new(tasks.len() as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };

        let results: Vec<Result<DownloadResult>> = stream::iter(tasks)
            .map(|task| {
                let registry = self.registry.clone();
                let cache = self.cache.clone();
                let stats = self.stats.clone();
                let integrity_checker = self.integrity_checker.clone();
                let progress = progress.clone();

                async move {
                    let result = download_single_package(
                        &registry,
                        &cache,
                        &integrity_checker,
                        task,
                        &stats,
                    )
                    .await;

                    if let Some(pb) = &progress {
                        pb.inc(1);
                    }

                    result
                }
            })
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        if let Some(pb) = progress {
            pb.finish_with_message("Done!");
        }

        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(r) => successes.push(r),
                Err(e) => {
                    self.stats.inc_failed();
                    errors.push(e);
                }
            }
        }

        for error in &errors {
            error!("Download error: {}", error);
        }

        if successes.is_empty() && !errors.is_empty() {
            return Err(errors.remove(0));
        }

        info!("{}", self.stats.summary());

        Ok(successes)
    }

    pub async fn download_package(&self, task: DownloadTask) -> Result<DownloadResult> {
        let results = self.download_packages(vec![task]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| SnpmError::Other("Download failed".into()))
    }
}

async fn download_single_package(
    registry: &RegistryClient,
    cache: &PackageCache,
    integrity_checker: &IntegrityChecker,
    task: DownloadTask,
    stats: &DownloadStats,
) -> Result<DownloadResult> {
    let cache_key = format!("{}@{}", task.name, task.version);

    if let Some(cached_path) = cache.get_tarball(&task.name, &task.version) {
        if cached_path.exists() {
            debug!("Cache hit for {}", cache_key);
            stats.inc_cache_hit();
            return Ok(DownloadResult {
                name: task.name,
                version: task.version,
                tarball_path: cached_path,
                from_cache: true,
            });
        }
    }

    debug!("Downloading {} from {}", cache_key, task.tarball_url);

    let bytes: bytes::Bytes = registry.download_tarball(&task.tarball_url).await?;
    let bytes_len = bytes.len() as u64;

    if let Some(ref integrity) = task.integrity {
        if !integrity_checker.verify_integrity(&bytes, integrity) {
            let actual = integrity_checker.compute_integrity(&bytes);
            return Err(SnpmError::IntegrityMismatch {
                package: cache_key,
                expected: integrity.clone(),
                actual,
            });
        }
        debug!("Integrity verified for {}", task.name);
    } else if let Some(ref shasum) = task.shasum {
        if !integrity_checker.verify_shasum(&bytes, shasum) {
            let actual = integrity_checker.compute_shasum(&bytes);
            return Err(SnpmError::IntegrityMismatch {
                package: cache_key,
                expected: shasum.clone(),
                actual,
            });
        }
        debug!("SHA-1 verified for {}", task.name);
    } else {
        warn!("No integrity hash for {}, skipping verification", task.name);
    }

    let tarball_path = cache
        .store_tarball(&task.name, &task.version, &bytes)
        .await?;

    stats.inc_downloaded();
    stats.add_bytes(bytes_len);

    Ok(DownloadResult {
        name: task.name,
        version: task.version,
        tarball_path,
        from_cache: false,
    })
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Parallel tarball extractor using rayon.
pub struct TarballExtractor {
    num_workers: usize,
}

impl TarballExtractor {
    pub fn new(num_workers: usize) -> Self {
        Self {
            num_workers: num_workers.max(1),
        }
    }

    #[instrument(skip(self, extractions))]
    pub fn extract_all(&self, extractions: Vec<(PathBuf, PathBuf)>) -> Result<Vec<PathBuf>> {
        use rayon::prelude::*;

        info!(
            "Extracting {} packages using {} threads",
            extractions.len(),
            self.num_workers
        );

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_workers)
            .build()
            .map_err(|e| SnpmError::Other(e.to_string()))?;

        let results: Vec<Result<PathBuf>> = pool.install(|| {
            extractions
                .par_iter()
                .map(|(tarball, dest)| {
                    extract_tarball(tarball, dest)?;
                    Ok(dest.clone())
                })
                .collect()
        });

        let mut successes = Vec::new();
        for result in results {
            match result {
                Ok(path) => successes.push(path),
                Err(e) => error!("Extraction error: {}", e),
            }
        }

        Ok(successes)
    }
}

fn extract_tarball(tarball_path: &Path, dest_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    debug!(
        "Extracting {} to {}",
        tarball_path.display(),
        dest_path.display()
    );

    let file = std::fs::File::open(tarball_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    std::fs::create_dir_all(dest_path)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        let stripped_path: PathBuf = path.components().skip(1).collect();
        if stripped_path.as_os_str().is_empty() {
            continue;
        }

        let dest_file = dest_path.join(&stripped_path);

        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(&dest_file)?;
    }

    Ok(())
}
