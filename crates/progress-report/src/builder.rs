use std::borrow::Cow;
use std::future::Future;

use firestore_rs::{Firestore, PathComponent};
use uuid::Uuid;

use crate::states::NoResult;
use crate::{ReportBuilder, ReportState, Update};

pub struct ProgressReporterBuilder<C = &'static str, F = (), U = (), T = (), P = ()> {
    // required
    collec_id: C,
    firestore: F,
    uid: U,
    task_id: T,
    max_progress: P,

    // optional
    init_progress: usize,
}

impl<C> ProgressReporterBuilder<C> {
    pub fn collection(collection: C) -> Self {
        Self {
            collec_id: collection,
            max_progress: (),
            uid: (),
            firestore: (),
            task_id: (),
            init_progress: 0,
        }
    }
}

// optional arg functions
impl<S, F, U, T, P> ProgressReporterBuilder<S, F, U, T, P> {
    pub fn initial_progress(mut self, initial_prog: usize) -> Self {
        self.init_progress = initial_prog;
        self
    }
}

mod fs {
    pub struct Fs<'a>(pub(super) &'a firestore_rs::Firestore);
}

impl<C: PathComponent> ProgressReporterBuilder<C> {
    pub fn firestore(
        self,
        firestore: &Firestore,
    ) -> ProgressReporterBuilder<C, fs::Fs<'_>, (), (), ()> {
        ProgressReporterBuilder {
            collec_id: self.collec_id,
            firestore: fs::Fs(firestore),
            uid: self.uid,
            task_id: self.task_id,
            max_progress: self.max_progress,
            init_progress: self.init_progress,
        }
    }

    pub fn firestore_fut<'a, F>(self, fut: F) -> ProgressReporterBuilder<C, F>
    where
        F: Future<Output = &'a Firestore>,
    {
        ProgressReporterBuilder {
            uid: self.uid,
            collec_id: self.collec_id,
            firestore: fut,
            task_id: self.task_id,
            max_progress: self.max_progress,
            init_progress: self.init_progress,
        }
    }
}

pub trait IntoFirestore<'a> {}

impl<'a> IntoFirestore<'a> for fs::Fs<'a> {}
impl<'a, F> IntoFirestore<'a> for F where F: Future<Output = &'a Firestore> {}

impl<'a, C, F> ProgressReporterBuilder<C, F>
where
    F: IntoFirestore<'a>,
    C: PathComponent,
{
    pub fn uid<'u>(self, uid: &'u str) -> ProgressReporterBuilder<C, F, &'u str> {
        ProgressReporterBuilder {
            collec_id: self.collec_id,
            firestore: self.firestore,
            uid,
            task_id: self.task_id,
            max_progress: self.max_progress,
            init_progress: self.init_progress,
        }
    }
}

impl<'a, 'u, F, C> ProgressReporterBuilder<C, F, &'u str>
where
    F: IntoFirestore<'a>,
    C: PathComponent,
{
    pub fn task_id(self, task_id: Uuid) -> ProgressReporterBuilder<C, F, &'u str, Uuid> {
        ProgressReporterBuilder {
            collec_id: self.collec_id,
            uid: self.uid,
            firestore: self.firestore,
            task_id,
            max_progress: self.max_progress,
            init_progress: self.init_progress,
        }
    }
}

impl<'a, 'u, F, C> ProgressReporterBuilder<C, F, &'u str, Uuid>
where
    F: IntoFirestore<'a>,
    C: PathComponent,
{
    pub fn max_progress(
        self,
        max_progress: usize,
    ) -> ProgressReporterBuilder<C, F, &'u str, Uuid, usize> {
        if max_progress == 0 {
            panic!("max_progress can't be 0");
        }

        ProgressReporterBuilder {
            collec_id: self.collec_id,
            firestore: self.firestore,
            task_id: self.task_id,
            uid: self.uid,
            max_progress,
            init_progress: self.init_progress,
        }
    }
}

impl<'a, 'u, C, F> ProgressReporterBuilder<C, F, &'u str, Uuid, usize>
where
    C: PathComponent + Clone + Send + Sync + 'static,
    F: Future<Output = &'a Firestore>,
{
    pub async fn initialize<M>(
        self,
        initial_message: M,
    ) -> firestore_rs::Result<super::ProgressReporter<'u, C>>
    where
        M: Into<Cow<'static, str>>,
    {
        async fn inner<'b, 'u, C2, F2>(
            builder: ProgressReporterBuilder<C2, F2, &'u str, Uuid, usize>,
            message: Cow<'static, str>,
        ) -> firestore_rs::Result<super::ProgressReporter<'u, C2>>
        where
            C2: PathComponent + Clone + Send + Sync + 'static,
            F2: Future<Output = &'b Firestore>,
        {
            let initial_progress =
                super::Progress::new(builder.init_progress, builder.max_progress);

            let mut new = super::ProgressReporter::from_firestore(
                builder.firestore.await,
                builder.collec_id,
                builder.task_id,
                initial_progress,
                builder.uid,
            );

            // create the initial report
            ReportBuilder::new(
                crate::MaybeOwnedMut::MutRef(&mut new),
                NoResult,
                Update::DEFAULT,
            )
            .message(message)
            .update();

            Ok(new)
        }

        inner(self, initial_message.into()).await
    }
}

impl<'a, 'u, C> ProgressReporterBuilder<C, fs::Fs<'a>, &'u str, Uuid, usize>
where
    C: PathComponent + Clone + Send + Sync + 'static,
{
    pub fn initialize<M>(
        self,
        initial_message: M,
    ) -> firestore_rs::Result<super::ProgressReporter<'u, C>>
    where
        M: Into<Cow<'static, str>>,
    {
        fn inner<'u, C2>(
            builder: ProgressReporterBuilder<C2, fs::Fs<'_>, &'u str, Uuid, usize>,
            message: Cow<'static, str>,
        ) -> firestore_rs::Result<super::ProgressReporter<'u, C2>>
        where
            C2: PathComponent + Clone + Send + Sync + 'static,
        {
            let initial_progress =
                super::Progress::new(builder.init_progress, builder.max_progress);

            let mut new = super::ProgressReporter::from_firestore(
                builder.firestore.0,
                builder.collec_id,
                builder.task_id,
                initial_progress,
                builder.uid,
            );

            // create the initial report
            ReportBuilder::new(
                crate::MaybeOwnedMut::MutRef(&mut new),
                NoResult,
                Update::DEFAULT,
            )
            .message(message)
            .update();

            Ok(new)
        }

        inner(self, initial_message.into())
    }
}
