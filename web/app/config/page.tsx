'use client'
import { useEffect, useState, useCallback } from 'react'
import { getConfig, putConfig, getInterfaces, postSystem } from '@/lib/api'

type SelectOpt = string | { value: string; label: string }
type Section = {
    title: string
    rows: {
        key: string
        label: string
        hint?: string
        type: 'text' | 'pw' | 'number' | 'bool' | 'select' | 'textarea'
        opts?: SelectOpt[]
        path: string[]
    }[]
}

const SECTIONS: Section[] = [
    {
        title: 'Wireless', rows: [
            { key: 'ap_iface', label: 'AP Interface', hint: 'e.g. wlan1', path: ['ap', 'interface'], type: 'select', opts: [] },
            { key: 'ssid', label: 'SSID', hint: '1–32 chars', path: ['ap', 'ssid'], type: 'text' },
            { key: 'pw', label: 'Password', hint: 'min 8 chars (WPA2)', path: ['ap', 'password'], type: 'pw' },
            { key: 'ch', label: 'Channel', hint: '1–14 (2.4 GHz) / 36–165 (5 GHz)', path: ['ap', 'channel'], type: 'number' },
            { key: 'hw', label: 'Wi-Fi Band', hint: '', path: ['ap', 'hw_mode'], type: 'select', opts: [{ value: 'g', label: '2.4 GHz' }, { value: 'a', label: '5 GHz' }] },
            { key: 'cc', label: 'Country Code', hint: 'ISO 3166-1 alpha-2', path: ['ap', 'country_code'], type: 'text' },
            { key: 'n80211n', label: 'Wi-Fi 4 (802.11n)', hint: 'Enable HT extensions', path: ['ap', 'ieee80211n'], type: 'bool' },
            { key: 'n80211ax', label: 'Wi-Fi 6 (802.11ax)', hint: 'Enable HE extensions', path: ['ap', 'ieee80211ax'], type: 'bool' },
            { key: 'wmm', label: 'WMM', hint: 'QoS for multimedia', path: ['ap', 'wmm_enabled'], type: 'bool' },
            { key: 'hide_ssid', label: 'Hidden Network', hint: 'Do not broadcast SSID', path: ['ap', 'ignore_broadcast_ssid'], type: 'bool' },
        ]
    },
    {
        title: 'WAN', rows: [
            { key: 'wan_iface', label: 'WAN Interface', hint: 'e.g. eth0, usb0, wlan0', path: ['wan', 'interface'], type: 'select', opts: [] },
        ]
    },
    {
        title: 'DHCP Server', rows: [
            { key: 'ap_ip', label: 'Gateway IP', hint: 'AP address on LAN', path: ['dhcp', 'ap_ip'], type: 'text' },
            { key: 'netmask', label: 'Netmask', hint: 'e.g. 255.255.255.0', path: ['dhcp', 'netmask'], type: 'text' },
            { key: 'prefix', label: 'Prefix Length', hint: 'e.g. 24 for /24', path: ['dhcp', 'prefix_len'], type: 'number' },
            { key: 'rs', label: 'Pool Start', hint: 'First DHCP address', path: ['dhcp', 'range_start'], type: 'text' },
            { key: 're', label: 'Pool End', hint: 'Last DHCP address', path: ['dhcp', 'range_end'], type: 'text' },
            { key: 'lt', label: 'Lease Time', hint: 'e.g. 12h, infinite', path: ['dhcp', 'lease_time'], type: 'text' },
            { key: 'dns', label: 'DNS Servers', hint: 'One IP per line', path: ['dhcp', 'dns_servers'], type: 'textarea' },
        ]
    },
    {
        title: 'System', rows: [
            { key: 'log', label: 'Log Level', hint: '', path: ['log_level'], type: 'select', opts: ['trace', 'debug', 'info', 'warn', 'error'] },
            { key: 'listen', label: 'API Listen Addr', hint: 'e.g. 0.0.0.0:8080', path: ['http_api', 'listen_addr'], type: 'text' },
            { key: 'devstore', label: 'Devices Store', hint: 'Path to devices.json', path: ['approval', 'devices_store'], type: 'text' },
        ]
    },
]

const getv = (o: any, p: string[]) => p.reduce((x, k) => x?.[k], o)

export default function ConfigPage() {
    const [cfg, setCfg] = useState<any>(null)
    const [vals, setVals] = useState<Record<string, any>>({})
    const [tab, setTab] = useState('Wireless')
    const [saved, setSaved] = useState(false)
    const [err, setErr] = useState<string | null>(null)
    const [dirty, setDirty] = useState(false)

    const [ifaces, setIfaces] = useState<string[]>([])

    const load = useCallback(async () => {
        try {
            const [c, ifcs] = await Promise.all([getConfig(), getInterfaces()])
            setCfg(c)
            setIfaces(ifcs)
            const init: Record<string, any> = {}
            for (const s of SECTIONS) {
                for (const r of s.rows) {
                    const v = getv(c, r.path)
                    init[r.key] = r.type === 'textarea'
                        ? (Array.isArray(v) ? v.join('\n') : (v ?? ''))
                        : (v ?? '')
                }
            }
            setVals(init)
            setDirty(false)
            setErr(null)
        } catch (e: any) { setErr(e.message) }
    }, [])

    useEffect(() => { load() }, [load])

    const set = (k: string, v: any) => { setVals(x => ({ ...x, [k]: v })); setDirty(true) }

    const save = async () => {
        try {
            const body = {
                wan: { interface: vals.wan_iface },
                ap: {
                    interface: vals.ap_iface,
                    ssid: vals.ssid,
                    password: vals.pw,
                    channel: Number(vals.ch),
                    hw_mode: vals.hw,
                    country_code: vals.cc,
                    ieee80211n: !!vals.n80211n,
                    ieee80211ax: !!vals.n80211ax,
                    ignore_broadcast_ssid: !!vals.hide_ssid,
                    wmm_enabled: !!vals.wmm,
                },
                dhcp: {
                    ap_ip: vals.ap_ip,
                    netmask: vals.netmask,
                    prefix_len: Number(vals.prefix),
                    range_start: vals.rs,
                    range_end: vals.re,
                    lease_time: vals.lt,
                    dns_servers: (vals.dns as string).split('\n').map((s: string) => s.trim()).filter(Boolean),
                },
                monitor: {
                    check_interval_secs: Number(vals.interval),
                    max_restart_attempts: Number(vals.restarts),
                },
                log_level: vals.log,
                http_api: { listen_addr: vals.listen },
                approval: { devices_store: vals.devstore },
            }
            await putConfig(body)
            setSaved(true)
            setDirty(false)
            setTimeout(() => setSaved(false), 3000)
        } catch (e: any) { setErr(e.message) }
    }

    const sec = SECTIONS.find(s => s.title === tab)

    return (
        <>
            <div className="tabbar">
                {SECTIONS.map(s => (
                    <div key={s.title} className={`tab ${tab === s.title ? 'active' : ''}`} onClick={() => setTab(s.title)}>
                        {s.title}
                    </div>
                ))}
            </div>

            <div className="toolbar">
                <button className="tb-btn tb-btn-primary" disabled={!dirty} onClick={save}>Apply Settings</button>
                <button className="tb-btn" onClick={load}>Discard Changes</button>
                <div className="tb-sep" />
                {saved && <span className="tb-info">Saved — restart daemon to apply</span>}
                {dirty && !saved && <span className="tb-info">Unsaved changes</span>}
                <div style={{ marginLeft: 'auto' }}>
                    {tab === 'System' && (
                        <button className="tb-btn" style={{ color: 'var(--danger, #e05252)' }} onClick={async () => {
                            if (confirm('Restart router daemon? This will drop all connections.')) {
                                try {
                                    await postSystem('restart_service');
                                    alert('Daemon restarting. Please wait a few seconds before refreshing.');
                                } catch (err: any) {
                                    alert(`Failed: ${err.message}`);
                                }
                            }
                        }}>Restart Daemon</button>
                    )}
                </div>
            </div>

            {err && <div className="alertbar alertbar-err">Error: {err}</div>}

            <div className="main-area">
                {!cfg ? (
                    <div className="empty-state">
                        Loading
                        <div className="empty-state-sub">Fetching configuration</div>
                    </div>
                ) : (
                    <>
                        <table>
                            <thead>
                                <tr>
                                    <th style={{ width: 220 }}>Property</th>
                                    <th>Value</th>
                                    <th style={{ width: 260 }}>Note</th>
                                </tr>
                            </thead>
                            <tbody>
                                {sec?.rows.map(r => (
                                    <tr key={r.key}>
                                        <td className="prop-label">{r.label}</td>
                                        <td>
                                            {r.type === 'bool' ? (
                                                <label className="ro-toggle">
                                                    <input type="checkbox" checked={!!vals[r.key]} onChange={e => set(r.key, e.target.checked)} />
                                                    <span className="ro-toggle-slider" />
                                                </label>
                                            ) : r.type === 'select' ? (
                                                <select className="form-select" value={vals[r.key] ?? ''} onChange={e => set(r.key, e.target.value)}>
                                                    {r.key === 'ap_iface' || r.key === 'wan_iface'
                                                        ? ifaces.map(o => <option key={o} value={o}>{o}</option>)
                                                        : r.opts?.map(o => {
                                                            const val = typeof o === 'string' ? o : o.value
                                                            const lbl = typeof o === 'string' ? o : o.label
                                                            return <option key={val} value={val}>{lbl}</option>
                                                        })}
                                                </select>
                                            ) : r.type === 'textarea' ? (
                                                <textarea
                                                    className="form-input mono"
                                                    style={{ width: 240, height: 72, resize: 'vertical', padding: '6px 10px' }}
                                                    value={vals[r.key] ?? ''}
                                                    onChange={e => set(r.key, e.target.value)}
                                                />
                                            ) : (
                                                <input
                                                    className="form-input mono"
                                                    style={{ width: 240 }}
                                                    type={r.type === 'pw' ? 'password' : r.type === 'number' ? 'number' : 'text'}
                                                    value={vals[r.key] ?? ''}
                                                    onChange={e => set(r.key, e.target.value)}
                                                />
                                            )}
                                        </td>
                                        <td style={{ color: 'var(--text-muted)', fontSize: 11 }}>{r.hint}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>

                    </>
                )}
            </div>
        </>
    )
}
