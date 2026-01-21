import { useState, type FormEvent } from 'react'
import type { components } from '../../generated/schema'

type SitePayload = components['schemas']['SitePayload']

interface SiteFormProps {
    onSubmit: (site: SitePayload) => void
    isLoading?: boolean
    mode?: 'create' | 'edit'
    initialData?: SitePayload
}

export function SiteForm({ onSubmit, isLoading, mode = 'create', initialData }: SiteFormProps) {
    const [name, setName] = useState(initialData?.name ?? '')
    const [url, setUrl] = useState(initialData?.url ?? '')
    const [expectedStatus, setExpectedStatus] = useState(initialData?.expected_status ?? 200)
    const [expectedText, setExpectedText] = useState(initialData?.expected_text ?? '')

    const handleSubmit = (e: FormEvent) => {
        e.preventDefault()
        onSubmit({ name, url, expected_status: expectedStatus, expected_text: expectedText || null })
        if (mode === 'create') {
            setName('')
            setUrl('')
            setExpectedStatus(200)
            setExpectedText('')
        }
    }

    const isEdit = mode === 'edit'

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
                placeholder="https://waghorn.tech"
                value={url}
                onChange={e => setUrl(e.target.value)}
                required
                style={{ padding: '8px', flex: 2 }}
            />
            <input
                type="number"
                placeholder="Expected status code"
                value={expectedStatus}
                onChange={e => setExpectedStatus(Number(e.target.value))}
                min={100}
                max={599}
                style={{ padding: '8px', width: '100px' }}
            />
            <input
                type="text"
                placeholder="Expected text (optional)"
                value={expectedText}
                onChange={e => setExpectedText(e.target.value)}
                style={{ padding: '8px', flex: 1 }}
            />
            <button type="submit" disabled={isLoading} style={{ padding: '8px 16px' }}>
                {isLoading ? (isEdit ? 'Saving...' : 'Adding...') : (isEdit ? 'Save' : 'Add Site')}
            </button>
        </form>
    )
}
