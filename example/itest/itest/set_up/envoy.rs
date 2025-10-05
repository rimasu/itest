use std::path::Path;

use itest_runner::{Context, depends_on, set_up};
use testcontainers::{GenericImage, ImageExt, core::Mount};

#[set_up(Envoy)]
#[depends_on(Server)]
fn set_up(ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
    // let cfg = Path::new("../server/etc/envoy/envoy.yaml")
    //     .canonicalize()
    //     .unwrap();
    // let cfg = cfg.to_str().unwrap();

    // let image = GenericImage::new("envoyproxy/envoy", "v1.33-latest")
    //     .with_container_name("itest-envoy")
    //     .with_mount(
    //         Mount::bind_mount(cfg, "/etc/envoy/envoy.yaml")
    //             .with_access_mode(testcontainers::core::AccessMode::ReadOnly),
    //     )
    //     .with_network("host")
    //     .into();

    // set_up_container(image)
    Ok(())
}
