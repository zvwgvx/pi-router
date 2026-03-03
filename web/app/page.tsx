'use client'
import { useEffect, useState, useCallback } from 'react'
import { getStatus, getDevices, approveDevice, denyDevice } from '@/lib/api'

export default function Dashboard() {
  const [status, setStatus] = useState<any>(null)
  const [devices, setDevices] = useState<any[]>([])
  const [err, setErr] = useState<string | null>(null)
  const [tab, setTab] = useState('Overview')

  const load = useCallback(async () => {
    try {
      const [s, d] = await Promise.all([getStatus(), getDevices()])
      setStatus(s); setDevices(d.devices ?? []); setErr(null)
    } catch (e: any) { setErr(e.message) }
  }, [])

  useEffect(() => { load(); const t = setInterval(load, 1000); return () => clearInterval(t) }, [load])

  const pending = devices.filter(d => d.state === 'pending')
  const approved = devices.filter(d => d.state === 'approved')

  const fmtUp = (s: number) => {
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
    return h ? `${h}h ${m}m` : `${m}m ${s % 60}s`
  }

  const fmtBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  const act = (mac: string, a: 'approve' | 'deny') =>
    (a === 'approve' ? approveDevice(mac) : denyDevice(mac)).then(load)

  const TABS = ['Overview', 'Pending Approvals', 'All Devices']

  return (
    <>
      <div className="tabbar">
        {TABS.map(t => (
          <div key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>{t}</div>
        ))}
      </div>

      <div className="toolbar">
        <div className="tb-info">{approved.length} allowed, {pending.length} pending</div>
      </div>

      {err && <div className="alertbar alertbar-err">Failed to fetch: {err}</div>}

      <div className="main-area">
        {tab === 'Overview' && (
          <table>
            <thead>
              <tr>
                <th>Property</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              {[
                ['Status', status ? 'Running' : '...'],
                ['Version', `v${status?.version ?? '...'}`],
                ['Uptime', status ? fmtUp(status.uptime) : '...'],
                ['CPU Usage', status?.system ? `${status.system.cpu_percent.toFixed(1)}%` : '...'],
                ['RAM Usage', status?.system ? `${fmtBytes(status.system.ram_used)} / ${fmtBytes(status.system.ram_total)} (${Math.round((status.system.ram_used / status.system.ram_total) * 100)}%)` : '...'],
                ['Temperature', status?.system?.temp_c ? `${status.system.temp_c.toFixed(1)} °C` : '...'],
                ['Disk Usage', status?.system ? `${fmtBytes(status.system.disk_used)} / ${fmtBytes(status.system.disk_total)} (${status.system.disk_total ? Math.round((status.system.disk_used / status.system.disk_total) * 100) : 0}%)` : '...'],
                ['WAN Bandwidth', status?.system ? `↓ ${fmtBytes(status.system.net_rx_bps)}/s  ↑ ${fmtBytes(status.system.net_tx_bps)}/s` : '...'],
                ['SSID', status?.ssid ?? '...'],
                ['AP / WAN Iface', status ? `${status.ap} / ${status.wan}` : '...'],
                ['Gateway IP', status?.ap_ip ?? '...'],
                ['Clients (Total)', String(status?.devices?.total ?? '...')],
                ['Clients (Allowed)', String(status?.devices?.approved ?? '...')],
                ['Clients (Pending)', String(status?.devices?.pending ?? '...')],
                ['Clients (Denied)', String(status?.devices?.denied ?? '...')],
              ].map(([k, v]) => (
                <tr key={k as string}>
                  <td className="prop-label">{k}</td>
                  <td className="mono">{v}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {tab === 'Pending Approvals' && (
          pending.length === 0 ? (
            <div className="empty-state">
              No pending devices
              <div className="empty-state-sub">All devices have been reviewed</div>
            </div>
          ) : (
            <table>
              <thead>
                <tr><th>MAC Address</th><th>IP Address</th><th>Hostname</th><th>Actions</th></tr>
              </thead>
              <tbody>
                {pending.map((d: any) => (
                  <tr key={d.mac}>
                    <td className="mono">{d.mac}</td>
                    <td className="mono">{d.ip}</td>
                    <td>{d.hostname || '\u2014'}</td>
                    <td>
                      <button className="tb-btn tb-btn-primary" style={{ marginRight: 4 }} onClick={() => act(d.mac, 'approve')}>Allow</button>
                      <button className="tb-btn" onClick={() => act(d.mac, 'deny')}>Block</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        )}

        {tab === 'All Devices' && (
          devices.length === 0 ? (
            <div className="empty-state">
              No devices connected
              <div className="empty-state-sub">Waiting for client connections</div>
            </div>
          ) : (
            <table>
              <thead>
                <tr><th>MAC Address</th><th>IP Address</th><th>Hostname</th><th>Status</th><th>Last Seen</th></tr>
              </thead>
              <tbody>
                {devices.map((d: any) => (
                  <tr key={d.mac}>
                    <td className="mono">{d.mac}</td>
                    <td className="mono">{d.ip}</td>
                    <td>{d.hostname || '\u2014'}</td>
                    <td>
                      {d.state === 'approved' && <span className="st-ok">Allowed</span>}
                      {d.state === 'denied' && <span className="st-err">Blocked</span>}
                      {d.state === 'pending' && <span className="st-warn">Pending</span>}
                    </td>
                    <td className="mono">{new Date(d.last_seen * 1000).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        )}
      </div>
    </>
  )
}
