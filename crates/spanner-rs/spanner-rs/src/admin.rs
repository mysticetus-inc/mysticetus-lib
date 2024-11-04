//! [`SpannerAdmin`], a wrapper around both 'google.spanner.admin.instance' and
//! 'google.spanner.admin.database' APIs.

use std::collections::HashMap;

use bytes::Bytes;
use gcp_auth_channel::{AuthChannel, Scope};
use longrunning::OperationHandle;
use protos::spanner::admin::database::database_admin_client::DatabaseAdminClient;
use protos::spanner::admin::database::{
    self, CreateDatabaseMetadata, CreateDatabaseRequest, DatabaseDialect, GetDatabaseDdlRequest,
};
use protos::spanner::admin::instance::instance_admin_client::InstanceAdminClient;
use protos::spanner::admin::instance::{self, CreateInstanceMetadata, CreateInstanceRequest};
use timestamp::Duration;

use crate::client::Client;
use crate::info::{Database, Instance};

const ADMIN_SCOPE: Scope = Scope::SpannerAdmin;

#[derive(Clone)]
pub struct SpannerAdmin {
    channel: AuthChannel,
}

pub enum InstanceCompute {
    Nodes(u16),
    ProcessingUnits(u16),
}

impl InstanceCompute {
    fn nodes(&self) -> i32 {
        match self {
            Self::Nodes(nodes) => *nodes as i32,
            _ => 0,
        }
    }

    fn processing_units(&self) -> i32 {
        match self {
            Self::ProcessingUnits(units) => *units as i32,
            _ => 0,
        }
    }
}

impl SpannerAdmin {
    pub(crate) fn from_channel(channel: AuthChannel) -> Self {
        Self {
            channel: channel.with_scope(Scope::SpannerAdmin),
        }
    }

    pub(crate) fn into_channel(self) -> AuthChannel {
        self.channel
    }

    pub fn from_client(client: &Client) -> Self {
        Self::from_channel(client.channel.clone())
    }

    fn db_client(&self) -> DatabaseAdminClient<AuthChannel> {
        DatabaseAdminClient::new(self.channel.clone())
    }

    fn instance_client(&self) -> InstanceAdminClient<AuthChannel> {
        InstanceAdminClient::new(self.channel.clone())
    }

    pub async fn build_new_spanner_stack(
        &self,
        database: &Database,
        compute: InstanceCompute,
        extra_statements: Vec<String>,
        timeout: Option<Duration>,
    ) -> crate::Result<(instance::Instance, database::Database)> {
        let instance_handle = self
            .create_instance(&database.as_instance_builder(), compute)
            .await?;

        let instance = instance_handle.wait(timeout).await?;

        let database_handle = self.create_database(database, extra_statements).await?;

        let database = database_handle.wait(timeout).await?;

        Ok((instance, database))
    }

    pub async fn create_database(
        &self,
        database: &Database,
        extra_statements: Vec<String>,
    ) -> crate::Result<OperationHandle<CreateDatabaseMetadata, database::Database>> {
        let req = CreateDatabaseRequest {
            proto_descriptors: Bytes::new(),
            parent: database.qualified_instance().to_owned(),
            create_statement: format!("CREATE DATABASE `{}`", database.database()),
            extra_statements,
            encryption_config: None,
            database_dialect: DatabaseDialect::GoogleStandardSql as i32,
        };

        let mut channel = self.channel.clone();
        let mut client = DatabaseAdminClient::new(&mut channel);

        let operation = client.create_database(req).await?.into_inner();

        Ok(OperationHandle::from_channel(channel, operation))
    }

    pub async fn create_instance<I: AsRef<str>>(
        &self,
        instance: &Instance<I>,
        compute: InstanceCompute,
    ) -> crate::Result<OperationHandle<CreateInstanceMetadata, instance::Instance>> {
        let req = CreateInstanceRequest {
            parent: instance.as_project().build_qualified(),
            instance_id: instance.instance().as_ref().to_owned(),
            instance: Some(instance::Instance {
                name: String::new(),
                edition: protos::spanner::admin::instance::instance::Edition::Standard as i32,
                autoscaling_config: None,
                config: String::new(),
                display_name: String::new(),
                node_count: compute.nodes(),
                processing_units: compute.processing_units(),
                state: 0,
                labels: HashMap::new(),
                replica_compute_capacity: vec![],
                default_backup_schedule_type:
                    instance::instance::DefaultBackupScheduleType::Unspecified as i32,
                endpoint_uris: vec![],
                create_time: None,
                update_time: None,
            }),
        };

        let mut channel = self.channel.clone();

        let operation = InstanceAdminClient::new(&mut channel)
            .create_instance(req)
            .await?
            .into_inner();

        Ok(OperationHandle::from_channel(channel, operation))
    }

    pub async fn get_database_ddl<S: AsRef<str>>(
        &mut self,
        db: &Database<S>,
    ) -> crate::Result<Vec<String>> {
        let req = GetDatabaseDdlRequest {
            database: db.qualified_database().to_owned(),
        };

        let resp = self.db_client().get_database_ddl(req).await?.into_inner();

        Ok(resp.statements)
    }

    pub async fn update_database_ddl<S: AsRef<str>>(
        &mut self,
        db: &Database<S>,
        statements: Vec<String>,
        operation_id: Option<String>,
        proto_descriptors: Option<Bytes>,
    ) -> crate::Result<longrunning::OperationHandle<database::UpdateDatabaseDdlMetadata>> {
        assert!(
            !statements.is_empty(),
            "no point in updating database ddl when no ddl is specified"
        );

        let req = database::UpdateDatabaseDdlRequest {
            database: db.qualified_database().to_owned(),
            statements,
            operation_id: operation_id.unwrap_or_default(),
            proto_descriptors: proto_descriptors.unwrap_or_default(),
        };

        let update_operation = self
            .db_client()
            .update_database_ddl(req)
            .await?
            .into_inner();

        Ok(longrunning::OperationHandle::from_channel(
            self.channel.clone(),
            update_operation,
        ))
    }
}
