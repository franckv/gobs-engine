use std::collections::{HashMap, hash_map::Entry};

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
            last_write: Self::filter_writes(scope),
            last_layout: layout,
            // false if first access is a write
            flushed: scope.access.writes().is_empty(),
            invalidates: HashMap::new(),
        };

        status.invalidate(scope);

        status
    }

    fn filter_writes(mut scope: BarrierSyncScope) -> BarrierSyncScope {
        scope.access = scope.access.writes();

        scope
    }

    pub fn last_write(&self) -> BarrierSyncScope {
        self.last_write
    }

    pub fn last_layout(&self) -> ImageLayout {
        self.last_layout
    }

    pub fn update(&mut self, scope: BarrierSyncScope, layout: ImageLayout) {
        self.last_write = Self::filter_writes(scope);
        self.last_layout = layout;
        // false if current update is a write
        self.flushed = scope.access.writes().is_empty();
    }

    pub fn flush(&mut self) {
        self.flushed = true;
    }

    pub fn invalidate(&mut self, scope: BarrierSyncScope) {
        let access = scope.access.reads();

        if access.is_empty() {
            return;
        }

        for stage in scope.stage {
            match self.invalidates.entry(stage) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() |= access;
                }
                Entry::Vacant(e) => {
                    e.insert(access);
                }
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

    pub fn clear_invalidates(&mut self) {
        self.invalidates.clear();
    }
}
