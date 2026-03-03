'use client'
import { useEffect, useRef, useState } from 'react'
import { API } from '@/lib/api'

export default function LogsPage() {
    const [lines, setLines] = useState<string[]>([])
    const [running, setRunning] = useState(true)
    const [err, setErr] = useState<string | null>(null)
    const bottomRef = useRef<HTMLDivElement>(null)
    const esRef = useRef<EventSource | null>(null)

    useEffect(() => {
        if (!running) return
        const es = new EventSource(`${API}/api/logs`)
        esRef.current = es
        es.onmessage = (e) => {
            setLines(prev => {
                const next = [...prev, e.data]
                return next.length > 2000 ? next.slice(next.length - 2000) : next
            })
        }
        es.onerror = () => setErr('Log stream disconnected. Is the daemon running with journald?')
        return () => { es.close(); esRef.current = null }
    }, [running])

    useEffect(() => {
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
    }, [lines])

    return (
        <>
            <div className="toolbar">
                <button className="tb-btn tb-btn-primary" onClick={() => { setLines([]); setRunning(false); setTimeout(() => setRunning(true), 50) }}>
                    Reconnect
                </button>
                <button className="tb-btn" onClick={() => setLines([])}>Clear</button>
                <div className="tb-sep" />
                <span className="tb-info">{lines.length} lines</span>
                <label style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: 'var(--text-muted)', cursor: 'pointer' }}>
                    <input type="checkbox" checked={running} onChange={e => setRunning(e.target.checked)} />
                    Live tail
                </label>
            </div>
            {err && <div className="alertbar alertbar-err">{err}</div>}
            <div className="main-area" style={{ padding: 0 }}>
                <div style={{
                    fontFamily: '"JetBrains Mono", "Fira Code", monospace',
                    fontSize: 12,
                    lineHeight: '1.6',
                    padding: '12px 16px',
                    color: 'var(--text-primary)',
                    overflowY: 'auto',
                    height: '100%',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-all',
                }}>
                    {lines.length === 0 && (
                        <span style={{ color: 'var(--text-muted)' }}>Connecting to log stream...</span>
                    )}
                    {lines.map((l, i) => (
                        <div key={i} style={{
                            color: l.includes('ERROR') || l.includes('error') ? '#e05252'
                                : l.includes('WARN') || l.includes('warn') ? '#e0a052'
                                    : 'var(--text-primary)'
                        }}>{l}</div>
                    ))}
                    <div ref={bottomRef} />
                </div>
            </div>
        </>
    )
}
