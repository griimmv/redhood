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

        if path.is_dir() {
            collect_videos(&path, out)?;
        } else if is_video_file(&path) {
            out.push(path);
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
