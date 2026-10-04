//! Coalesce redraws without sleeping in the event handler or polling an idle GPU.

use std::time::{Duration, Instant};

const FOREGROUND_FRAME: Duration = Duration::from_nanos(16_666_667);
const BACKGROUND_FRAME: Duration = Duration::from_millis(100);

#[derive(Default)]
pub(crate) struct Redraw {
    last_frame: Option<Instant>,
    pending: bool,
    repaint_at: Option<Instant>,
}

impl Redraw {
    /// Mark work without immediately waking the window again. Timers use this
    /// so an overdue hover/drag cannot spin on rejected OS redraw events.
    pub(crate) fn request(&mut self) {
        self.pending = true;
    }

    /// An OS redraw may arrive for every PTY wake or pointer movement. Keep
    /// the newest state pending until a frame is due; never discard a redraw.
    pub(crate) fn begin(&mut self, now: Instant, focused: bool, drawable: bool) -> bool {
        self.request();
        if !self
            .deadline(now, focused, drawable)
            .is_some_and(|at| at <= now)
        {
            return false;
        }
        self.pending = false;
        self.repaint_at = None;
        self.last_frame = Some(now);
        true
    }

    /// egui's delay is relative to the start of its frame. Both immediate
    /// widget changes and delayed animations pass through the same frame cap.
    pub(crate) fn repaint_after(&mut self, frame_start: Instant, delay: Duration) {
        self.repaint_at = frame_start.checked_add(delay);
    }

    /// No deadline when idle or hidden: a 60 FPS cap is not a 60 Hz timer.
    pub(crate) fn deadline(&self, now: Instant, focused: bool, drawable: bool) -> Option<Instant> {
        if !drawable {
            return None;
        }
        let requested = if self.pending { now } else { self.repaint_at? };
        let interval = if focused {
            FOREGROUND_FRAME
        } else {
            BACKGROUND_FRAME
        };
        Some(
            self.last_frame
                .map_or(requested, |last| requested.max(last + interval)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_burst_is_coalesced_and_the_last_update_is_not_lost() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(redraw.begin(now, true, true));
        assert_eq!(redraw.deadline(now, true, true), None);
        for ms in 1..=16 {
            let at = now + Duration::from_millis(ms);
            assert!(!redraw.begin(at, true, true));
            assert_eq!(
                redraw.deadline(at, true, true),
                Some(now + FOREGROUND_FRAME)
            );
        }
        assert!(redraw.begin(now + FOREGROUND_FRAME, true, true));
        assert_eq!(redraw.deadline(now + FOREGROUND_FRAME, true, true), None);
    }

    #[test]
    fn background_work_is_throttled_but_focus_restores_responsiveness() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(redraw.begin(now, false, true));
        let at = now + Duration::from_millis(20);
        assert!(!redraw.begin(at, false, true));
        assert_eq!(
            redraw.deadline(at, false, true),
            Some(now + BACKGROUND_FRAME)
        );
        assert!(redraw.begin(at, true, true));
    }

    #[test]
    fn hidden_window_keeps_work_without_a_render_timer() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(!redraw.begin(now, true, false));
        assert_eq!(redraw.deadline(now, true, false), None);
        assert_eq!(redraw.deadline(now, true, true), Some(now));
        assert!(redraw.begin(now, true, true));
        assert_eq!(redraw.deadline(now, true, true), None);
    }

    #[test]
    fn delayed_egui_animation_wakes_at_its_deadline_and_is_replaced_by_new_frame() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(redraw.begin(now, true, true));
        redraw.repaint_after(now, Duration::from_millis(50));
        assert_eq!(
            redraw.deadline(now, true, true),
            Some(now + Duration::from_millis(50))
        );
        assert!(redraw.begin(now + Duration::from_millis(30), true, true));
        redraw.repaint_after(now + Duration::from_millis(30), Duration::MAX);
        assert_eq!(redraw.deadline(now, true, true), None);
    }

    #[test]
    fn immediate_egui_repaints_cannot_spin_and_hidden_animations_do_not_wake() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(redraw.begin(now, true, true));
        redraw.repaint_after(now, Duration::ZERO);
        assert_eq!(
            redraw.deadline(now, true, true),
            Some(now + FOREGROUND_FRAME)
        );
        assert_eq!(
            redraw.deadline(now, false, true),
            Some(now + BACKGROUND_FRAME)
        );
        assert_eq!(redraw.deadline(now, true, false), None);
    }

    #[test]
    fn sustained_output_respects_both_frame_budgets() {
        for (focused, maximum) in [(true, 601), (false, 101)] {
            let start = Instant::now();
            let mut redraw = Redraw::default();
            let frames = (0..=10_000)
                .filter(|ms| redraw.begin(start + Duration::from_millis(*ms), focused, true))
                .count();
            assert!(frames <= maximum, "rendered {frames} frames in ten seconds");
            let at = start + Duration::from_secs(11);
            assert!(redraw.begin(at, focused, true));
            assert_eq!(redraw.deadline(at, focused, true), None);
        }
    }

    #[test]
    fn overdue_timer_requests_wait_for_the_frame_budget() {
        let now = Instant::now();
        let mut redraw = Redraw::default();
        assert!(redraw.begin(now, true, true));
        for ms in 1..=16 {
            redraw.request();
            assert_eq!(
                redraw.deadline(now + Duration::from_millis(ms), true, true),
                Some(now + FOREGROUND_FRAME)
            );
        }
        assert!(redraw.begin(now + FOREGROUND_FRAME, true, true));
        assert_eq!(redraw.deadline(now + FOREGROUND_FRAME, true, true), None);
    }
}
