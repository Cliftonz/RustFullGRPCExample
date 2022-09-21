#[macro_use]
extern crate lazy_static;
extern crate pub_sub;

use std::collections::HashMap;
use pub_sub::PubSub;
use async_stream;
use std::ops::{Deref};
use std::pin::Pin;
use std::sync::Mutex;
use std::time::Duration;
use log::{info, trace, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use tokio_stream::{Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tonic::metadata::MetadataValue;
use tonic_health::server::HealthReporter;
use tonic_reflection::server::Builder;
use crossbeam_queue::SegQueue;
use crossbeam_channel::{unbounded};
use crossbeam;
use tokio::sync::mpsc;
use voting::{
    voting_server::{Voting, VotingServer},
    VotingRequest, VotingResponse, StatusResponse,
};

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    static ref CHANNEL: PubSub<VotingResponse> = pub_sub::PubSub::new();
    static ref QUEUE: crossbeam_queue::SegQueue<VotingResponse> =  SegQueue::new();
    static ref CCHANNEL: ( crossbeam::channel::Sender<VotingResponse>, crossbeam::channel::Receiver<VotingResponse>) = unbounded();
}

pub mod voting {
    tonic::include_proto!("voting");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("voting_descriptor");
}

fn match_vote(url: String, vote: i32) -> Result<Response<VotingResponse>, Status> {
    let mut map = HASHMAP.lock().unwrap();

    match vote {
        0 => {
            map.entry(url.to_string()).and_modify(|counter| *counter += 1).or_insert(1);
            let resp = Response::new(voting::VotingResponse {
                confirmation: { format!("Happy to confirm that you upvoted for {} who now has {} votes.", url, map.get_key_value(&*url.to_string()).unwrap().1  ) },
            });
            CCHANNEL.0.send(resp.get_ref().clone()).unwrap();
            Ok(resp)
        },
        1 => {
            map.entry(url.to_string()).and_modify(|counter| *counter -= 1).or_insert(1);
            let resp = Response::new(voting::VotingResponse {
                confirmation: { format!("Confirmation that you downvoted for {} who now has {} votes.", url, map.get_key_value(&*url.to_string()).unwrap().1) },
            });
            CCHANNEL.0.send(resp.get_ref().clone()).unwrap();
            Ok(resp)
        },
        _ =>{
            trace!("Vote Type unsupported");
            Err(Status::new(
                tonic::Code::OutOfRange,
                "Invalid vote provided",)
            )

        },
    }
}

#[derive(Debug, Default)]
pub struct VotingService {}

#[tonic::async_trait]
impl Voting for VotingService {

    async fn status(
        &self,
        _request: Request<()>,
    ) -> Result<Response<StatusResponse>, Status> {

        let map = HASHMAP.lock().unwrap();

        Ok(Response::new(voting::StatusResponse {
            candidates: {map.deref().clone()},
        }))

    }


    async fn vote(
        &self,
        request: Request<VotingRequest>,
    ) -> Result<Response<VotingResponse>, Status> {

        let r = request.into_inner();

        info!("Voting for {} with {}", r.url.to_string(), r.vote);

        match_vote( r.url,r.vote)

    }

    // use tokio::sync::mpsc;
    // use tonic::Status;
    //
    // use futures_core::Stream;
    // use std::pin::Pin;
    //
    // pub type SyncBoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a + Send + Sync>>;

    type VotingStreamStream =  // ReceiverStream<Result<VotingResponse, Status>>;
        //SyncBoxStream<'static, Result<VotingResponse, Status>>;
        Pin<Box<dyn Stream<Item = Result<VotingResponse, Status>> + 'static + Send + Sync >>;

    // type BatchVoteStream =  // ReceiverStream<Result<VotingResponse, Status>>;
    //     //SyncBoxStream<'static, Result<VotingResponse, Status>>;
    //     Pin<Box<dyn Stream<Item = Result<VotingResponse, Status>> + 'static + Send + Sync >>;

    async fn watch_stream(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::VotingStreamStream>, Status> {

        let (tx, rx) = mpsc::channel(1);

        let recx_clone: crossbeam::channel::Receiver<VotingResponse> = CCHANNEL.1.clone();

        let mut count: u32 = 0;
        tokio::spawn(async move {
            while let Ok(data) = recx_clone.recv() {  //timeout(Duration::from_secs(1))  {}
                //thread::sleep(Duration::from_millis(20));

                info!("{}", "sending data from server");

                tx.send(Ok(data)).await.unwrap();

                count += 1;

                info!("count: {}", &count);
            }
            info!("{}", "failed sending data from server");
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    type WatchStreamStream =
        Pin<Box<dyn Stream<Item = Result<VotingResponse, Status>> + 'static + Send + Sync >>;

    async fn batch_vote(
        &self,
        request: Request<tonic::Streaming<VotingRequest>>,
    ) -> Result<Response<()>, Status> {

        let mut stream: Streaming<VotingRequest> = request.into_inner();
        let mut packet_count: u32 = 0;

        while let Some(vote) = stream.next().await {
            let v_request: VotingRequest = vote.unwrap();

            match_vote(v_request.url, v_request.vote).unwrap();

            packet_count += 1;
        }

        info!("Submitted {} votes", packet_count);

        Ok(Response::new(()))
    }

    async fn voting_stream(
        &self,
        request: Request<tonic::Streaming<VotingRequest>>,
    ) -> Result<Response<Self::VotingStreamStream>, Status> {

        let (tx, rx) = mpsc::channel(1);

        let mut stream: Streaming<VotingRequest> = request.into_inner();
        let mut packet_count: u32 = 0;

        tokio::spawn(async move {
            while let Some(vote) = stream.next().await {
                let v_request: VotingRequest = vote.unwrap();

                let temp = match_vote(v_request.url, v_request.vote).unwrap().into_inner();

                tx.send(Ok(temp)).await.unwrap();

                packet_count += 1;
            }

            info!("{}", "failed sending data from server");
        });


        info!("Submitted {} votes", packet_count);

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))


    }


}

/// This function (somewhat improbably) flips the status of a service every second, in order
/// that the effect of `tonic_health::HealthReporter::watch` can be easily observed.
async fn twiddle_service_status(mut reporter: HealthReporter) {
    let mut iter = 0u64;
    loop {
        iter += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;

        if iter % 2 == 0 {
            reporter.set_serving::<VotingServer<VotingService>>().await;
        } else {
            reporter.set_not_serving::<VotingServer<VotingService>>().await;
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    info!("Configuring Logging");
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    info!("Configuring Server");
    let address = "[::1]:8080".parse().unwrap();
    let voting_service = VotingService::default();

    info!("Configuring Authentication");
    let svc =  voting::voting_server::VotingServer::with_interceptor(voting_service, check_auth);

    info!("Configuring Health Check");
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<VotingServer<VotingService>>()
        .await;

    tokio::spawn(twiddle_service_status(health_reporter.clone()));

    info!("Configuring Reflection");
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(voting::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::proto::GRPC_HEALTH_V1_FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    println!("GreeterServer listening on {}", address);
    Server::builder()
        .add_service(reflection_service)
        .add_service(svc)
        .add_service(health_service)
        .serve(address)
        .await?;
    Ok(())
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    info!("token is \'{}\'", token.to_str().unwrap());

    match req.metadata().get("authorization") {
        Some(t) if !token.is_empty() => { info!("value is {}",t.to_str().unwrap()); Ok(req)  },
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
