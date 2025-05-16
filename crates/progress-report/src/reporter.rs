use firestore_rs::{Firestore, PathComponent};
use timestamp::Timestamp;
use uuid::Uuid;

use crate::driver::Driver;
use crate::states::NoResult;
use crate::{Failed, Finished, GcsLocation, Progress, ReportBuilder, ReportState, Update};

pub struct ProgressReporter<'a, CollecId: PathComponent = &'static str> {
    pub(crate) task_id: Uuid,
    pub(crate) uid: &'a str,
    pub(crate) driver: Driver<CollecId>,
    pub(crate) progress: Progress,
    pub(crate) started: Option<Timestamp>,
}

impl<'a, CollecId: PathComponent + Clone + Sync + Send + 'static> ProgressReporter<'a, CollecId> {
    pub fn builder(
        collection_id: CollecId,
    ) -> crate::builder::ProgressReporterBuilder<CollecId, (), (), (), ()> {
        crate::builder::ProgressReporterBuilder::collection(collection_id)
    }

    pub(crate) fn from_firestore(
        firestore: &Firestore,
        collec_id: CollecId,
        task_id: Uuid,
        initial_progress: Progress,
        uid: &'a str,
    ) -> Self {
        let doc_ref = firestore.collection(collec_id).doc(task_id);

        Self {
            uid,
            task_id,
            driver: Driver::new(doc_ref),
            progress: initial_progress,
            started: None,
        }
    }

    pub fn task_id(&self) -> Uuid {
        self.task_id
    }
    pub fn build_update(&mut self) -> ReportBuilder<'_, 'a, Update, NoResult, CollecId> {
        ReportBuilder::new(
            crate::MaybeOwnedMut::MutRef(self),
            NoResult,
            Update::DEFAULT,
        )
    }

    pub fn build_finish<'b>(self) -> ReportBuilder<'b, 'a, Finished, (), CollecId> {
        ReportBuilder::new(crate::MaybeOwnedMut::Owned(self), (), Finished::DEFAULT)
    }

    pub fn build_error<'b>(
        self,
    ) -> ReportBuilder<'b, 'a, Failed, Option<GcsLocation<'a>>, CollecId> {
        ReportBuilder::new(crate::MaybeOwnedMut::Owned(self), None, Failed::DEFAULT)
    }
}
