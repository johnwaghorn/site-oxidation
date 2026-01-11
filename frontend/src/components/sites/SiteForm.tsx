import { useState, type FormEvent } from 'react'
import type { components } from '../../generated/schema'

type SitePayload = components['schemas']['SitePayload']

interface SiteFormProps {
    onSubmit: (site: SitePayload) => void
    isLoading?: boolean
}

export function SiteForm({ onSubmit, isLoading }: SiteFormProps) {
    const [name, setName] = useState('')
    const [url, setUrl] = useState('')
    const [expectedStatus, setExpectedStatus] = useState(200)

    const handleSubmit = (e: FormEvent) => {
        e.preventDefault()
        onSubmit({ name, url, expected_status: expectedStatus })
        setName('')
        setUrl('')
        setExpectedStatus(200)
    }

    return (
        <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '8px', marginBottom: '24px' }}>
            <input
                type="text"
                placeholder="Site name"
                value={name}
                onChange={e => setName(e.target.value)}
                required
                minLength={1}
                maxLength={100}
                style={{ padding: '8px', flex: 1 }}
            />
            <input
                type="url"
                placeholder="https://example.com"
                value={url}
                onChange={e => setUrl(e.target.value)}
                required
                style={{ padding: '8px', flex: 2 }}
            />
            <input
                type="number"
                placeholder="Expected status"
                value={expectedStatus}
                onChange={e => setExpectedStatus(Number(e.target.value))}
                min={100}
                max={599}
                style={{ padding: '8px', width: '100px' }}
            />
            <button type="submit" disabled={isLoading} style={{ padding: '8px 16px' }}>
                {isLoading ? 'Adding...' : 'Add Site'}
            </button>
        </form>
    )
}