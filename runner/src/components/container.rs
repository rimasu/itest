use testcontainers::{Container, ContainerRequest, GenericImage, Image, runners::SyncRunner};

use crate::{SetUp, SetUpResult, TearDown};

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
    fn set_up(&mut self) -> SetUpResult {
        let image = self.image.take().unwrap();
        match image.start() {
            Ok(container) => Ok(Box::new(ContainerComponent { container })),
            Err(e) => Err(()),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ContainerComponent {
    container: Container<GenericImage>,
}

impl TearDown for ContainerComponent {
    fn tear_down(&self) -> Result<(), ()> {
        self.container.stop().unwrap();
        Ok(())
    }
}
