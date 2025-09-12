use crate::LogOptions;
use crate::registry::Records;

/// Handle to a constructed [Subscriber].
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Handle {
    pub(super) records: &'static Records,
}

impl Handle {
    pub fn set_project_id(&self, project_id: &'static str) -> Result<(), ()> {
        self.records.project_id.set(project_id).map_err(|_| ())
    }

    pub fn get_or_init_project_id(
        &self,
        get_project_id: impl FnOnce() -> &'static str,
    ) -> &'static str {
        self.records.project_id.get_or_init(get_project_id)
    }

    pub fn set_options(&self, opts: impl LogOptions + 'static) {
        *self.records.options.write() = Box::new(opts);
    }

    pub fn set_stage(&self, stage: crate::Stage) -> Result<(), crate::Stage> {
        self.records.stage.set(stage)
    }

    pub fn get_stage(&self) -> Option<crate::Stage> {
        self.records.stage.get().copied()
    }

    pub fn get_or_detect_stage(&self) -> crate::Stage {
        *self.records.stage.get_or_init(crate::Stage::default)
    }
}
