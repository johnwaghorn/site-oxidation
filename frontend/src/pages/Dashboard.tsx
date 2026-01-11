import { useSites, useCreateSite, useDeleteSite } from '../hooks/useSites'
import { usePagination } from '../hooks/usePagination'
import { SiteList } from '../components/sites/SiteList'
import { SiteForm } from '../components/sites/SiteForm'
import { Pagination } from '../components/ui/Pagination'
import { LoadingSpinner } from '../components/ui/LoadingSpinner'
import { ErrorMessage } from '../components/ui/ErrorMessage'

export function Dashboard() {
    const { page, goToPage } = usePagination()
    const { data, isLoading, error } = useSites(page)
    const createSite = useCreateSite()
    const deleteSite = useDeleteSite()

    const totalPages = data ? Math.ceil(data.total / data.per_page) : 0

    return (
        <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '24px' }}>
            <h1 style={{ marginBottom: '24px' }}>Site Monitor</h1>

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
                        onDelete={id => deleteSite.mutate(id)}
                    />
                    <Pagination
                        page={data.page}
                        totalPages={totalPages}
                        onPageChange={goToPage}
                    />
                </>
            ) : null}
        </div>
    )
}