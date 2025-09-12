use bytes::Bytes;
use gcp_auth_provider::service::AuthSvc;
use net_utils::header::GoogRequestParam;
use protos::tasks;
use timestamp::Timestamp;
use tonic::transport::Channel;

use crate::http::HttpRequestBuilder;

#[derive(Debug)]
pub struct TaskQueueClient {
    queue: Box<str>,
    channel: AuthSvc<Channel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId(Box<str>);

impl TaskQueueClient {
    pub(crate) fn new(channel: AuthSvc<Channel>, queue: Box<str>) -> Self {
        Self { queue, channel }
    }

    async fn create_task_inner(
        &self,
        task: tasks::Task,
        view: tasks::task::View,
    ) -> crate::Result<tasks::Task> {
        let request = tasks::CreateTaskRequest {
            parent: String::from(self.queue.as_ref()),
            task: Some(task),
            response_view: view as i32,
        };

        let header_bytes = Bytes::from(format!("parent={}", self.queue));
        let metadata = http::HeaderValue::from_maybe_shared(header_bytes)
            .expect("this should always be valid");

        let mut client = tasks::cloud_tasks_client::CloudTasksClient::new(GoogRequestParam::new(
            self.channel.clone(),
            metadata,
        ));

        let task = client.create_task(request).await?.into_inner();
        Ok(task)
    }

    pub async fn create_task(&self, request: HttpRequestBuilder) -> crate::Result<TaskInfo> {
        let task = tasks::Task {
            name: String::new(),
            schedule_time: None,
            create_time: None,
            dispatch_deadline: None,
            dispatch_count: 0,
            response_count: 0,
            last_attempt: None,
            first_attempt: None,
            message_type: Some(tasks::task::MessageType::HttpRequest(request.into_proto())),
            view: tasks::task::View::Basic as i32,
        };

        let task = self
            .create_task_inner(task, tasks::task::View::Basic)
            .await?;

        TaskInfo::from_proto(task)
    }
}

impl TaskInfo {
    fn from_proto(task: tasks::Task) -> crate::Result<Self> {
        let create_time = task
            .create_time
            .ok_or_else(|| crate::Error::missing_proto_field::<tasks::Task>("create_time"))?
            .into();

        let schedule_time = task
            .schedule_time
            .ok_or_else(|| crate::Error::missing_proto_field::<tasks::Task>("schedule_time"))?
            .into();

        let first_attempt = task.first_attempt.map(Attempt::from_proto).transpose()?;
        let last_attempt = task.last_attempt.map(Attempt::from_proto).transpose()?;

        Ok(Self {
            id: TaskId(task.name.into_boxed_str()),
            create_time,
            schedule_time,
            dispatch_count: task.dispatch_count as u32,
            response_count: task.response_count as u32,
            first_attempt,
            last_attempt,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: TaskId,
    pub create_time: Timestamp,
    pub schedule_time: Timestamp,
    pub dispatch_count: u32,
    pub response_count: u32,
    pub first_attempt: Option<Attempt>,
    pub last_attempt: Option<Attempt>,
}

#[derive(Debug, Clone)]
pub struct Attempt {
    pub schedule_time: Timestamp,
    pub dispatch_time: Timestamp,
    pub response_time: Option<Timestamp>,
    pub response_status: Option<tonic::Status>,
}

impl Attempt {
    fn from_proto(proto: tasks::Attempt) -> crate::Result<Self> {
        let schedule_time = proto
            .schedule_time
            .ok_or_else(|| crate::Error::missing_proto_field::<tasks::Attempt>("schedule_time"))?
            .into();

        let dispatch_time = proto
            .dispatch_time
            .ok_or_else(|| crate::Error::missing_proto_field::<tasks::Attempt>("dispatch_time"))?
            .into();

        let response_time = proto.response_time.map(Into::into);

        let response_status = proto
            .response_status
            .map(|status| tonic::Status::new(tonic::Code::from_i32(status.code), status.message));

        Ok(Self {
            schedule_time,
            dispatch_time,
            response_time,
            response_status,
        })
    }
}
