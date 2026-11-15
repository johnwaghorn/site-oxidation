#![allow(clippy::needless_for_each)]

use super::endpoints;
use super::responses::{NotificationOutboxEntryResponse, NotificationOutboxEntryStatus};
use crate::api::errors::ApiError;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::list_notification_outbox_entries,
        endpoints::cancel_notification_outbox_entry,
    ),
    components(schemas(
        NotificationOutboxEntryResponse,
        NotificationOutboxEntryStatus,
        ApiError,
    )),
    tags(
        (name = "admin/notification-outbox", description = "Notifications waiting to be sent (admin only)"),
    ),
)]
pub struct NotificationOutboxApiDoc;
