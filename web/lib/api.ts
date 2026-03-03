// Central API client — all fetches point to the Rust daemon's HTTP API
const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8080'

async function api<T>(path: string, init?: RequestInit): Promise<T> {
    const res = await fetch(`${API_BASE}${path}`, {
        headers: { 'Content-Type': 'application/json' },
        ...init,
    })
    if (!res.ok) {
        const text = await res.text()
        throw new Error(`API ${path}: ${res.status} ${text}`)
    }
    return res.json()
}

export const getStatus = () => api<any>('/api/status')
export const getDevices = () => api<any>('/api/devices')
export const approveDevice = (mac: string) => api<any>(`/api/devices/${mac}/approve`, { method: 'POST' })
export const denyDevice = (mac: string) => api<any>(`/api/devices/${mac}/deny`, { method: 'POST' })
export const deleteDevice = (mac: string) => api<any>(`/api/devices/${mac}`, { method: 'DELETE' })

export const getConfig = () => api<any>('/api/config')
export const putConfig = (body: any) => api<any>('/api/config', { method: 'PUT', body: JSON.stringify(body) })

export const getInterfaces = () => api<string[]>('/api/interfaces')
export const postSystem = (action: string) => api<any>('/api/system', { method: 'POST', body: JSON.stringify({ action }) })

export const getFirewall = () => api<any>('/api/firewall')
export const addFirewallRule = (rule: any) => api<any>('/api/firewall', { method: 'POST', body: JSON.stringify(rule) })
export const delFirewallRule = (rule_num: number) => api<any>('/api/firewall', { method: 'DELETE', body: JSON.stringify({ rule_num }) })

export const getNat = () => api<any>('/api/nat')
export const addNatRule = (rule: any) => api<any>('/api/nat', { method: 'POST', body: JSON.stringify(rule) })
export const delNatRule = (rule_num: number) => api<any>('/api/nat', { method: 'DELETE', body: JSON.stringify({ rule_num }) })
