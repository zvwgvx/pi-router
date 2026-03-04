'use client'
import { usePathname, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'
import Sidebar from './Sidebar'
import Topbar from './Topbar'

export default function LayoutShell({ children }: { children: React.ReactNode }) {
    const pathname = usePathname()
    const router = useRouter()
    const [mounted, setMounted] = useState(false)

    useEffect(() => {
        setMounted(true)
        if (pathname !== '/login') {
            const token = localStorage.getItem('token')
            if (!token) {
                router.push('/login')
            }
        }
    }, [pathname, router])

    // Avoid SSR hydration mismatch when checking localStorage
    if (!mounted) {
        return null
    }

    // Login page gets NO shell — full-screen render
    if (pathname === '/login') {
        return <>{children}</>
    }

    return (
        <div className="shell">
            <Topbar />
            <div className="shell-body">
                <Sidebar />
                <div className="content-pane">
                    {children}
                </div>
            </div>
        </div>
    )
}
