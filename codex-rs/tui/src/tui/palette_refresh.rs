//! Bounded OSC 10/11 refreshes, answered through the ordinary terminal event reader.

use std::io;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

use crate::terminal_probe::DefaultColors;

const TIMEOUT: Duration = Duration::from_millis(250);
const QUERY: &[u8] = b"\x1b]10;?\x1b\\\x1b]11;?\x1b\\";

/// Subscribe only while Codex owns terminal input, including across suspend and editor handoff.
pub(super) fn set_reporting(output: &mut impl Write, enabled: bool) -> io::Result<()> {
    output.write_all(if enabled {
        b"\x1b[?2031h"
    } else {
        b"\x1b[?2031l"
    })?;
    output.flush()
}

#[derive(Default)]
pub(super) struct PaletteRefresh {
    pending: Option<Pending>,
}

struct Pending {
    deadline: Instant,
    foreground: Option<(u8, u8, u8)>,
    background: Option<(u8, u8, u8)>,
}

impl PaletteRefresh {
    pub(super) fn request(&mut self, output: &mut impl Write, now: Instant) -> io::Result<()> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| now < pending.deadline)
        {
            return Ok(());
        }
        self.pending = None;
        output.write_all(QUERY)?;
        output.flush()?;
        self.pending = Some(Pending {
            deadline: now + TIMEOUT,
            foreground: None,
            background: None,
        });
        Ok(())
    }

    pub(super) fn cancel(&mut self) {
        self.pending = None;
    }

    pub(super) fn accept(
        &mut self,
        slot: u8,
        rgb: Option<(u8, u8, u8)>,
        now: Instant,
    ) -> Option<DefaultColors> {
        let pending = self.pending.as_mut()?;
        if now >= pending.deadline || rgb.is_none() {
            self.cancel();
            return None;
        }
        match slot {
            10 => pending.foreground = rgb,
            11 => pending.background = rgb,
            _ => return None,
        }
        let colors = DefaultColors {
            fg: pending.foreground?,
            bg: pending.background?,
        };
        self.cancel();
        Some(colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn refreshes_both_directions_and_coalesces_outstanding_requests() {
        let mut refresh = PaletteRefresh::default();
        let now = Instant::now();
        for (fg, bg) in [((0, 0, 0), (255, 255, 255)), ((255, 255, 255), (0, 0, 0))] {
            let mut output = Vec::new();
            refresh.request(&mut output, now).unwrap();
            refresh.request(&mut output, now).unwrap();
            assert_eq!(output, QUERY);
            assert!(refresh.accept(11, Some(bg), now).is_none());
            let colors = refresh.accept(10, Some(fg), now).unwrap();
            assert_eq!((colors.fg, colors.bg), (fg, bg));
            assert!(refresh.accept(10, Some(fg), now).is_none());
        }
    }

    #[test]
    fn missing_partial_and_invalid_replies_allow_a_later_retry() {
        let mut refresh = PaletteRefresh::default();
        let now = Instant::now();
        let fg = (220, 220, 220);
        let bg = (20, 20, 20);
        refresh.request(&mut Vec::new(), now).unwrap();
        assert!(refresh.accept(10, Some(fg), now).is_none());
        assert!(refresh.accept(11, Some(bg), now + TIMEOUT).is_none());
        refresh.request(&mut Vec::new(), now + TIMEOUT).unwrap();
        assert!(refresh.accept(10, None, now + TIMEOUT).is_none());
        assert!(refresh.accept(11, Some(bg), now + TIMEOUT).is_none());
        refresh.request(&mut Vec::new(), now + TIMEOUT).unwrap();
        assert!(refresh.accept(11, Some(bg), now + TIMEOUT).is_none());
        let colors = refresh.accept(10, Some(fg), now + TIMEOUT).unwrap();
        assert_eq!((colors.fg, colors.bg), (fg, bg));
    }

    #[test]
    fn reporting_is_enabled_and_disabled_and_write_failure_can_retry() {
        let mut output = Vec::new();
        set_reporting(&mut output, /*enabled*/ true).unwrap();
        set_reporting(&mut output, /*enabled*/ false).unwrap();
        assert_eq!(output, b"\x1b[?2031h\x1b[?2031l");
        let mut refresh = PaletteRefresh::default();
        let now = Instant::now();
        assert!(refresh.request(&mut &mut [][..], now).is_err());
        let mut query = Vec::new();
        refresh.request(&mut query, now).unwrap();
        assert_eq!(query, QUERY);
    }

    #[test]
    fn terminal_handoff_cancels_the_outstanding_pair() {
        let mut refresh = PaletteRefresh::default();
        let now = Instant::now();
        refresh.request(&mut Vec::new(), now).unwrap();
        assert!(refresh.accept(10, Some((255, 255, 255)), now).is_none());
        refresh.cancel();
        assert!(refresh.accept(11, Some((0, 0, 0)), now).is_none());
    }
}
