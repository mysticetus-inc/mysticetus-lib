use std::borrow::Cow;
use std::future::Future;

use firestore_rs::{DocFields, PathComponent};
use timestamp::Timestamp;

use crate::driver::Driver;
use crate::report::{GcsLocation, ProgressReport};
use crate::states::NoResult;
use crate::{Failed, Finished, ProgressReporter, ReportState, Update};

pub struct ReportBuilder<'a, 'b, State: ReportState, ResultFile, CollecId: PathComponent> {
    reporter: crate::MaybeOwnedMut<'a, ProgressReporter<'b, CollecId>>,
    result_file: ResultFile,
    update: Option<ProgressUpdate>,
    timestamp: Option<Timestamp>,
    message: Option<Cow<'static, str>>,
    detail: Option<Cow<'static, str>>,
    state: State,
}

enum ProgressUpdate {
    Incr(usize),
    Absolute(usize),
    Finish,
}

impl<'a, 'b, State: ReportState, ResultFile, CollecId: PathComponent>
    ReportBuilder<'a, 'b, State, ResultFile, CollecId>
{
    pub(super) fn new(
        reporter: crate::MaybeOwnedMut<'a, ProgressReporter<'b, CollecId>>,
        result_file: ResultFile,
        state: State,
    ) -> Self {
        Self {
            result_file,
            reporter,
            state,
            timestamp: None,
            update: None,
            message: None,
            detail: None,
        }
    }
}

impl<'a, 'b, S: ReportState, CollecId> ReportBuilder<'a, 'b, S, S::ResultFile<'a>, CollecId>
where
    CollecId: PathComponent + Clone + Send + Sync + 'static,
{
    fn build_message(
        &mut self,
    ) -> (
        &mut Driver<CollecId>,
        ProgressReport<'_>,
        firestore_rs::Result<DocFields>,
    ) {
        match self.update {
            Some(ProgressUpdate::Incr(incr)) => {
                self.reporter.progress = self.reporter.progress.increment(incr);
            }
            Some(ProgressUpdate::Absolute(set)) => {
                self.reporter.progress = self.reporter.progress.update(set);
            }
            Some(ProgressUpdate::Finish) => {
                self.reporter.progress = self
                    .reporter
                    .progress
                    .update(self.reporter.progress.total());
            }
            None => (),
        }

        let started = match (self.timestamp, self.reporter.started) {
            (Some(ts), None) => {
                self.reporter.started = Some(ts);
                ts
            }
            (Some(this_ts), Some(started)) => started.min(this_ts),
            (None, Some(started)) => started,
            (None, None) => *self.reporter.started.insert(Timestamp::now()),
        };

        let message = ProgressReport {
            state: self.state.to_report_state(),
            uid: &self.reporter.uid,
            progress: self.reporter.progress,
            result_file: self.result_file.into(),
            started,
            last_updated: self.timestamp.unwrap_or_else(Timestamp::now),
            message: self.message.as_deref(),
            details: self.detail.as_deref(),
        };

        // serialize the document once, that way we only need to clone it if a request fails.
        let result = firestore_rs::DocFields::serialize_merge(&message);

        (&mut self.reporter.driver, message, result)
    }

    pub(crate) fn update_inner<'borr, O: 'borr>(
        &'borr mut self,
        send_fn: impl FnOnce(&'borr mut Driver<CollecId>, Timestamp, DocFields) -> O,
    ) -> (ProgressReport<'borr>, Result<O, firestore_rs::Error>) {
        let (driver, raw, message_result) = self.build_message();
        match message_result {
            Err(error) => (raw, Err(error)),
            Ok(message) => (raw, Ok(send_fn(driver, raw.last_updated, message))),
        }
    }

    pub fn update_with_result(mut self) -> firestore_rs::Result<()> {
        self.update_inner(|driver, ts, fields| driver.send(ts, fields))
            .1
    }

    pub fn update(mut self) {
        let (message, result) = self.update_inner(|driver, ts, fields| driver.send(ts, fields));
        if let Err(error) = result {
            tracing::error!(message = "failed to serialize progress report", report = ?message, ?error);
        }
    }
}

impl<'a, 'b, S: ReportState, ResultFile, CollecId: PathComponent>
    ReportBuilder<'a, 'b, S, ResultFile, CollecId>
{
    pub fn message<M>(mut self, message: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        self.message = Some(message.into());
        self
    }

    pub fn at_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn detail<D>(mut self, detail: D) -> Self
    where
        D: Into<Cow<'static, str>>,
    {
        self.detail = Some(detail.into());
        self
    }
}

impl<'a, 'b, CollecId: PathComponent> ReportBuilder<'a, 'b, Update, NoResult, CollecId> {
    pub fn increment_steps(mut self, steps: usize) -> Self {
        self.update = Some(ProgressUpdate::Incr(steps));
        self
    }

    pub fn increment_to_end(self) -> Self {
        let total = self.reporter.progress.total();
        self.set_progress(total)
    }

    pub fn set_progress(mut self, progress: usize) -> Self {
        self.update = Some(ProgressUpdate::Absolute(progress));
        self
    }
}

impl<'a, 'b, CollecId> ReportBuilder<'a, 'b, Finished, (), CollecId>
where
    CollecId: PathComponent + Clone + Send + Sync + 'static,
{
    pub fn finish(
        self,
        bucket: &'a str,
        path: &'a str,
    ) -> impl Future<Output = firestore_rs::Result<()>> + Send + 'static {
        let mut builder = ReportBuilder {
            result_file: GcsLocation::new(bucket, path),
            reporter: self.reporter,
            update: Some(ProgressUpdate::Finish),
            timestamp: self.timestamp,
            message: self.message,
            detail: self.detail,
            state: Finished::DEFAULT,
        };

        let (_, result) =
            builder.update_inner(|driver, ts, fields| driver.send_recv_result(ts, fields));

        async move {
            let rx = result?;

            match rx.await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(error)) => Err(error),
                Err(_) => Err(firestore_rs::Error::ListenerClosed),
            }
        }
    }
}

impl<'a, CollecId> ReportBuilder<'a, '_, Failed, Option<GcsLocation<'a>>, CollecId>
where
    CollecId: PathComponent + Clone + Send + Sync + 'static,
{
    pub fn with_result_file(mut self, bucket: &'a str, path: &'a str) -> Self {
        self.result_file = Some(GcsLocation::new(bucket, path));
        self
    }

    pub async fn finish(mut self) -> firestore_rs::Result<()> {
        self.update = Some(ProgressUpdate::Finish);
        let (_, result) = self.update_inner(|driver, update_ts, fields| {
            tracing::info!(
                message = "failure state encoded fields",
                ?fields,
                ?update_ts
            );
            driver.send_recv_result(update_ts, fields)
        });
        let rx = result?;
        let (reference, timestamp) = rx.await.unwrap()?;
        tracing::info!(message = "sent failure message", ?reference, ?timestamp);
        Ok(())
    }
}
