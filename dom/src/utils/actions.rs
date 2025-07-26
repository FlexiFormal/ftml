use leptos::prelude::{Get, GetUntracked, ReadSignal, RwSignal, Set, WriteSignal};

#[derive(Copy, Clone, Debug)]
pub struct OneShot {
    click: WriteSignal<bool>,
    done: ReadSignal<bool>,
}
impl OneShot {
    pub(crate) fn new() -> (Self, SetOneShotDone) {
        let click = RwSignal::new(false);
        let done_sig = RwSignal::new(false);
        let done = SetOneShotDone {
            was_set: click.read_only(),
            is_done: done_sig.write_only(),
        };
        let os = Self {
            click: click.write_only(),
            done: done_sig.read_only(),
        };
        (os, done)
    }
    #[inline]
    pub fn activate(self) {
        if !self.is_done_untracked() {
            self.click.set(true);
        }
    }
    #[inline]
    pub fn is_done(self) -> bool {
        self.done.get()
    }
    #[inline]
    pub fn is_done_untracked(self) -> bool {
        self.done.get_untracked()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("oneshot can't be set done, if it wasn't activated before")]
pub struct OneShotNotSet;

#[derive(Copy, Clone, Debug)]
pub struct SetOneShotDone {
    was_set: ReadSignal<bool>,
    is_done: WriteSignal<bool>,
}
impl SetOneShotDone {
    pub fn was_clicked(self) -> bool {
        self.was_set.get()
    }
    pub fn was_clicked_untracked(self) -> bool {
        self.was_set.get_untracked()
    }
    /// ### Errors
    #[inline]
    pub fn set(self) -> Result<(), OneShotNotSet> {
        if self.was_set.get_untracked() {
            self.is_done.set(true);
            Ok(())
        } else {
            Err(OneShotNotSet)
        }
    }
}
