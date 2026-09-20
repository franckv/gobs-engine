use std::collections::HashMap;

use gobs_render_hal::{BarrierAccess, BarrierStage, BarrierSyncScope, ImageLayout};

pub struct SyncStatus {
    last_write: BarrierSyncScope,
    last_layout: ImageLayout,
    flushed: bool,
    invalidates: HashMap<BarrierStage, BarrierAccess>,
}

impl SyncStatus {
    pub fn new(scope: BarrierSyncScope, layout: ImageLayout) -> Self {
        let mut status = Self {
            last_write: scope.writes_only(),
            last_layout: layout,
            flushed: true,
            invalidates: HashMap::new(),
        };

        status.update(scope, layout);

        status
    }

    pub fn last_write(&self) -> BarrierSyncScope {
        self.last_write
    }

    pub fn last_layout(&self) -> ImageLayout {
        self.last_layout
    }

    pub fn update(&mut self, scope: BarrierSyncScope, layout: ImageLayout) {
        self.last_layout = layout;

        if scope.access.is_write() {
            self.last_write = scope.writes_only();
            self.flushed = false;
            self.invalidates.clear();
        } else {
            self.flushed = true;

            let access = scope.access.reads();
            for stage in scope.stage {
                self.invalidates
                    .entry(stage)
                    .and_modify(|a| *a |= access)
                    .or_insert(access);
            }
        }
    }

    pub fn is_invalidated(&self, scope: BarrierSyncScope) -> bool {
        let access = scope.access.reads();

        if access.is_empty() {
            return false;
        }

        let mut result = false;
        for stage in scope.stage {
            match self.invalidates.get(&stage) {
                Some(a) if a.contains(access) => {
                    result = true;
                }
                _ => {
                    result = false;
                    break;
                }
            }
        }

        result
    }

    pub fn is_flushed(&self) -> bool {
        self.flushed
    }
}
