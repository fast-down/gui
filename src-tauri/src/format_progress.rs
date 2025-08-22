use fast_down::ProgressEntry;

pub fn fmt_progress(progress: &[Vec<ProgressEntry>]) -> Vec<Vec<(u64, u64)>> {
    progress
        .iter()
        .map(|v| v.iter().map(|r| (r.start, r.end)).collect())
        .collect()
}
