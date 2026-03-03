'use client'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { postSystem } from '@/lib/api'

const NAV = [
    { label: 'Overview', href: '/' },
    { label: 'Devices', href: '/devices' },
    { label: 'Firewall', href: '/firewall' },
    { label: 'NAT', href: '/nat' },
    { label: 'Configuration', href: '/config' },
    { label: 'System Log', href: '/logs' },
    { label: 'Terminal', href: '/terminal' },
]

export default function Sidebar() {
    const path = usePathname()

    const restart = async () => {
        if (confirm('Restart router daemon? This will drop all connections.')) {
            try {
                await postSystem('restart_service')
                alert('Daemon restarting. Please wait a few seconds before refreshing.')
            } catch (e: any) {
                alert(`Failed: ${e.message}`)
            }
        }
    }

    return (
        <nav className="sidebar">
            {NAV.map(item => (
                <Link
                    key={item.href}
                    href={item.href}
                    className={`nav-item ${path === item.href ? 'active' : ''}`}
                >
                    {item.label}
                </Link>
            ))}
            <div style={{ marginTop: 'auto' }}>
                <button
                    onClick={restart}
                    style={{
                        width: '100%',
                        padding: '8px 12px',
                        background: 'transparent',
                        border: '1px solid #e05252',
                        borderRadius: 6,
                        color: '#e05252',
                        fontSize: 13,
                        cursor: 'pointer',
                        textAlign: 'left',
                        transition: 'opacity 0.15s',
                    }}
                    onMouseEnter={e => { (e.target as HTMLButtonElement).style.opacity = '0.7' }}
                    onMouseLeave={e => { (e.target as HTMLButtonElement).style.opacity = '1' }}
                >
                    Restart Daemon
                </button>
            </div>
        </nav>
    )
}
