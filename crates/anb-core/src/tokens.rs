//! Token measurement for the Budget: a deterministic estimate, no tokenizer.
//!
//! The Core targets WASM and carries no dependencies, so it cannot ship a
//! BPE vocabulary; it estimates from byte counts instead. Calibrated against
//! `o200k_base` (via the gpt-tokenizer npm package) on Status-shaped
//! fixtures, 2026-08-28: structured dashboard text runs ~3.7
//! bytes per token, and dividing by 3.5 keeps the estimate at or above the
//! true count — within +6% on multi-line composites, up to ~+20% on a
//! single short line (the rounding weighs more), up to ~+50% on long
//! English prose. The overshoot is the chosen side: a Status that cuts a
//! row too many stays within Budget; one token over the ceiling does not.
//! Known limit: text denser than 3 bytes per token is undercounted — CJK
//! and long emoji sequences are the realistic cases.

/// The estimated `o200k_base` token count of `text`: ASCII bytes at 3.5 per
/// token, non-ASCII at 3 (6/21 and 7/21 in one integer rounding).
#[must_use]
pub fn estimate_tokens(text: &str) -> u32 {
    let ascii = text.bytes().filter(u8::is_ascii).count() as u64;
    let non_ascii = text.len() as u64 - ascii;
    u32::try_from((ascii * 6 + non_ascii * 7).div_ceil(21)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The calibration policy: an estimate lands at the truth or above —
    /// under the truth a Status could bust its Budget — and alarms past the
    /// per-shape ceiling, set above the worst overshoot the calibration run
    /// measured (structured lines +18%, prose +48%) and below a divisor
    /// drift worth investigating.
    const STRUCTURED_OVERSHOOT_PERCENT: u32 = 25;
    const PROSE_OVERSHOOT_PERCENT: u32 = 60;

    /// `truth` is the true `o200k_base` count of the exact string, measured
    /// 2026-08-28 with gpt-tokenizer 2.x.
    fn assert_calibrated(text: &str, truth: u32, overshoot_percent: u32) {
        let estimate = estimate_tokens(text);
        assert!(
            estimate >= truth,
            "estimate {estimate} under the true count {truth}"
        );
        let ceiling = truth + truth * overshoot_percent / 100;
        assert!(
            estimate <= ceiling,
            "estimate {estimate} over the calibrated ceiling {ceiling}"
        );
    }

    #[test]
    fn a_dashboard_row_lands_within_the_calibrated_band() {
        assert_calibrated(
            "  task.parser-budget-0,0,1d,Parser accepts budget handling in the fences path\n",
            21,
            STRUCTURED_OVERSHOOT_PERCENT,
        );
    }

    #[test]
    fn a_counts_line_lands_within_the_calibrated_band() {
        assert_calibrated(
            "ok: notebook quiet — 60 tasks, 20 decisions, 30 notes, 15 questions. anb --help when needed.\n",
            27,
            STRUCTURED_OVERSHOOT_PERCENT,
        );
    }

    #[test]
    fn a_debt_line_lands_within_the_calibrated_band() {
        assert_calibrated(
            "  question-age: question.escape-policy (16d)\n",
            11,
            STRUCTURED_OVERSHOOT_PERCENT,
        );
    }

    #[test]
    fn cyrillic_text_stays_at_or_above_the_truth() {
        assert_calibrated(
            "  task.fix-cyrillic-9,2,3d,Починить разбор заголовков в кириллице\n",
            29,
            STRUCTURED_OVERSHOOT_PERCENT,
        );
    }

    #[test]
    fn english_prose_overshoots_but_never_undershoots() {
        assert_calibrated(
            "The budget line reports what was spent and what was cut so the reader can restore the full dashboard with one command.\n",
            23,
            PROSE_OVERSHOOT_PERCENT,
        );
    }

    #[test]
    fn empty_text_costs_nothing() {
        assert_eq!(estimate_tokens(""), 0);
    }
}
