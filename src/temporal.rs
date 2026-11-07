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

/// Extension input stops at authenticated notice data. Its instant is derived.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInput {
    pub source_id: String,
    pub effective: EffectivePeriod,
    pub replacement: LocalDateTime,
    pub priority: u32,
}

/// Inputs to deadline derivation. No deadline or intermediate result is accepted
/// from the caller.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TemporalDerivationInput {
    pub base_evidence_source: String,
    pub publication: CivilDate,
    pub calendar_days: u32,
    pub start_day_included: bool,
    pub close_time: WallTime,
    pub offset_seconds: i32,
    pub verification_instant: i64,
    pub extensions: Vec<ExtensionInput>,
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
    /// Derive all calculation steps and the conclusion. The API deliberately
    /// has no conclusion argument.
    pub fn derive(
        input: TemporalDerivationInput,
        sources: &BTreeMap<String, crate::proof::SourceEvidence>,
    ) -> Result<Self> {
        ensure!(
            sources.contains_key(&input.base_evidence_source),
            "Missing base temporal evidence"
        );
        let publication_day = day_index(&input.publication)?;
        let added = if input.start_day_included {
            input
                .calendar_days
                .checked_sub(1)
                .context("An inclusive zero-day period is invalid")?
        } else {
            input.calendar_days
        };
        let target_date = date(&input.publication)?
            .checked_add_days(Days::new(added as u64))
            .context("Calendar-day addition overflow")?;
        let target_day = publication_day
            .checked_add(added as i64)
            .context("Day overflow")?;
        let close_second = second_of_day(&input.close_time)?;
        ensure!(
            input.offset_seconds.unsigned_abs() < 86_400,
            "UTC offset must be less than 24 hours"
        );
        let local_second = target_day
            .checked_mul(86_400)
            .and_then(|v| v.checked_add(close_second as i64))
            .context("Local-second overflow")?;
        let base_instant = local_second
            .checked_sub(input.offset_seconds as i64)
            .context("Instant overflow")?;
        let extensions: Vec<ExtensionNotice> = input
            .extensions
            .into_iter()
            .map(|extension| {
                ensure!(
                    sources.contains_key(&extension.source_id),
                    "Missing extension evidence"
                );
                Ok(ExtensionNotice {
                    replacement_instant: local_to_instant(&extension.replacement)?,
                    source_id: extension.source_id,
                    effective: extension.effective,
                    replacement: extension.replacement,
                    priority: extension.priority,
                })
            })
            .collect::<Result<_>>()?;
        let active: Vec<usize> = extensions
            .iter()
            .enumerate()
            .filter_map(|(i, extension)| {
                extension
                    .effective
                    .contains(input.verification_instant)
                    .then_some(i)
            })
            .collect();
        let selected_extension = if active.is_empty() {
            None
        } else {
            let max = active
                .iter()
                .map(|i| extensions[*i].priority)
                .max()
                .unwrap();
            let winners: Vec<_> = active
                .into_iter()
                .filter(|i| extensions[*i].priority == max)
                .collect();
            let first = extensions[winners[0]].replacement_instant;
            ensure!(
                winners
                    .iter()
                    .all(|i| extensions[*i].replacement_instant == first),
                "Conflicting equal-priority extension notices"
            );
            Some(winners[0])
        };
        let (conclusion_local, conclusion_instant) = match selected_extension {
            Some(index) => (
                extensions[index].replacement.clone(),
                extensions[index].replacement_instant,
            ),
            None => (
                LocalDateTime {
                    date: CivilDate {
                        year: target_date.year(),
                        month: target_date.month(),
                        day: target_date.day(),
                    },
                    time: input.close_time.clone(),
                    offset_seconds: input.offset_seconds,
                },
                base_instant,
            ),
        };
        let certificate = Self {
            publication: input.publication,
            publication_day,
            calendar_days: input.calendar_days,
            start_day_included: input.start_day_included,
            target_day,
            close_time: input.close_time,
            close_second,
            offset_seconds: input.offset_seconds,
            local_second,
            base_instant,
            verification_instant: input.verification_instant,
            base_evidence_source: input.base_evidence_source,
            extensions,
            selected_extension,
            conclusion_local,
            conclusion_instant,
        };
        certificate.validate(sources)?;
        Ok(certificate)
    }

    /// Parse the deliberately narrow fixture notice grammar, then derive the
    /// deadline. This binds calculation inputs to exact authenticated bytes.
    pub fn derive_from_notice_evidence(
        source_id: &str,
        sources: &BTreeMap<String, crate::proof::SourceEvidence>,
        verification_instant: i64,
        extensions: Vec<ExtensionInput>,
    ) -> Result<Self> {
        let text = &sources
            .get(source_id)
            .context("Notice evidence source is absent")?
            .text;
        let publication_marker = "Publication is ";
        let publication_start = text
            .find(publication_marker)
            .context("Notice has no publication field")?
            + publication_marker.len();
        let publication_text = text
            .get(publication_start..publication_start + 10)
            .context("Malformed publication date")?;
        let publication_date = NaiveDate::parse_from_str(publication_text, "%Y-%m-%d")?;
        let count_marker = "Comments close after ";
        let count_start =
            text.find(count_marker).context("Notice has no day count")? + count_marker.len();
        let count_end = text[count_start..]
            .find(" calendar days at ")
            .context("Malformed calendar-day clause")?
            + count_start;
        let calendar_days: u32 = text[count_start..count_end].parse()?;
        let time_start = count_end + " calendar days at ".len();
        let time_text = text
            .get(time_start..time_start + 5)
            .context("Malformed close time")?;
        let mut time_parts = time_text.split(':');
        let close_time = WallTime {
            hour: time_parts.next().context("Missing hour")?.parse()?,
            minute: time_parts.next().context("Missing minute")?.parse()?,
            second: 0,
        };
        let offset_text = text
            .get(time_start + 6..time_start + 15)
            .context("Malformed UTC offset")?;
        ensure!(
            offset_text.starts_with("UTC") && offset_text.len() == 9,
            "Notice offset must be UTC±HH:MM"
        );
        let sign = match &offset_text[3..4] {
            "+" => 1,
            "-" => -1,
            _ => bail!("Notice UTC offset has no sign"),
        };
        let offset_seconds = sign
            * (offset_text[4..6].parse::<i32>()? * 3600 + offset_text[7..9].parse::<i32>()? * 60);
        Self::derive(
            TemporalDerivationInput {
                base_evidence_source: source_id.into(),
                publication: CivilDate {
                    year: publication_date.year(),
                    month: publication_date.month(),
                    day: publication_date.day(),
                },
                calendar_days,
                start_day_included: false,
                close_time,
                offset_seconds,
                verification_instant,
                extensions,
            },
            sources,
        )
    }

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

    fn notice(text: &str) -> SourceEvidence {
        SourceEvidence {
            text: text.into(),
            ..source()
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
    fn derives_substantive_deadline_from_notice_without_conclusion_input() {
        let notice_text = "FICTIONAL NOTICE: Publication is 2030-01-01. Comments close after 30 calendar days at 17:00 UTC-06:00.";
        let sources = BTreeMap::from([("notice".into(), notice(notice_text))]);
        let derived = TemporalCertificate::derive_from_notice_evidence(
            "notice",
            &sources,
            1_894_730_400,
            vec![],
        )
        .unwrap();
        assert_eq!(derived.publication_day, 21_915);
        assert_eq!(derived.target_day, 21_945);
        assert_eq!(derived.local_second, 1_896_109_200);
        assert_eq!(derived.conclusion_instant, 1_896_130_800);
        assert_eq!(
            derived.canonical_claim_text(),
            "The fictional public-comment deadline is 2030-01-31T17:00:00-06:00."
        );

        // Changing only authenticated input changes the generated conclusion;
        // no expected deadline is accepted by the derivation API.
        let changed_sources = BTreeMap::from([(
            "notice".into(),
            notice(&notice_text.replace("2030-01-01", "2030-01-02")),
        )]);
        let changed = TemporalCertificate::derive_from_notice_evidence(
            "notice",
            &changed_sources,
            1_894_730_400,
            vec![],
        )
        .unwrap();
        assert_eq!(
            changed.canonical_claim_text(),
            "The fictional public-comment deadline is 2030-02-01T17:00:00-06:00."
        );
        assert_ne!(changed.conclusion_instant, derived.conclusion_instant);
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
