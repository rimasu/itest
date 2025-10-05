use std::path::Path;

use itest_runner::{
    components::container::ContainerTearDown, depends_on, set_up, Context, TearDown
};
use testcontainers::{GenericImage, ImageExt, core::Mount, runners::AsyncRunner};

#[set_up(Envoy)]
#[depends_on(Server)]
async fn set_up(ctx: Context) -> Result<impl TearDown, Box<dyn std::error::Error>> {
    let cfg = Path::new("../server/etc/envoy/envoy.yaml").canonicalize()?;
    let cfg = cfg.to_str().unwrap();

    let image = GenericImage::new("envoyproxy/envoy", "v1.33-latest")
        .with_container_name("itest-envoy")
        .with_mount(
            Mount::bind_mount(cfg, "/etc/envoy/envoy.yaml")
                .with_access_mode(testcontainers::core::AccessMode::ReadOnly),
        )
        .with_network("host");

    let container = image.start().await?;

    Ok(ContainerTearDown::new2(container))
}
