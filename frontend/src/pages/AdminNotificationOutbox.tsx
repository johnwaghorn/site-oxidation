import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  useAdminNotificationOutbox,
  useCancelNotificationOutboxEntry,
} from "../hooks/useAdminNotificationOutbox";
import { usePagination } from "../hooks/usePagination";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Pagination } from "../components/ui/Pagination";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import type { components } from "../generated/schema";
import { usePageTitle } from "../hooks/usePageTitle";

type OutboxEntry = components["schemas"]["NotificationOutboxEntryResponse"];

interface OutboxRowProps {
  entry: OutboxEntry;
  isOpen: boolean;
  onToggle: () => void;
  onCancel: () => void;
}

const COLUMN_COUNT = 8;

function describeEntry(entry: OutboxEntry) {
  if (entry.summary) return entry.summary;
  const storedNotificationIsUnreadable = !entry.provider;
  return storedNotificationIsUnreadable
    ? "Notification that could not be read"
    : "Notification queued before summaries were kept";
}

function describeDestination(entry: OutboxEntry) {
  const channel = `via ${entry.provider}`;
  return entry.team ? `${channel} to ${entry.team}` : channel;
}

function describeRetries(entry: OutboxEntry) {
  const nextTry = formatDateTime(entry.next_attempt_at);
  if (entry.status === "queued") {
    return `Not tried yet. First try ${nextTry}.`;
  }
  const timesFailed = `${entry.attempts} ${entry.attempts === 1 ? "time" : "times"}`;
  return `Failed ${timesFailed}. Next try ${nextTry}.`;
}

function formatDateTime(timestamp: string) {
  return new Date(timestamp).toLocaleString(undefined, {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function OutboxRow({ entry, isOpen, onToggle, onCancel }: OutboxRowProps) {
  const summary = describeEntry(entry);
  const detailId = `outbox-detail-${entry.id}`;
  return (
    <>
      <tr
        className={`table-row outbox-row${isOpen ? " is-open" : ""}`}
        onClick={onToggle}
      >
        <td>
          <button
            type="button"
            className="outbox-toggle"
            aria-expanded={isOpen}
            aria-controls={detailId}
            aria-label={`${isOpen ? "Hide" : "Show"} details for ${summary}`}
            onClick={(event) => {
              event.stopPropagation();
              onToggle();
            }}
          >
            <svg
              aria-hidden="true"
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M6 3l5 5-5 5" />
            </svg>
          </button>
        </td>
        <td className="outbox-truncate" title={summary}>
          <span
            role="img"
            aria-label={entry.status === "failing" ? "Failing" : "Queued"}
            className={`outbox-status-dot is-${entry.status}`}
          />
          {summary}
        </td>
        <td className="outbox-truncate" title={entry.team ?? undefined}>
          {entry.team ?? "-"}
        </td>
        <td className="outbox-truncate">{entry.provider ?? "-"}</td>
        <td>{entry.attempts}</td>
        <td
          className="outbox-truncate muted-text"
          title={entry.last_error ?? undefined}
        >
          {entry.last_error ?? "Waiting for its first try"}
        </td>
        <td
          className="outbox-truncate muted-text"
          title={new Date(entry.created_at).toLocaleString()}
        >
          {formatDateTime(entry.created_at)}
        </td>
        <td>
          {entry.status === "failing" && (
            <button
              type="button"
              className="button-table-action button-table-danger compact-input"
              aria-haspopup="dialog"
              onClick={(event) => {
                event.stopPropagation();
                onCancel();
              }}
            >
              Cancel
            </button>
          )}
        </td>
      </tr>
      {isOpen && (
        <tr id={detailId} className="table-row outbox-detail-row">
          <td />
          <td colSpan={COLUMN_COUNT - 1}>
            <div className="outbox-detail">
              {entry.last_error && (
                <div className="outbox-detail-error">
                  <span className="outbox-detail-label">Last error</span>
                  <div className="outbox-error-text">{entry.last_error}</div>
                </div>
              )}
              <div className="outbox-detail-footer">
                <span className="muted-text">{describeRetries(entry)}</span>
                {entry.team_id != null && (
                  <Link
                    className="button-secondary-action"
                    to={`/notifications/${entry.team_id}`}
                  >
                    Notification settings for {entry.team ?? "this team"}
                  </Link>
                )}
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

export function AdminNotificationOutbox() {
  usePageTitle("Notification outbox");
  const { page, perPage, goToPage, correctPage } = usePagination();
  const {
    data: outbox,
    isLoading,
    error,
  } = useAdminNotificationOutbox(page, perPage);
  const cancelEntry = useCancelNotificationOutboxEntry();
  const [entryToCancel, setEntryToCancel] = useState<OutboxEntry | null>(null);
  const [openEntryId, setOpenEntryId] = useState<number | null>(null);

  const entries = outbox?.data ?? [];
  const totalPages = outbox ? Math.ceil(outbox.total / outbox.per_page) : 0;

  useEffect(() => {
    const pageHasEmptied =
      outbox && outbox.data.length === 0 && outbox.total > 0 && page > 1;
    if (pageHasEmptied) correctPage(totalPages);
  }, [outbox, page, totalPages, correctPage]);

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Notification outbox</h1>
      <p className="page-subtitle">
        Notifications waiting to be sent. One that fails is retried until it
        succeeds, so cancel it here if it is no longer worth sending. Select a
        row for the full error.
      </p>

      {cancelEntry.isError && <ErrorMessage error={cancelEntry.error} />}

      <TableCard
        toolbar={
          <span className="muted-text">
            {outbox ? `${outbox.total} waiting` : ""}
          </span>
        }
      >
        {isLoading ? (
          <LoadingSpinner />
        ) : error ? (
          <ErrorMessage error={error} />
        ) : entries.length > 0 ? (
          <table className="data-table outbox-table">
            <colgroup>
              <col style={{ width: 44 }} />
              <col style={{ width: "26%" }} />
              <col style={{ width: "13%" }} />
              <col style={{ width: "13%" }} />
              <col style={{ width: 56 }} />
              <col />
              <col style={{ width: 136 }} />
              <col style={{ width: 88 }} />
            </colgroup>
            <thead>
              <tr className="table-header-row">
                <th>
                  <span className="visually-hidden">Details</span>
                </th>
                <th>Notification</th>
                <th>Team</th>
                <th>Channel</th>
                <th>Tries</th>
                <th>Last error</th>
                <th>Queued</th>
                <th>
                  <span className="visually-hidden">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <OutboxRow
                  key={entry.id}
                  entry={entry}
                  isOpen={entry.id === openEntryId}
                  onToggle={() =>
                    setOpenEntryId(entry.id === openEntryId ? null : entry.id)
                  }
                  onCancel={() => setEntryToCancel(entry)}
                />
              ))}
            </tbody>
          </table>
        ) : (
          <TableCardEmpty>Nothing is waiting to be sent.</TableCardEmpty>
        )}
      </TableCard>
      {entries.length > 0 && (
        <Pagination
          page={page}
          totalPages={totalPages}
          onPageChange={goToPage}
        />
      )}

      <ConfirmDialog
        isOpen={entryToCancel !== null}
        onClose={() => setEntryToCancel(null)}
        onConfirm={() => {
          if (entryToCancel) cancelEntry.mutate(entryToCancel.id);
        }}
        title="Cancel notification"
        message={
          entryToCancel
            ? `"${describeEntry(entryToCancel)}" will not be sent${entryToCancel.provider ? ` ${describeDestination(entryToCancel)}` : ""}. This cannot be undone.`
            : ""
        }
        confirmText="Cancel notification"
        cancelText="Keep it"
        isDestructive
      />
    </div>
  );
}
