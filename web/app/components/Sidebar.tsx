'use client'
import Link from 'next/link'
import { usePathname } from 'next/navigation'

const NAV = [
    { label: 'Overview', href: '/' },
    { label: 'Devices', href: '/devices' },
    { label: 'Firewall', href: '/firewall' },
    { label: 'NAT', href: '/nat' },
    { label: 'Configuration', href: '/config' },
]

export default function Sidebar() {
    const path = usePathname()

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
        </nav>
    )
}
