use crate::{services::HubService, web};

pub struct App {
    service: HubService,
}

impl App {
    pub async fn bootstrap() -> Result<Self, std::io::Error> {
        Ok(Self {
            service: HubService::new(),
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        web::serve(self.service).await
    }
}
