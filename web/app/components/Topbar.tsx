'use client'
import { useEffect, useState } from 'react'
import { getStatus } from '@/lib/api'

export default function Topbar() {
    const [status, setStatus] = useState<any>(null)

    useEffect(() => {
        const load = async () => {
            try { setStatus(await getStatus()) } catch { }
        }
        load(); const t = setInterval(load, 5000); return () => clearInterval(t)
    }, [])

    return (
        <header className="topbar">
            <div className="topbar-brand">PI-ROUTER</div>
            <div className="topbar-right">
                {status && (
                    <div className="topbar-stats">
                        {status.ssid}
                        <span className="topbar-sep">/</span>
                        {status.devices?.approved ?? 0} allowed
                        <span className="topbar-sep">/</span>
                        {status.devices?.pending ?? 0} pending
                    </div>
                )}
            </div>
        </header>
    )
}
