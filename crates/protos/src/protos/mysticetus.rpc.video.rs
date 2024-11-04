/// Response after triggering
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateFramesResponse {
    #[prost(oneof = "generate_frames_response::Status", tags = "1, 2")]
    pub status: ::core::option::Option<generate_frames_response::Status>,
}
/// Nested message and enum types in `GenerateFramesResponse`.
pub mod generate_frames_response {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GenerateFramesInProgress {
        #[prost(message, required, tag = "1")]
        pub progress: super::super::super::common::Progress,
        #[prost(string, optional, tag = "2")]
        pub message: ::core::option::Option<::prost::alloc::string::String>,
    }
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Status {
        #[prost(message, tag = "1")]
        InProgress(GenerateFramesInProgress),
        /// The final mesage
        #[prost(message, tag = "2")]
        Completed(super::super::super::video::VideoInfo),
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListVideosRequest {
    #[prost(
        enumeration = "super::super::video::VideoLabelingStatus",
        optional,
        tag = "1"
    )]
    pub status: ::core::option::Option<i32>,
    #[prost(string, repeated, tag = "2")]
    pub projects: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, optional, tag = "3")]
    pub include_testing: ::core::option::Option<bool>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListFramesRequest {
    /// Start after the given 'frame_slug'.
    #[prost(string, optional, tag = "3")]
    pub start_after_slug: ::core::option::Option<::prost::alloc::string::String>,
    /// Limit to a given number of documents.
    #[prost(uint32, optional, tag = "4")]
    pub limit: ::core::option::Option<u32>,
    /// How we want to query for the video. Lets us query by video path or
    /// video slug, whichever is more convienient.
    #[prost(oneof = "list_frames_request::By", tags = "1, 2")]
    pub by: ::core::option::Option<list_frames_request::By>,
}
/// Nested message and enum types in `ListFramesRequest`.
pub mod list_frames_request {
    /// How we want to query for the video. Lets us query by video path or
    /// video slug, whichever is more convienient.
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum By {
        #[prost(string, tag = "1")]
        VideoSlug(::prost::alloc::string::String),
        #[prost(string, tag = "2")]
        VideoPath(::prost::alloc::string::String),
    }
}
/// Generated client implementations.
pub mod video_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    ///
    #[derive(Debug, Clone)]
    pub struct VideoServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl VideoServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> VideoServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> VideoServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            VideoServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Trigger the frame generation for a given video.
        ///
        /// Streams back progress updates, and the final message in the stream to
        /// indicate completion is the the updated 'VideoInfo', with 'frame_path' set.
        pub async fn generate_frames(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::video::VideoInfo>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::GenerateFramesResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mysticetus.rpc.video.VideoService/GenerateFrames",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        /// Updates info about a single video. Merges the request with the existing
        /// data, and returns the merged result.
        pub async fn update_video_info(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::video::VideoInfo>,
        ) -> Result<tonic::Response<super::super::super::video::VideoInfo>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mysticetus.rpc.video.VideoService/UpdateVideoInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Returns a stream of video docs. Can filter/query the results via the
        /// request.
        pub async fn list_videos(
            &mut self,
            request: impl tonic::IntoRequest<super::ListVideosRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::super::super::video::VideoInfo>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mysticetus.rpc.video.VideoService/ListVideos",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
    }
}
/// Generated client implementations.
pub mod frame_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct FrameServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FrameServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> FrameServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> FrameServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            FrameServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Lists the frames for a single video.
        pub async fn list_frames(
            &mut self,
            request: impl tonic::IntoRequest<super::ListFramesRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::super::super::video::FrameInfo>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mysticetus.rpc.video.FrameService/ListFrames",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
    }
}
/// Generated server implementations.
pub mod video_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with
    /// VideoServiceServer.
    #[async_trait]
    pub trait VideoService: Send + Sync + 'static {
        ///Server streaming response type for the GenerateFrames method.
        type GenerateFramesStream: futures_core::Stream<Item = Result<super::GenerateFramesResponse, tonic::Status>>
            + Send
            + 'static;
        /// Trigger the frame generation for a given video.
        ///
        /// Streams back progress updates, and the final message in the stream to
        /// indicate completion is the the updated 'VideoInfo', with 'frame_path' set.
        async fn generate_frames(
            &self,
            request: tonic::Request<super::super::super::video::VideoInfo>,
        ) -> Result<tonic::Response<Self::GenerateFramesStream>, tonic::Status>;
        /// Updates info about a single video. Merges the request with the existing
        /// data, and returns the merged result.
        async fn update_video_info(
            &self,
            request: tonic::Request<super::super::super::video::VideoInfo>,
        ) -> Result<tonic::Response<super::super::super::video::VideoInfo>, tonic::Status>;
        ///Server streaming response type for the ListVideos method.
        type ListVideosStream: futures_core::Stream<
                Item = Result<super::super::super::video::VideoInfo, tonic::Status>,
            > + Send
            + 'static;
        /// Returns a stream of video docs. Can filter/query the results via the
        /// request.
        async fn list_videos(
            &self,
            request: tonic::Request<super::ListVideosRequest>,
        ) -> Result<tonic::Response<Self::ListVideosStream>, tonic::Status>;
    }
    ///
    #[derive(Debug)]
    pub struct VideoServiceServer<T: VideoService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: VideoService> VideoServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for VideoServiceServer<T>
    where
        T: VideoService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/mysticetus.rpc.video.VideoService/GenerateFrames" => {
                    #[allow(non_camel_case_types)]
                    struct GenerateFramesSvc<T: VideoService>(pub Arc<T>);
                    impl<T: VideoService>
                        tonic::server::ServerStreamingService<super::super::super::video::VideoInfo>
                        for GenerateFramesSvc<T>
                    {
                        type Response = super::GenerateFramesResponse;
                        type ResponseStream = T::GenerateFramesStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::super::video::VideoInfo>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).generate_frames(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GenerateFramesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mysticetus.rpc.video.VideoService/UpdateVideoInfo" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateVideoInfoSvc<T: VideoService>(pub Arc<T>);
                    impl<T: VideoService>
                        tonic::server::UnaryService<super::super::super::video::VideoInfo>
                        for UpdateVideoInfoSvc<T>
                    {
                        type Response = super::super::super::video::VideoInfo;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::super::video::VideoInfo>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_video_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateVideoInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mysticetus.rpc.video.VideoService/ListVideos" => {
                    #[allow(non_camel_case_types)]
                    struct ListVideosSvc<T: VideoService>(pub Arc<T>);
                    impl<T: VideoService>
                        tonic::server::ServerStreamingService<super::ListVideosRequest>
                        for ListVideosSvc<T>
                    {
                        type Response = super::super::super::video::VideoInfo;
                        type ResponseStream = T::ListVideosStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListVideosRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_videos(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListVideosSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: VideoService> Clone for VideoServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: VideoService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: VideoService> tonic::server::NamedService for VideoServiceServer<T> {
        const NAME: &'static str = "mysticetus.rpc.video.VideoService";
    }
}
/// Generated server implementations.
pub mod frame_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with
    /// FrameServiceServer.
    #[async_trait]
    pub trait FrameService: Send + Sync + 'static {
        ///Server streaming response type for the ListFrames method.
        type ListFramesStream: futures_core::Stream<
                Item = Result<super::super::super::video::FrameInfo, tonic::Status>,
            > + Send
            + 'static;
        /// Lists the frames for a single video.
        async fn list_frames(
            &self,
            request: tonic::Request<super::ListFramesRequest>,
        ) -> Result<tonic::Response<Self::ListFramesStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct FrameServiceServer<T: FrameService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: FrameService> FrameServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for FrameServiceServer<T>
    where
        T: FrameService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/mysticetus.rpc.video.FrameService/ListFrames" => {
                    #[allow(non_camel_case_types)]
                    struct ListFramesSvc<T: FrameService>(pub Arc<T>);
                    impl<T: FrameService>
                        tonic::server::ServerStreamingService<super::ListFramesRequest>
                        for ListFramesSvc<T>
                    {
                        type Response = super::super::super::video::FrameInfo;
                        type ResponseStream = T::ListFramesStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListFramesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_frames(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListFramesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: FrameService> Clone for FrameServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: FrameService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: FrameService> tonic::server::NamedService for FrameServiceServer<T> {
        const NAME: &'static str = "mysticetus.rpc.video.FrameService";
    }
}
