use std::path::{Path, PathBuf};

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ogv",
];

pub fn pick_random(base_dir: &Path) -> anyhow::Result<Option<PathBuf>> {
    let mut videos = Vec::new();
    collect_videos(base_dir, &mut videos)?;

    if videos.is_empty() {
        return Ok(None);
    }

    use rand::seq::SliceRandom;
    let path = videos.choose(&mut rand::thread_rng()).cloned();
    Ok(path)
}

fn collect_videos(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Cannot read directory {:?}: {e}", dir);
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Skipping entry in {:?}: {e}", dir);
                continue;
            }
        };

        let path = entry.path();

        // checks file type (prevent symlink infinite recursion)
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => {
                collect_videos(&path, out)?;
            }
            Ok(_) => {
                if is_video_file(&path) {
                    out.push(path);
                }
            }
            Err(e) => {
                tracing::warn!("Cannot read entry metadata {:?}: {e}", path);
            }
        }
    }

    Ok(())
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("redhood-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"content").unwrap();
        path
    }

    #[test]
    fn pick_random_in_root() {
        let dir = tmp_dir("in_root");
        let expected = touch(&dir, "video.mp4");
        let result = pick_random(&dir).unwrap();
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn pick_random_in_subdir() {
        let dir = tmp_dir("in_subdir");
        let sub = dir.join("sub").join("deep");
        fs::create_dir_all(&sub).unwrap();
        let expected = touch(&sub, "movie.mkv");
        let result = pick_random(&dir).unwrap();
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn pick_random_no_videos() {
        let dir = tmp_dir("no_videos");
        touch(&dir, "readme.txt");
        touch(&dir, "image.png");
        let result = pick_random(&dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn pick_random_non_video_files_only() {
        let dir = tmp_dir("only_non_video");
        touch(&dir, "doc.pdf");
        touch(&dir, "notes.txt");
        let result = pick_random(&dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn pick_random_mixed_content() {
        let dir = tmp_dir("mixed");
        touch(&dir, "ignore.txt");
        let vid = touch(&dir, "clip.mp4");
        touch(&dir, "photo.jpg");
        let result = pick_random(&dir).unwrap();
        assert_eq!(result, Some(vid));
    }

    #[test]
    fn pick_random_nonexistent_dir() {
        let dir = Path::new("/tmp/redhood-test-nonexistent_should_never_exist");
        let result = pick_random(dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn pick_random_multiple_calls() {
        let dir = tmp_dir("multiple_calls");
        touch(&dir, "a.mp4");
        touch(&dir, "b.mkv");
        touch(&dir, "c.avi");

        for _ in 0..10 {
            let result = pick_random(&dir).unwrap();
            let p = result.expect("should find a video");
            assert!(is_video_file(&p), "result is not a video: {p:?}");
        }
    }

    #[test]
    fn pick_random_empty_dir() {
        let dir = tmp_dir("empty");
        let result = pick_random(&dir).unwrap();
        assert!(result.is_none());
    }
}
