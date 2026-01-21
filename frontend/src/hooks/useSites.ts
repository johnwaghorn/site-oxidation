import { useQuery, useMutation, useQueryClient} from '@tanstack/react-query'
import {api} from '../lib/api'
import type {components} from "../generated/schema"

type SitePayload = components['schemas']['SitePayload']


export function  useSites(page = 1, perPage = 20) {
    return useQuery({
        queryKey: ['sites', {page, perPage}],
        queryFn: async () => {
        const {data, error} = await api.GET('/api/sites', {
            params: {query: {page, per_page: perPage}},
        })
        if(error) throw error
        return data!
    },
        refetchInterval: 30_000, // 30 seconds
    })
}

export function useSite(id: number) {
    return useQuery({
        queryKey: ['sites', id],
        queryFn: async () => {
            const {data, error} = await api.GET('/api/sites/{id}',{
                params: {path: {id }},
            })
            if (error) throw error
            return data!
        },
        enabled: id > 0,
    })
}

export function useCreateSite() {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (site: SitePayload) => {
            const {data, error} = await api.POST('/api/sites', {body: site})
            if (error) throw error
            return data!
        },
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: ['sites']})
        },
    })
}

export function  useUpdateSite() {
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async ({ id, site }: { id: number; site: SitePayload }) => {
            const { data, error } = await api.PUT('/api/sites/{id}', {
                params: {path: {id}},
                body: site,
            })
            if (error) throw error
            return data!
        },
        onSuccess: (_, {id}) => {
            queryClient.invalidateQueries({queryKey:['sites']})
            queryClient.invalidateQueries({queryKey:['sites', id]})
        },
    })
}

export function useDeleteSite(){
    const queryClient = useQueryClient()
    return useMutation({
        mutationFn: async (id: number) => {
            const {error } = await api.DELETE('/api/sites/{id}', {
                params: {path: {id}},
            })
            if (error) throw error
        },
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey:['sites']})
        },
    })
}

export function useOutages(siteId: number, page = 1, perPage = 20) {
    return useQuery({
        queryKey: ['sites', siteId, 'outages', { page, perPage }],
        queryFn: async () => {
            const { data, error } = await api.GET('/api/sites/{id}/outages', {
                params: { path: { id: siteId }, query: { page, per_page: perPage } },
            })
            if (error) throw error
            return data!
        },
        enabled: siteId > 0,
    })
}