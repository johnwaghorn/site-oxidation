import { useState } from 'react'
import { useSites, useCreateSite, useDeleteSite } from '../hooks/useSites'
import { usePagination } from '../hooks/usePagination'
import { SiteList } from '../components/sites/SiteList'
import { SiteForm } from '../components/sites/SiteForm'
import { Pagination } from '../components/ui/Pagination'
import { LoadingSpinner } from '../components/ui/LoadingSpinner'
import { ErrorMessage } from '../components/ui/ErrorMessage'
import { ConfirmDialog } from '../components/ui/ConfirmDialog'
import type { components } from '../generated/schema'

type SiteResponse = components['schemas']['SiteResponse']

export function Dashboard() {
    const { page, goToPage } = usePagination()
    const { data, isLoading, error } = useSites(page)
    const createSite = useCreateSite()
    const deleteSite = useDeleteSite()
    const [siteToDelete, setSiteToDelete] = useState<SiteResponse | null>(null)

    const totalPages = data ? Math.ceil(data.total / data.per_page) : 0

    return (
        <div style={{ maxWidth: '1200px', padding: '24px' }}>
            <h1 style={{ marginBottom: '24px' }}>Site Oxidation</h1>

            <SiteForm
                onSubmit={site => createSite.mutate(site)}
                isLoading={createSite.isPending}
            />

            {createSite.isError && <ErrorMessage error={createSite.error} />}

            {isLoading ? (
                <LoadingSpinner />
            ) : error ? (
                <ErrorMessage error={error} />
            ) : data ? (
                <>
                    <SiteList
                        sites={data.data}
                        onDelete={site => setSiteToDelete(site)}
                    />
                    <Pagination
                        page={data.page}
                        totalPages={totalPages}
                        onPageChange={goToPage}
                    />
                </>
            ) : null}

            <ConfirmDialog
                isOpen={siteToDelete !== null}
                onClose={() => setSiteToDelete(null)}
                onConfirm={() => siteToDelete && deleteSite.mutate(siteToDelete.id)}
                title="Delete Site"
                message={`Are you sure you want to delete "${siteToDelete?.name}"? This will also delete all outage history.`}
                confirmText="Delete"
                isDestructive
            />
        </div>
    )
}
