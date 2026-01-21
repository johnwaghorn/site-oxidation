import { Link } from 'react-router-dom'
import type { components } from '../../generated/schema'
import { StatusBadge } from '../ui/StatusBadge'

type SiteResponse = components['schemas']['SiteResponse']

interface SiteRowProps {
    site: SiteResponse
    onDelete?: (id: number) => void
}

export function SiteRow({ site, onDelete }: SiteRowProps) {
    const lastChecked = site.last_checked_at
        ? new Date(site.last_checked_at).toLocaleString()
        : 'Never'

    return (
        <tr style={{ borderBottom: '1px solid #e5e7eb' }}>
            <td style={{ padding: '12px 8px', fontWeight: 500 }}>
                <Link to={`/sites/${site.id}`}>
                    {site.name}
                </Link>
            </td>
            <td style={{ padding: '12px 8px', color: '#6b7280', fontSize: '14px' }}>
                {site.url}
            </td>
            <td style={{ padding: '12px 8px', textAlign: 'center' }}>
                <StatusBadge status={site.status} />
            </td>
            <td style={{ padding: '12px 8px', textAlign: 'right', fontFamily: 'monospace' }}>
                {site.last_response_time_ms != null ? `${site.last_response_time_ms}ms` : '-'}
            </td>
            <td style={{ padding: '12px 8px', textAlign: 'right', color: '#6b7280', fontSize: '14px' }}>
                {lastChecked}
            </td>
            {onDelete && (
                <td style={{ padding: '12px 8px', textAlign: 'right' }}>
                    <button
                        onClick={() => onDelete(site.id)}
                        style={{ color: '#991b1b', background: 'none', border: 'none', cursor: 'pointer' }}
                    >
                        Delete
                    </button>
                </td>
            )}
        </tr>
    )
}