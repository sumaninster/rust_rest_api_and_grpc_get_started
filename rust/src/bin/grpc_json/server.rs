use tonic::{transport::Server, Request, Response, Status};
use muscle_exercises_json::data_server::{Data, DataServer};
use muscle_exercises_json::{DataReply, DataRequest};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use api::api::get_data_async;

pub mod muscle_exercises_json {
    tonic::include_proto!("muscle_exercises_json");
}

#[derive(Default)]
pub struct MyGrpc {}

#[tonic::async_trait]
impl Data for MyGrpc {
    type SendReplyStream = ReceiverStream<Result<DataReply, Status>>;

    async fn send_reply (
        &self,
        request: Request<DataRequest>,
    ) -> Result<Response<Self::SendReplyStream>, Status> {
        println!("Got a request from {:?} : {:?}", request.remote_addr(), request);
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let DataRequest { id, name } = request.into_inner();
            let data = DataReply {
                message: get_data_async(String::from(&name).as_str(), &id).await,
            };
            tx.send(Ok(data)).await.unwrap();
        });
        return Ok(Response::new(ReceiverStream::new(rx)));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = api::api::grpc_server_url().parse().unwrap();
    let my_grpc = MyGrpc::default();
    println!("DataServer listening on {}", address);
    Server::builder()
        .add_service(DataServer::new(my_grpc))
        .serve(address)
        .await?;
    Ok(())
}