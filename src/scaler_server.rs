use tonic::transport::Server;

use crate::{
    event::Sender, externalscaler::external_scaler_server::ExternalScalerServer,
    scaler_api::ScalerApi,
};

const REFLECTION_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("externalscaler");

pub(crate) async fn scaler_server(sender: Sender) {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(REFLECTION_FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();
    Server::builder()
        .add_service(ExternalScalerServer::new(ScalerApi::new(sender)))
        .add_service(reflection_service)
        .serve("127.0.0.1:8081".parse().unwrap())
        .await
        .unwrap();
}
