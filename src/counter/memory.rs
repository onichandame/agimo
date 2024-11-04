use super::base::Counter;

#[derive(Debug, Clone)]
pub(crate) struct MemoryCounter {
    active: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
}

impl MemoryCounter {
    pub(crate) fn new() -> Self {
        Self {
            active: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

impl Counter for MemoryCounter {
    type Error = std::convert::Infallible;

    async fn inc(&self, host: &str) -> Result<(), Self::Error> {
        let mut count = self.active.lock().unwrap();
        count
            .entry(host.to_owned())
            .and_modify(|v| *v += 1)
            .or_insert(1);
        Ok(())
    }

    async fn dec(&self, host: &str) -> Result<(), Self::Error> {
        let mut count = self.active.lock().unwrap();
        count
            .entry(host.to_owned())
            .and_modify(|v| *v -= 1)
            .or_insert(0);
        Ok(())
    }

    async fn get_active(&self, host: &str) -> Result<u64, Self::Error> {
        let count = self.active.lock().unwrap();
        match count.get(host) {
            Some(v) => Ok(*v),
            None => Ok(0),
        }
    }
}
