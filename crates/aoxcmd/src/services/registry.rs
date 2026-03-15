#[derive(Debug, Clone, Default)]
pub struct ServiceRegistry {
    services: Vec<String>,
}

impl ServiceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn register(&mut self, service_name: impl Into<String>) {
        self.services.push(service_name.into());
    }

    #[must_use]
    pub fn services(&self) -> &[String] {
        &self.services
    }
}
