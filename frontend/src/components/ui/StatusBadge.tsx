type SiteStatus = 'pending' | 'up' | 'down'

interface StatusBadgeProps {
    status: SiteStatus
}

const statusConfig = {
    pending: {
        label: 'PENDING',
        backgroundColor: '#dbeafe',
        color: '#1e40af',
    },
    up: {
        label: 'UP',
        backgroundColor: '#dcfce7',
        color: '#166534',
    },
    down: {
        label: 'DOWN',
        backgroundColor: '#fee2e2',
        color: '#991b1b',
    },
}

export function StatusBadge({ status }: StatusBadgeProps) {
    const config = statusConfig[status]
    return (
        <span
            style={{
                display: 'inline-block',
                padding: '2px 8px',
                borderRadius: '4px',
                fontSize: '12px',
                fontWeight: 500,
                backgroundColor: config.backgroundColor,
                color: config.color,
            }}
        >
            {config.label}
        </span>
    )
}
