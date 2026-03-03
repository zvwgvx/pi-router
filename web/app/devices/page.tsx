'use client'
import { useEffect, useState, useCallback } from 'react'
import { getDevices, approveDevice, denyDevice, deleteDevice } from '@/lib/api'

export default function DevicesPage() {
    const [devices, setDevices] = useState<any[]>([])
    const [selected, setSelected] = useState<string | null>(null)
    const [filter, setFilter] = useState<'all' | 'pending' | 'approved' | 'denied'>('all')
    const [search, setSearch] = useState('')
    const [busy, setBusy] = useState(false)
    const [err, setErr] = useState<string | null>(null)

    const load = useCallback(async () => {
        try { const d = await getDevices(); setDevices(d.devices ?? []); setErr(null) }
        catch (e: any) { setErr(e.message) }
    }, [])

    useEffect(() => { load(); const t = setInterval(load, 5000); return () => clearInterval(t) }, [load])

    const act = async (mac: string, a: 'approve' | 'deny' | 'delete') => {
        setBusy(true)
        try {
            if (a === 'approve') await approveDevice(mac)
            else if (a === 'deny') await denyDevice(mac)
            else { await deleteDevice(mac); setSelected(null) }
            await load()
        } catch (e: any) { setErr(e.message) }
        finally { setBusy(false) }
    }

    const visible = devices.filter(d => {
        if (filter !== 'all' && d.state !== filter) return false
        const q = search.toLowerCase()
        return !q || d.mac.includes(q) || d.ip.includes(q) || (d.hostname || '').toLowerCase().includes(q)
    })

    const sel = selected ? devices.find(d => d.mac === selected) : null

    const TABS: Array<'all' | 'pending' | 'approved' | 'denied'> = ['all', 'pending', 'approved', 'denied']

    return (
        <>
            <div className="tabbar">
                {TABS.map(t => {
                    const n = t === 'all' ? devices.length : devices.filter(d => d.state === t).length
                    return (
                        <div key={t} className={`tab ${filter === t ? 'active' : ''}`} onClick={() => setFilter(t)}>
                            {t.charAt(0).toUpperCase() + t.slice(1)} ({n})
                        </div>
                    )
                })}
            </div>

            <div className="toolbar">
                {sel ? (
                    <>
                        {sel.state !== 'approved' && (
                            <button className="tb-btn tb-btn-primary" disabled={busy}
                                onClick={() => act(sel.mac, 'approve')}>Allow</button>
                        )}
                        {sel.state !== 'denied' && (
                            <button className="tb-btn" disabled={busy}
                                onClick={() => act(sel.mac, 'deny')}>Block</button>
                        )}
                        <button className="tb-btn" disabled={busy}
                            onClick={() => act(sel.mac, 'delete')}>Remove</button>
                        <div className="tb-sep" />
                    </>
                ) : null}
                <button className="tb-btn" onClick={load}>Refresh</button>
                <div className="tb-sep" />
                <input
                    className="form-input"
                    style={{ width: 180 }}
                    placeholder="Filter..."
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                />
                <div className="tb-info">{visible.length} device{visible.length !== 1 ? 's' : ''}</div>
            </div>

            {err && <div className="alertbar alertbar-err">Error: {err}</div>}

            <div className="main-area">
                {visible.length === 0 ? (
                    <div className="empty-state">
                        No devices to show
                        <div className="empty-state-sub">No devices match the current filter</div>
                    </div>
                ) : (
                    <table>
                        <thead>
                            <tr>
                                <th>MAC Address</th>
                                <th>IP Address</th>
                                <th>Hostname</th>
                                <th>Status</th>
                                <th>First Seen</th>
                                <th>Last Active</th>
                            </tr>
                        </thead>
                        <tbody>
                            {visible.map((d: any) => (
                                <tr key={d.mac}
                                    className={selected === d.mac ? 'selected' : ''}
                                    onClick={() => setSelected(selected === d.mac ? null : d.mac)}
                                    style={{ cursor: 'default' }}
                                >
                                    <td className="mono">{d.mac}</td>
                                    <td className="mono">{d.ip}</td>
                                    <td>{d.hostname || '\u2014'}</td>
                                    <td>
                                        {d.state === 'approved' && <span className="st-ok">Allowed</span>}
                                        {d.state === 'denied' && <span className="st-err">Blocked</span>}
                                        {d.state === 'pending' && <span className="st-warn">Pending</span>}
                                    </td>
                                    <td className="mono">{new Date(d.first_seen * 1000).toLocaleString()}</td>
                                    <td className="mono">{new Date(d.last_seen * 1000).toLocaleString()}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>
        </>
    )
}
