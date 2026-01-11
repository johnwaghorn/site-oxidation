import type {components} from '../../generated/schema'
import {SiteRow} from './SiteRow.tsx'

type SiteResponse = components['schemas']['SiteResponse']

interface SiteListProps {
    sites: SiteResponse[]
    onDelete?: (id:number) => void
}

export function SiteList({sites, onDelete}: SiteListProps){
    if (sites.length ===0){
        return <p>No sites configured. Add one!</p>
    }
    return (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
            <tr style={{ borderBottom: '2px solid #e5e7eb' }}>
                <th style={{ textAlign: 'left', padding: '12px 8px' }}>Name</th>
                <th style={{ textAlign: 'left', padding: '12px 8px' }}>URL</th>
                <th style={{ textAlign: 'center', padding: '12px 8px' }}>Status</th>
                <th style={{ textAlign: 'right', padding: '12px 8px' }}>Latency</th>
                <th style={{ textAlign: 'right', padding: '12px 8px' }}>Last Checked</th>
                {onDelete && <th style={{ padding: '12px 8px' }}></th>}
            </tr>
            </thead>
            <tbody>
            {sites.map(site => (
                <SiteRow key={site.id} site={site} onDelete={onDelete} />
            ))}
            </tbody>
        </table>
)
}


