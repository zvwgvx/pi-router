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
            { key: 'wifi_ver', label: 'Wi-Fi Version', hint: '', path: ['_virtual'], type: 'select', opts: [{ value: 'legacy', label: 'Legacy (802.11a/b/g)' }, { value: '4', label: 'Wi-Fi 4 (802.11n)' }, { value: '5', label: 'Wi-Fi 5 (802.11ac)' }, { value: '6', label: 'Wi-Fi 6 (802.11ax)' }] },
            { key: 'wmm', label: 'WMM', hint: 'QoS for multimedia', path: ['ap', 'wmm_enabled'], type: 'bool' },
            { key: 'hide_ssid', label: 'Hidden Network', hint: 'Do not broadcast SSID', path: ['ap', 'ignore_broadcast_ssid'], type: 'bool' },
            { key: 'pw_enabled', label: 'Require Password', hint: 'Disable for open network', path: ['ap', 'password_enabled'], type: 'bool' },
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
        title: 'Monitor', rows: [
            { key: 'interval', label: 'Health Check Interval', hint: 'Seconds between checks', path: ['monitor', 'check_interval_secs'], type: 'number' },
            { key: 'restarts', label: 'Max Restarts', hint: 'Give up if daemon crashes N times', path: ['monitor', 'max_restart_attempts'], type: 'number' },
        ]
    },
    {
        title: 'System', rows: [
            { key: 'log', label: 'Log Level', hint: '', path: ['log_level'], type: 'select', opts: ['trace', 'debug', 'info', 'warn', 'error'] },
            { key: 'listen', label: 'API Listen Addr', hint: 'e.g. 0.0.0.0:8080', path: ['http_api', 'listen_addr'], type: 'text' },
            { key: 'devstore', label: 'Devices Store', hint: 'Path to devices.json', path: ['approval', 'devices_store'], type: 'text' },
            { key: 'require_approval', label: 'Device Approval', hint: 'Require manual approval for new clients', path: ['approval', 'require_approval'], type: 'bool' },
            { key: 'admin_user', label: 'Admin Username', hint: '', path: ['admin', 'username'], type: 'text' },
            { key: 'admin_pass', label: 'Admin Password', hint: 'Panel login password', path: ['admin', 'password'], type: 'pw' },
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
    const [chErr, setChErr] = useState<string | null>(null)

    const CH_24 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
    const CH_5 = [36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 149, 153, 157, 161, 165]

    const randomCh = (band: string) => {
        const pool = band === 'a' ? CH_5 : CH_24
        return pool[Math.floor(Math.random() * pool.length)]
    }

    const validateChannel = (ch: number, band: string) => {
        if (band === 'a' && !CH_5.includes(ch))
            return `Channel ${ch} is not valid for 5 GHz. Valid: 36–165 (standard U-NII channels).`
        if (band === 'g' && (ch < 1 || ch > 13))
            return `Channel ${ch} is not valid for 2.4 GHz. Valid: 1–13.`
        return null
    }

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
            // Derive wifi_ver from AP settings
            if (c.ap.ieee80211ax) init.wifi_ver = '6'
            else if (c.ap.ieee80211ac) init.wifi_ver = '5'
            else if (c.ap.ieee80211n) init.wifi_ver = '4'
            else init.wifi_ver = 'legacy'

            setVals(init)
            setDirty(false)
            setErr(null)
        } catch (e: any) { setErr(e.message) }
    }, [])

    useEffect(() => { load() }, [load])

    const set = (k: string, v: any) => {
        setVals(x => {
            const next = { ...x, [k]: v }
            // When band changes, auto-randomize channel
            if (k === 'hw') {
                next.ch = randomCh(v as string)
                setChErr(null)
            }
            // When channel changes, live-validate
            if (k === 'ch') {
                setChErr(validateChannel(Number(v), x.hw as string))
            }
            return next
        })
        setDirty(true)
    }

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
                    ieee80211n: vals.wifi_ver === '4' || vals.wifi_ver === '5' || vals.wifi_ver === '6',
                    ieee80211ac: vals.wifi_ver === '5' || vals.wifi_ver === '6',
                    ieee80211ax: vals.wifi_ver === '6',
                    ignore_broadcast_ssid: !!vals.hide_ssid,
                    wmm_enabled: !!vals.wmm,
                    password_enabled: !!vals.pw_enabled,
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
                approval: { devices_store: vals.devstore, require_approval: !!vals.require_approval },
                admin: { username: vals.admin_user, password: vals.admin_pass },
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
                                        <td style={{ color: 'var(--text-muted)', fontSize: 11 }}>
                                            {r.key === 'ch' && chErr
                                                ? <span style={{ color: 'var(--danger, #e05252)' }}>{chErr}</span>
                                                : r.hint}
                                        </td>
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
