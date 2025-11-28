//! File hashing for deduplication.

use crate::error::AudioError;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::trace;

/// Compute a SHA-256 hash of a file's contents.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn compute_file_hash(path: &Path) -> Result<String, AudioError> {
    use sha2::{Digest, Sha256};

    trace!("Computing hash for: {}", path.display());

    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let hash = hex::encode(result);

    trace!("Hash for {}: {}", path.display(), hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_hash() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let hash = compute_file_hash(file.path()).unwrap();

        // Known SHA-256 hash of "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_hash_nonexistent_file() {
        let result = compute_file_hash(Path::new("/nonexistent/file.mp3"));
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let hash = compute_file_hash(file.path()).unwrap();

        // SHA-256 of empty input
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
