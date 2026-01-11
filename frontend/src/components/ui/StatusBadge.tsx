interface StatusBadgeProps {
    isUp: boolean
}

export function StatusBadge({isUp}: StatusBadgeProps){
    return (
        <span
            style={{
                display: 'inline-block',
                padding: '2px 8px',
                borderRadius: '4px',
                fontSize: '12px',
                fontWeight: 500,
                backgroundColor: isUp ? '#dcfce7' : '#fee2e2',
                color: isUp ? '#166534' : '#991b1b',
            }}
        >
            {isUp? 'UP': 'DOWN'}
        </span>
    )
}