use crate::{environments::Environment, services::HubService, web};

pub struct App {
    service: HubService,
}

impl App {
    pub async fn bootstrap() -> Result<Self, std::io::Error> {
        let service = HubService::new();
        if let Ok(raw) = std::env::var("AOXCHUB_DEFAULT_ENV") {
            if let Some(env) = Environment::from_slug(&raw) {
                service.set_environment(env).await;
            }
        }

        Ok(Self { service })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        web::serve(self.service).await
    }
}
