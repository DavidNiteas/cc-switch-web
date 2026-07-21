pub struct UsageCache;

impl UsageCache {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UsageCache {
    fn default() -> Self {
        Self::new()
    }
}
