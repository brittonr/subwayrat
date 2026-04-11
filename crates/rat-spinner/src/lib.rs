//! Reusable spinner presets and animation state.

use core::time::Duration;

const DEFAULT_INTERVAL_MS: u64 = 80;
const DOTS_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const LINE_FRAMES: &[&str] = &["-", "\\", "|", "/"];
const PULSE_FRAMES: &[&str] = &["∙∙∙", "●∙∙", "∙●∙", "∙∙●"];
const ARROW_FRAMES: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];
const BOUNCE_FRAMES: &[&str] = &["⠁", "⠂", "⠄", "⠂"];

#[derive(Debug, Clone)]
pub struct SpinnerSpec<'a> {
    frames: SpinnerFrames<'a>,
    interval: Duration,
    reversed: bool,
}

#[derive(Debug, Clone)]
pub enum SpinnerFrames<'a> {
    Preset(SpinnerPreset),
    Custom(&'a [&'a str]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerPreset {
    Dots,
    Line,
    Pulse,
    Arrow,
    Bounce,
}

#[derive(Debug, Clone, Default)]
pub struct SpinnerState {
    frame: usize,
    elapsed_ms: u128,
}

impl<'a> SpinnerSpec<'a> {
    pub fn new(preset: SpinnerPreset) -> Self {
        Self {
            frames: SpinnerFrames::Preset(preset),
            interval: preset.default_interval(),
            reversed: false,
        }
    }

    pub fn dots() -> Self {
        Self::new(SpinnerPreset::Dots)
    }

    pub fn line() -> Self {
        Self::new(SpinnerPreset::Line)
    }

    pub fn pulse() -> Self {
        Self::new(SpinnerPreset::Pulse)
    }

    pub fn arrow() -> Self {
        Self::new(SpinnerPreset::Arrow)
    }

    pub fn bounce() -> Self {
        Self::new(SpinnerPreset::Bounce)
    }

    pub fn custom(frames: &'a [&'a str]) -> Self {
        Self {
            frames: SpinnerFrames::Custom(frames),
            interval: Duration::from_millis(DEFAULT_INTERVAL_MS),
            reversed: false,
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = normalize_interval(interval);
        self
    }

    pub fn with_reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub fn label(&self) -> &'static str {
        match &self.frames {
            SpinnerFrames::Preset(preset) => preset.label(),
            SpinnerFrames::Custom(_) => "custom",
        }
    }

    pub fn frames(&self) -> &'a [&'a str] {
        self.frames.as_slice()
    }
}

impl<'a> SpinnerFrames<'a> {
    fn as_slice(&self) -> &'a [&'a str] {
        match self {
            Self::Preset(preset) => preset.frames(),
            Self::Custom(frames) => frames,
        }
    }
}

impl SpinnerPreset {
    pub fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Dots => DOTS_FRAMES,
            Self::Line => LINE_FRAMES,
            Self::Pulse => PULSE_FRAMES,
            Self::Arrow => ARROW_FRAMES,
            Self::Bounce => BOUNCE_FRAMES,
        }
    }

    pub fn default_interval(self) -> Duration {
        match self {
            Self::Dots => Duration::from_millis(80),
            Self::Line => Duration::from_millis(120),
            Self::Pulse => Duration::from_millis(100),
            Self::Arrow => Duration::from_millis(90),
            Self::Bounce => Duration::from_millis(110),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dots => "dots",
            Self::Line => "line",
            Self::Pulse => "pulse",
            Self::Arrow => "arrow",
            Self::Bounce => "bounce",
        }
    }
}

impl SpinnerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, spinner: &SpinnerSpec<'_>) {
        let len = spinner.frames().len();
        if len == 0 {
            return;
        }
        self.frame = (self.frame + 1) % len;
    }

    pub fn advance(&mut self, spinner: &SpinnerSpec<'_>, delta: Duration) {
        let len = spinner.frames().len();
        if len == 0 {
            return;
        }

        let interval_ms = spinner.interval().as_millis().max(1);
        self.elapsed_ms += delta.as_millis();
        let steps = self.elapsed_ms / interval_ms;
        self.elapsed_ms %= interval_ms;

        if steps > 0 {
            self.frame = (self.frame + (steps as usize % len)) % len;
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.elapsed_ms = 0;
    }

    pub fn current_frame<'a>(&self, spinner: &'a SpinnerSpec<'a>) -> &'a str {
        let frames = spinner.frames();
        if frames.is_empty() {
            return "";
        }

        let idx = if spinner.is_reversed() {
            frames.len() - 1 - (self.frame % frames.len())
        } else {
            self.frame % frames.len()
        };
        frames[idx]
    }
}

fn normalize_interval(interval: Duration) -> Duration {
    Duration::from_millis(interval.as_millis().max(1).min(u64::MAX as u128) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_defaults_match_expected_intervals() {
        let dots = SpinnerSpec::dots();
        let pulse = SpinnerSpec::pulse();

        assert_eq!(dots.interval(), Duration::from_millis(80));
        assert_eq!(pulse.interval(), Duration::from_millis(100));
        assert_eq!(dots.frames()[0], "⠋");
        assert_eq!(pulse.frames()[0], "∙∙∙");
    }

    #[test]
    fn tick_cycles_custom_frames() {
        let frames = &["a", "b", "c"];
        let spec = SpinnerSpec::custom(frames);
        let mut state = SpinnerState::new();

        assert_eq!(state.current_frame(&spec), "a");
        state.tick(&spec);
        assert_eq!(state.current_frame(&spec), "b");
        state.tick(&spec);
        assert_eq!(state.current_frame(&spec), "c");
        state.tick(&spec);
        assert_eq!(state.current_frame(&spec), "a");
    }

    #[test]
    fn reverse_reads_frames_from_end() {
        let frames = &["a", "b", "c"];
        let spec = SpinnerSpec::custom(frames).with_reversed(true);
        let mut state = SpinnerState::new();

        assert_eq!(state.current_frame(&spec), "c");
        state.tick(&spec);
        assert_eq!(state.current_frame(&spec), "b");
        state.tick(&spec);
        assert_eq!(state.current_frame(&spec), "a");
    }

    #[test]
    fn advance_respects_interval() {
        let spec = SpinnerSpec::line().with_interval(Duration::from_millis(100));
        let mut state = SpinnerState::new();

        state.advance(&spec, Duration::from_millis(99));
        assert_eq!(state.current_frame(&spec), "-");
        state.advance(&spec, Duration::from_millis(1));
        assert_eq!(state.current_frame(&spec), "\\");
        state.advance(&spec, Duration::from_millis(250));
        assert_eq!(state.current_frame(&spec), "/");
    }

    #[test]
    fn empty_custom_spinner_is_safe() {
        let spec = SpinnerSpec::custom(&[]);
        let mut state = SpinnerState::new();

        state.tick(&spec);
        state.advance(&spec, Duration::from_millis(10));
        assert_eq!(state.current_frame(&spec), "");
    }
}
