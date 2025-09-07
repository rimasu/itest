use testcontainers::{Container, ContainerRequest, GenericImage, Image, runners::SyncRunner};

use crate::{Context, SetUp, SetUpResult, TearDown};

pub struct ContainerSetUp {
    name: String,
    image: Option<ContainerRequest<GenericImage>>,
}

impl ContainerSetUp {
    pub fn new(request: ContainerRequest<GenericImage>) -> Box<ContainerSetUp> {
        let name = request.image().name().to_owned();
        Box::new(ContainerSetUp {
            name,
            image: Some(request),
        })
    }
}

impl SetUp for ContainerSetUp {
    fn set_up(&mut self, _ctx: &mut Context) -> SetUpResult {
        let image = self.image.take().unwrap();
        let container = image.start()?;
        Ok(Box::new(ContainerComponent { container }))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ContainerComponent {
    container: Container<GenericImage>,
}

impl TearDown for ContainerComponent {
    fn tear_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.container.stop()?;
        Ok(())
    }
}
