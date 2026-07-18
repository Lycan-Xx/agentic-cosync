use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MirroredNotification {
    pub package_name: String,
    pub title: String,
    pub text: String,
    pub extras: HashMap<String, String>,
    pub posted_at_ms: i64,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationFilter {
    pub allow_list: HashSet<String>,
    pub deny_list: HashSet<String>,
}

impl NotificationFilter {
    pub fn new() -> Self { Self::default() }
    pub fn allow(mut self, package: &str) -> Self { self.allow_list.insert(package.to_lowercase()); self }
    pub fn deny(mut self, package: &str) -> Self { self.deny_list.insert(package.to_lowercase()); self }

    pub fn should_mirror(&self, package_name: &str) -> bool {
        let pkg = package_name.to_lowercase();
        if !self.allow_list.is_empty() { return self.allow_list.contains(&pkg); }
        !self.deny_list.contains(&pkg)
    }
}

pub struct NotificationMirror {
    filter: NotificationFilter,
    seen_keys: HashSet<String>,
    max_seen_keys: usize,
}

impl NotificationMirror {
    pub fn new() -> Self { Self { filter: NotificationFilter::new(), seen_keys: HashSet::new(), max_seen_keys: 1000 } }
    pub fn set_filter(&mut self, filter: NotificationFilter) { self.filter = filter; }
    pub fn filter(&self) -> &NotificationFilter { &self.filter }

    pub fn handle_notification_event(&mut self, package_name: &str, title: &str, text: &str,
        extras: HashMap<String, String>, posted_at_ms: i64) -> Option<MirroredNotification> {
        if !self.filter.should_mirror(package_name) { return None; }
        let key = format!("{}:{}:{}", package_name, title, posted_at_ms);
        if self.seen_keys.contains(&key) { return None; }
        self.seen_keys.insert(key.clone());
        if self.seen_keys.len() > self.max_seen_keys {
            if let Some(first) = self.seen_keys.iter().next().cloned() { self.seen_keys.remove(&first); }
        }
        Some(MirroredNotification { package_name: package_name.to_string(), title: title.to_string(),
            text: text.to_string(), extras, posted_at_ms })
    }

    pub fn build_reply(notification_key: &str, reply_text: &str) -> (String, String) {
        (notification_key.to_string(), reply_text.to_string())
    }
}

impl Default for NotificationMirror { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter_allows_all() {
        let f = NotificationFilter::new();
        assert!(f.should_mirror("com.whatsapp"));
    }

    #[test]
    fn test_deny_list() {
        let f = NotificationFilter::new().deny("com.spam.app");
        assert!(!f.should_mirror("com.spam.app"));
        assert!(f.should_mirror("com.whatsapp"));
    }

    #[test]
    fn test_allow_list_priority() {
        let f = NotificationFilter::new().allow("com.whatsapp").deny("com.whatsapp");
        assert!(f.should_mirror("com.whatsapp"));
        assert!(!f.should_mirror("com.telegram"));
    }

    #[test]
    fn test_notification_dedup() {
        let mut m = NotificationMirror::new();
        let ex = HashMap::new();
        assert!(m.handle_notification_event("com.test", "Hello", "World", ex.clone(), 1000).is_some());
        assert!(m.handle_notification_event("com.test", "Hello", "World", ex, 1000).is_none());
    }
}