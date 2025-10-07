use async_trait::async_trait;
use testcontainers::{ContainerAsync, GenericImage};

use crate::{Context, TearDown};

pub struct ContainerTearDown {
    container: Option<ContainerAsync<GenericImage>>,
}

impl ContainerTearDown {
    pub fn new(container: ContainerAsync<GenericImage>) -> Self {
        Self {
            container: Some(container),
        }
    }
}

#[async_trait]
impl TearDown for ContainerTearDown {
    async fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(container) = self.container.take() {
            container.stop().await?;
            container.rm().await?;
        }
        Ok(())
    }
}
