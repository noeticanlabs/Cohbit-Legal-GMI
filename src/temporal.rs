//! Exact, fixed-offset temporal model used by LEGAL-PROOF-001.
//!
//! Calendar dates are proleptic Gregorian dates. Instants are signed Unix
//! seconds. Local date-times always carry an explicit fixed UTC offset; named
//! timezone rules are deliberately not inferred. Effective periods are
//! half-open `[start, end)` intervals.
use anyhow::{bail, ensure, Context, Result};
use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CivilDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WallTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalDateTime {
    pub date: CivilDate,
    pub time: WallTime,
    #[serde(rename = "offsetSeconds")]
    pub offset_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EffectivePeriod {
    #[serde(rename = "startInstant")]
    pub start_instant: i64,
    #[serde(rename = "endExclusiveInstant")]
    pub end_exclusive_instant: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExtensionNotice {
    #[serde(rename = "sourceId")]
    pub source_id: String,
    pub effective: EffectivePeriod,
    pub replacement: LocalDateTime,
    #[serde(rename = "replacementInstant")]
    pub replacement_instant: i64,
    pub priority: u32,
}

/// The fields ending in `Day`, `Second`, or `Instant` are explicit calculation
/// steps, not trusted conclusions. Rust replays every step and Lean checks the
/// normalized arithmetic before accepting the Horn proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TemporalCertificate {
    pub publication: CivilDate,
    #[serde(rename = "publicationDay")]
    pub publication_day: i64,
    #[serde(rename = "calendarDays")]
    pub calendar_days: u32,
    #[serde(rename = "startDayIncluded")]
    pub start_day_included: bool,
    #[serde(rename = "targetDay")]
    pub target_day: i64,
    #[serde(rename = "closeTime")]
    pub close_time: WallTime,
    #[serde(rename = "closeSecond")]
    pub close_second: u32,
    #[serde(rename = "offsetSeconds")]
    pub offset_seconds: i32,
    #[serde(rename = "localSecond")]
    pub local_second: i64,
    #[serde(rename = "baseInstant")]
    pub base_instant: i64,
    #[serde(rename = "verificationInstant")]
    pub verification_instant: i64,
    #[serde(rename = "baseEvidenceSource")]
    pub base_evidence_source: String,
    pub extensions: Vec<ExtensionNotice>,
    #[serde(rename = "selectedExtension")]
    pub selected_extension: Option<usize>,
    #[serde(rename = "conclusionLocal")]
    pub conclusion_local: LocalDateTime,
    #[serde(rename = "conclusionInstant")]
    pub conclusion_instant: i64,
}

fn date(value: &CivilDate) -> Result<NaiveDate> {
    NaiveDate::from_ymd_opt(value.year, value.month, value.day).context("Invalid civil date")
}

fn day_index(value: &CivilDate) -> Result<i64> {
    Ok(date(value)?
        .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
        .num_days())
}

fn second_of_day(value: &WallTime) -> Result<u32> {
    ensure!(
        value.hour < 24 && value.minute < 60 && value.second < 60,
        "Invalid wall time"
    );
    Ok(value.hour * 3600 + value.minute * 60 + value.second)
}

pub fn local_to_instant(value: &LocalDateTime) -> Result<i64> {
    ensure!(
        value.offset_seconds.unsigned_abs() < 86_400,
        "UTC offset must be less than 24 hours"
    );
    let wall_second = second_of_day(&value.time)? as i64;
    day_index(&value.date)?
        .checked_mul(86_400)
        .and_then(|v| v.checked_add(wall_second))
        .and_then(|v| v.checked_sub(value.offset_seconds as i64))
        .context("Temporal arithmetic overflow")
}

impl EffectivePeriod {
    pub fn contains(&self, instant: i64) -> bool {
        self.start_instant <= instant && instant < self.end_exclusive_instant
    }
}

impl TemporalCertificate {
    pub fn validate(&self, sources: &BTreeMap<String, crate::proof::SourceEvidence>) -> Result<()> {
        ensure!(
            sources.contains_key(&self.base_evidence_source),
            "Missing base temporal evidence"
        );
        ensure!(
            self.publication_day == day_index(&self.publication)?,
            "Publication day step is wrong"
        );
        let added = if self.start_day_included {
            self.calendar_days
                .checked_sub(1)
                .context("An inclusive zero-day period is invalid")?
        } else {
            self.calendar_days
        };
        let expected_date = date(&self.publication)?
            .checked_add_days(Days::new(added as u64))
            .context("Calendar-day addition overflow")?;
        let expected_day = self
            .publication_day
            .checked_add(added as i64)
            .context("Day overflow")?;
        ensure!(
            self.target_day == expected_day,
            "Calendar-day addition step is wrong"
        );
        ensure!(
            expected_date.num_days_from_ce() as i64 - 719_163 == expected_day,
            "Civil and normalized target dates disagree"
        );
        ensure!(
            self.close_second == second_of_day(&self.close_time)?,
            "Close-time step is wrong"
        );
        ensure!(
            self.offset_seconds.unsigned_abs() < 86_400,
            "UTC offset must be less than 24 hours"
        );
        let local = self
            .target_day
            .checked_mul(86_400)
            .and_then(|v| v.checked_add(self.close_second as i64))
            .context("Local-second overflow")?;
        ensure!(self.local_second == local, "Local-second step is wrong");
        ensure!(
            self.base_instant == local - self.offset_seconds as i64,
            "Offset conversion step is wrong"
        );

        let active: Vec<usize> = self
            .extensions
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.effective.contains(self.verification_instant).then_some(i))
            .collect();
        for extension in &self.extensions {
            ensure!(
                sources.contains_key(&extension.source_id),
                "Missing extension evidence"
            );
            ensure!(
                extension.effective.start_instant < extension.effective.end_exclusive_instant,
                "Empty extension effective period"
            );
            ensure!(
                extension.replacement_instant == local_to_instant(&extension.replacement)?,
                "Extension offset calculation is wrong"
            );
        }
        let expected = if active.is_empty() {
            ensure!(
                self.selected_extension.is_none(),
                "Inactive extension selected"
            );
            self.base_instant
        } else {
            let max = active
                .iter()
                .map(|i| self.extensions[*i].priority)
                .max()
                .unwrap();
            let winners: Vec<_> = active
                .into_iter()
                .filter(|i| self.extensions[*i].priority == max)
                .collect();
            let first = self.extensions[winners[0]].replacement_instant;
            if winners
                .iter()
                .any(|i| self.extensions[*i].replacement_instant != first)
            {
                bail!("Conflicting equal-priority extension notices");
            }
            let selected = self
                .selected_extension
                .context("Active extension was not selected")?;
            ensure!(
                winners.contains(&selected),
                "Selected extension does not control"
            );
            first
        };
        ensure!(
            self.conclusion_instant == expected,
            "Temporal conclusion is not the calculated deadline"
        );
        ensure!(
            local_to_instant(&self.conclusion_local)? == expected,
            "Conclusion local time and instant disagree"
        );
        Ok(())
    }

    pub fn canonical_claim_text(&self) -> String {
        let d = &self.conclusion_local.date;
        let t = &self.conclusion_local.time;
        let offset = self.conclusion_local.offset_seconds;
        let sign = if offset < 0 { '-' } else { '+' };
        let absolute = offset.unsigned_abs();
        format!("The fictional public-comment deadline is {:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}.",
            d.year, d.month, d.day, t.hour, t.minute, t.second, sign, absolute / 3600, (absolute % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::SourceEvidence;

    fn source() -> SourceEvidence {
        SourceEvidence {
            text: "fixture".into(),
            publisher: "fixture".into(),
            official_uri: "fixture://time".into(),
            retrieved_at: "2030-01-01T00:00:00Z".into(),
            acquisition_record: "fixture".into(),
            revision: "1".into(),
            status_record: "current".into(),
            status_checked_at: "2030-01-01T00:00:00Z".into(),
            amendment_history: vec![],
        }
    }

    fn base() -> TemporalCertificate {
        TemporalCertificate {
            publication: CivilDate {
                year: 2030,
                month: 1,
                day: 1,
            },
            publication_day: 21_915,
            calendar_days: 30,
            start_day_included: false,
            target_day: 21_945,
            close_time: WallTime {
                hour: 17,
                minute: 0,
                second: 0,
            },
            close_second: 61_200,
            offset_seconds: -21_600,
            local_second: 1_896_109_200,
            base_instant: 1_896_130_800,
            verification_instant: 1_894_730_400,
            base_evidence_source: "notice".into(),
            extensions: vec![],
            selected_extension: None,
            conclusion_local: LocalDateTime {
                date: CivilDate {
                    year: 2030,
                    month: 1,
                    day: 31,
                },
                time: WallTime {
                    hour: 17,
                    minute: 0,
                    second: 0,
                },
                offset_seconds: -21_600,
            },
            conclusion_instant: 1_896_130_800,
        }
    }

    #[test]
    fn rejects_altered_dates_and_timezone_steps() {
        let sources = BTreeMap::from([("notice".into(), source())]);
        assert!(base().validate(&sources).is_ok());
        let mut altered = base();
        altered.publication.day = 2;
        assert!(altered.validate(&sources).is_err());
        let mut zone = base();
        zone.offset_seconds = -18_000;
        assert!(zone.validate(&sources).is_err());
    }

    #[test]
    fn extension_requires_evidence_and_conflicts_fail_closed() {
        let mut sources = BTreeMap::from([
            ("notice".into(), source()),
            ("extension-a".into(), source()),
        ]);
        let replacement = LocalDateTime {
            date: CivilDate {
                year: 2030,
                month: 2,
                day: 7,
            },
            time: WallTime {
                hour: 17,
                minute: 0,
                second: 0,
            },
            offset_seconds: -21_600,
        };
        let replacement_instant = local_to_instant(&replacement).unwrap();
        let mut certificate = base();
        certificate.extensions.push(ExtensionNotice {
            source_id: "extension-a".into(),
            effective: EffectivePeriod {
                start_instant: 1_894_000_000,
                end_exclusive_instant: 1_895_000_000,
            },
            replacement: replacement.clone(),
            replacement_instant,
            priority: 10,
        });
        certificate.selected_extension = Some(0);
        certificate.conclusion_local = replacement;
        certificate.conclusion_instant = replacement_instant;
        assert!(certificate.validate(&sources).is_ok());
        sources.remove("extension-a");
        assert!(certificate.validate(&sources).is_err());
        sources.insert("extension-a".into(), source());
        sources.insert("extension-b".into(), source());
        let other = LocalDateTime {
            date: CivilDate {
                year: 2030,
                month: 2,
                day: 8,
            },
            time: WallTime {
                hour: 17,
                minute: 0,
                second: 0,
            },
            offset_seconds: -21_600,
        };
        certificate.extensions.push(ExtensionNotice {
            source_id: "extension-b".into(),
            effective: EffectivePeriod {
                start_instant: 1_894_000_000,
                end_exclusive_instant: 1_895_000_000,
            },
            replacement_instant: local_to_instant(&other).unwrap(),
            replacement: other,
            priority: 10,
        });
        assert!(certificate.validate(&sources).is_err());
    }
}
