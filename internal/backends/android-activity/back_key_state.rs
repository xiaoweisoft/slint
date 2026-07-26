#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackKey {
    Back,
    ButtonB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyPhase {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BackKeyState {
    back_pressed: bool,
    button_b_pressed: bool,
}

impl BackKeyState {
    /// Records a key edge and returns whether an orphan back-like release needs
    /// a synthetic press immediately before it.
    pub(crate) fn observe(&mut self, key: Option<BackKey>, phase: KeyPhase) -> bool {
        let Some(pressed) = key.map(|key| match key {
            BackKey::Back => &mut self.back_pressed,
            BackKey::ButtonB => &mut self.button_b_pressed,
        }) else {
            return false;
        };

        match phase {
            KeyPhase::Pressed => {
                *pressed = true;
                false
            }
            KeyPhase::Released => !core::mem::replace(pressed, false),
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{BackKey, BackKeyState, KeyPhase};

    #[test]
    fn observed_back_down_up_does_not_synthesize_a_press() {
        let mut state = BackKeyState::default();

        assert!(!state.observe(Some(BackKey::Back), KeyPhase::Pressed));
        assert!(!state.observe(Some(BackKey::Back), KeyPhase::Released));
    }

    #[test]
    fn orphan_back_like_releases_synthesize_exactly_one_press() {
        for key in [BackKey::Back, BackKey::ButtonB] {
            let mut state = BackKeyState::default();

            assert!(state.observe(Some(key), KeyPhase::Released));
            assert!(state.observe(Some(key), KeyPhase::Released));
        }
    }

    #[test]
    fn orphan_other_key_release_is_unchanged() {
        let mut state = BackKeyState::default();

        assert!(!state.observe(None, KeyPhase::Released));
    }

    #[test]
    fn reset_forgets_observed_back_like_down_edges() {
        let mut state = BackKeyState::default();
        state.observe(Some(BackKey::Back), KeyPhase::Pressed);
        state.observe(Some(BackKey::ButtonB), KeyPhase::Pressed);

        state.reset();

        assert!(state.observe(Some(BackKey::Back), KeyPhase::Released));
        assert!(state.observe(Some(BackKey::ButtonB), KeyPhase::Released));
    }
}
