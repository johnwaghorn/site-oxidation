use super::queries::LIST_OUTAGES_OLDEST_FIRST_AFTER_OUTAGE_ID;
use super::responses::OutageResponse;
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use futures::stream::{self, Stream, StreamExt};
use sqlx::SqlitePool;

const OUTAGE_CSV_HEADER: &str =
    "started_at,ended_at,duration_seconds,http_status,expected_status,error_message\n";

const OUTAGES_READ_PER_CSV_CHUNK: u32 = 1000;

const CHARACTERS_A_SPREADSHEET_TREATS_AS_A_FORMULA_START: [char; 6] =
    ['=', '+', '-', '@', '\t', '\r'];

pub(super) fn site_outages_as_csv_read_in_chunks(
    pool: SqlitePool,
    site_id: i64,
) -> impl Stream<Item = Result<String, sqlx::Error>> {
    let csv_header = stream::once(async { Ok(OUTAGE_CSV_HEADER.to_owned()) });
    csv_header.chain(outage_csv_rows_read_in_chunks(pool, site_id))
}

pub(super) fn outage_csv_filename(site_name: &str, date_exported: NaiveDate) -> String {
    format!(
        "site-oxidation-outages-{}-{date_exported}.csv",
        site_name_safe_for_a_filename(site_name)
    )
}

fn site_name_safe_for_a_filename(site_name: &str) -> String {
    let words: Vec<&str> = site_name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        "site".to_owned()
    } else {
        words.join("-")
    }
}

fn outage_csv_rows_read_in_chunks(
    pool: SqlitePool,
    site_id: i64,
) -> impl Stream<Item = Result<String, sqlx::Error>> {
    stream::try_unfold(None::<i64>, move |id_of_last_outage_written| {
        let pool = pool.clone();
        async move {
            let outages =
                sqlx::query_as::<_, OutageResponse>(LIST_OUTAGES_OLDEST_FIRST_AFTER_OUTAGE_ID)
                    .bind(site_id)
                    .bind(id_of_last_outage_written)
                    .bind(OUTAGES_READ_PER_CSV_CHUNK)
                    .fetch_all(&pool)
                    .await
                    .inspect_err(|e| {
                        tracing::error!("Outage export for site {site_id} stopped early: {e}");
                    })?;
            let Some(last_outage_in_chunk) = outages.last() else {
                return Ok(None);
            };
            Ok(Some((
                outages_as_csv_rows(&outages),
                Some(last_outage_in_chunk.id),
            )))
        }
    })
}

fn outages_as_csv_rows(outages: &[OutageResponse]) -> String {
    outages.iter().map(outage_as_csv_row).collect()
}

fn outage_as_csv_row(outage: &OutageResponse) -> String {
    let duration_seconds = outage.ended_at.map(|ended_at| {
        ended_at
            .signed_duration_since(outage.started_at)
            .num_seconds()
    });
    let fields = [
        timestamp_in_utc(outage.started_at),
        outage.ended_at.map_or_default(timestamp_in_utc),
        number_or_empty(duration_seconds),
        number_or_empty(outage.http_status),
        number_or_empty(outage.expected_status),
        csv_text_field(outage.error_message.as_deref().unwrap_or_default()),
    ];
    let mut row = fields.join(",");
    row.push('\n');
    row
}

fn timestamp_in_utc(timestamp: DateTime<Utc>) -> String {
    timestamp.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn number_or_empty(number: Option<i64>) -> String {
    number.map_or_default(|number| number.to_string())
}

fn csv_text_field(text: &str) -> String {
    let starts_like_a_formula =
        text.starts_with(CHARACTERS_A_SPREADSHEET_TREATS_AS_A_FORMULA_START);
    let text_safe_to_open_in_a_spreadsheet = if starts_like_a_formula {
        format!("'{text}")
    } else {
        text.to_owned()
    };
    let needs_quotes = text_safe_to_open_in_a_spreadsheet.contains([',', '"', '\n', '\r']);
    if needs_quotes {
        format!(
            "\"{}\"",
            text_safe_to_open_in_a_spreadsheet.replace('"', "\"\"")
        )
    } else {
        text_safe_to_open_in_a_spreadsheet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_filename_keeps_only_letters_and_digits_of_the_site_name() {
        let date_exported = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        assert_eq!(
            outage_csv_filename("Waghorn Technology Ltd", date_exported),
            "site-oxidation-outages-Waghorn-Technology-Ltd-2026-11-15.csv"
        );
        assert_eq!(
            outage_csv_filename("../shop \"live\"; rm", date_exported),
            "site-oxidation-outages-shop-live-rm-2026-11-15.csv"
        );
        assert_eq!(
            outage_csv_filename("???", date_exported),
            "site-oxidation-outages-site-2026-11-15.csv"
        );
    }

    #[test]
    fn test_plain_text_is_left_as_it_is() {
        assert_eq!(csv_text_field("connection refused"), "connection refused");
    }

    #[test]
    fn test_text_with_commas_quotes_or_line_breaks_is_quoted() {
        assert_eq!(
            csv_text_field("Expected HTTP status 200, received 503"),
            "\"Expected HTTP status 200, received 503\""
        );
        assert_eq!(
            csv_text_field("said \"no\"\nthen closed"),
            "\"said \"\"no\"\"\nthen closed\""
        );
    }

    #[test]
    fn test_text_a_spreadsheet_would_run_as_a_formula_is_defused() {
        assert_eq!(
            csv_text_field("=HYPERLINK(\"https://evil.waghorn.tech\")"),
            "\"'=HYPERLINK(\"\"https://evil.waghorn.tech\"\")\""
        );
        assert_eq!(csv_text_field("-1 retries left"), "'-1 retries left");
    }

    #[test]
    fn test_ongoing_outage_has_no_end_or_duration() {
        let outage = OutageResponse {
            id: 1,
            site_id: 1,
            http_status: None,
            expected_status: Some(200),
            started_at: Utc.with_ymd_and_hms(2026, 11, 15, 9, 30, 0).unwrap(),
            ended_at: None,
            error_message: Some("connection refused".to_owned()),
        };
        assert_eq!(
            outages_as_csv_rows(&[outage]),
            "2026-11-15T09:30:00Z,,,,200,connection refused\n"
        );
    }
}
