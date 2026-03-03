'use client'
import { useEffect, useState, useCallback } from 'react'
import { getFirewall, addFirewallRule, delFirewallRule } from '@/lib/api'

export default function FirewallPage() {
    const [fw, setFw] = useState<any>(null)
    const [selected, setSelected] = useState<number | null>(null)
    const [err, setErr] = useState<string | null>(null)
    const [busy, setBusy] = useState(false)
    const [tab, setTab] = useState<'forward' | 'input' | 'output'>('forward')

    const load = useCallback(async () => {
        try { setFw(await getFirewall()); setErr(null) }
        catch (e: any) { setErr(e.message) }
    }, [])

    useEffect(() => { load() }, [load])

    const act = async (type: 'add' | 'del', rule_num?: number) => {
        setBusy(true)
        try {
            if (type === 'add') {
                await addFirewallRule({
                    chain: tab.toUpperCase(), proto: 'tcp',
                    action: 'DROP', comment: 'New Rule'
                })
            } else if (type === 'del' && rule_num !== undefined) {
                await delFirewallRule(rule_num)
                setSelected(null)
            }
            await load()
        } catch (e: any) { setErr(e.message) }
        finally { setBusy(false) }
    }

    const TABS = [
        { id: 'forward', label: 'FORWARD Chain' },
        { id: 'input', label: 'INPUT Chain' },
        { id: 'output', label: 'OUTPUT Chain' }
    ] as const

    const rules = (fw?.rules || []).filter((r: any) => r.chain.toLowerCase() === tab)

    return (
        <>
            <div className="tabbar">
                {TABS.map(t => (
                    <div key={t.id} className={`tab ${tab === t.id ? 'active' : ''}`} onClick={() => { setTab(t.id); setSelected(null) }}>
                        {t.label}
                    </div>
                ))}
            </div>

            <div className="toolbar">
                <button className="tb-btn tb-btn-primary" disabled={busy} onClick={() => act('add')}>Add Rule</button>
                {selected !== null && (
                    <button className="tb-btn" disabled={busy} onClick={() => act('del', selected)}>Delete Selected</button>
                )}
                <div className="tb-sep" />
                <button className="tb-btn" onClick={load}>Refresh</button>
                <div className="tb-info">{rules.length} rule{rules.length !== 1 ? 's' : ''}</div>
            </div>

            {err && <div className="alertbar alertbar-err">Error: {err}</div>}

            <div className="main-area">
                {!fw ? (
                    <div className="empty-state">Loading rules</div>
                ) : rules.length === 0 ? (
                    <div className="empty-state">
                        No rules to show
                        <div className="empty-state-sub">There are no {tab.toUpperCase()} chain rules</div>
                    </div>
                ) : (
                    <table>
                        <thead>
                            <tr>
                                <th style={{ width: 40 }}>#</th>
                                <th>Action</th>
                                <th>Protocol</th>
                                <th>Src Addr</th>
                                <th>Dst Addr</th>
                                <th>Options</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rules.map((r: any) => (
                                <tr key={r.num}
                                    className={selected === r.num ? 'selected' : ''}
                                    onClick={() => setSelected(selected === r.num ? null : r.num)}
                                    style={{ cursor: 'default' }}
                                >
                                    <td className="mono" style={{ color: 'var(--text-muted)' }}>{r.num}</td>
                                    <td className="st-ok">{r.target}</td>
                                    <td className="mono">{r.prot}</td>
                                    <td className="mono">{r.source}</td>
                                    <td className="mono">{r.destination}</td>
                                    <td className="mono" style={{ color: 'var(--text-secondary)' }}>{r.options}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>
        </>
    )
}
