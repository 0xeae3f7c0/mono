//! Internal gRPC service (tonic) that answers "how active is this repository?"
//! for other services, e.g. `webhooks` and `pipelines`. Messages and service
//! are defined in Rust, so no protoc is needed.
//!
//!   activity-rpc serve 127.0.0.1:50051
//!   activity-rpc call http://127.0.0.1:50051 owner/name

use std::time::Duration;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Clone, PartialEq, prost::Message)]
pub struct SummaryRequest {
    #[prost(string, tag = "1")]
    pub repo: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SummaryReply {
    #[prost(string, tag = "1")]
    pub summary: String,
}

pub mod activity {
    use super::{SummaryReply, SummaryRequest};
    use tonic::codegen::*;

    const PATH: &str = "/activity.Activity/Summarize";

    #[tonic::async_trait]
    pub trait Activity: Send + Sync + 'static {
        async fn summarize(
            &self,
            request: tonic::Request<SummaryRequest>,
        ) -> Result<tonic::Response<SummaryReply>, tonic::Status>;
    }

    /// tower Service adapter routing the single RPC to a `Activity`.
    pub struct ActivityServer<T>(pub Arc<T>);

    impl<T> Clone for ActivityServer<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> tonic::server::NamedService for ActivityServer<T> {
        const NAME: &'static str = "activity.Activity";
    }

    impl<T: Activity, B: Body + Send + 'static> Service<http::Request<B>> for ActivityServer<T>
    where
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.0.clone();
            if req.uri().path() != PATH {
                return Box::pin(async { Ok(tonic::Status::unimplemented("").into_http()) });
            }
            struct Svc<T>(Arc<T>);
            impl<T: Activity> tonic::server::UnaryService<SummaryRequest> for Svc<T> {
                type Response = SummaryReply;
                type Future = BoxFuture<tonic::Response<SummaryReply>, tonic::Status>;
                fn call(&mut self, request: tonic::Request<SummaryRequest>) -> Self::Future {
                    let inner = self.0.clone();
                    Box::pin(async move { inner.summarize(request).await })
                }
            }
            Box::pin(async move {
                let codec = tonic_prost::ProstCodec::default();
                Ok(tonic::server::Grpc::new(codec).unary(Svc(inner), req).await)
            })
        }
    }

    /// Client for the greeter service.
    pub struct ActivityClient(tonic::client::Grpc<tonic::transport::Channel>);

    impl ActivityClient {
        pub fn new(channel: tonic::transport::Channel) -> Self {
            Self(tonic::client::Grpc::new(channel))
        }

        pub async fn summarize(
            &mut self,
            request: SummaryRequest,
        ) -> Result<SummaryReply, tonic::Status> {
            self.0
                .ready()
                .await
                .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(PATH);
            Ok(self
                .0
                .unary(tonic::Request::new(request), path, codec)
                .await?
                .into_inner())
        }
    }
}

/// Event counts per repository (in production loaded from `events-api`).
#[derive(Default)]
struct Counts(std::collections::HashMap<String, u64>);

#[tonic::async_trait]
impl activity::Activity for Counts {
    async fn summarize(
        &self,
        request: Request<SummaryRequest>,
    ) -> Result<Response<SummaryReply>, Status> {
        let repo = request.into_inner().repo;
        if !repo.contains('/') {
            return Err(Status::invalid_argument("repo must look like owner/name"));
        }
        let events = self.0.get(&repo).copied().unwrap_or(0);
        Ok(Response::new(SummaryReply {
            summary: format!("{repo}: {events} events"),
        }))
    }
}

fn sample() -> Counts {
    Counts([("commit/commit".to_owned(), 42)].into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["serve", addr] => {
            Server::builder()
                .add_service(activity::ActivityServer(std::sync::Arc::new(sample())))
                .serve(addr.parse()?)
                .await?;
        }
        ["call", url, repo] => {
            let channel = tonic::transport::Endpoint::from_shared(url.to_string())?
                .timeout(Duration::from_secs(5))
                .connect()
                .await?;
            let reply = activity::ActivityClient::new(channel)
                .summarize(SummaryRequest {
                    repo: repo.to_string(),
                })
                .await?;
            println!("{}", reply.summary);
        }
        _ => eprintln!("usage: activity-rpc serve <addr> | activity-rpc call <url> <owner/name>"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn start() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(
            Server::builder()
                .add_service(activity::ActivityServer(std::sync::Arc::new(sample())))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
        );
        format!("http://{addr}")
    }

    async fn client(url: String) -> activity::ActivityClient {
        activity::ActivityClient::new(
            tonic::transport::Endpoint::from_shared(url)
                .unwrap()
                .connect()
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn summarizes_over_the_network() {
        let mut client = client(start().await).await;
        let reply = client
            .summarize(SummaryRequest {
                repo: "commit/commit".into(),
            })
            .await
            .unwrap();
        assert_eq!(reply.summary, "commit/commit: 42 events");
        let reply = client
            .summarize(SummaryRequest {
                repo: "o/unknown".into(),
            })
            .await
            .unwrap();
        assert_eq!(reply.summary, "o/unknown: 0 events");
    }

    #[tokio::test]
    async fn malformed_repo_is_invalid() {
        let mut client = client(start().await).await;
        let status = client
            .summarize(SummaryRequest::default())
            .await
            .unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }
}
