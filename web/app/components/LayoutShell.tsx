'use client'
import { usePathname } from 'next/navigation'
import Sidebar from './Sidebar'
import Topbar from './Topbar'

export default function LayoutShell({ children }: { children: React.ReactNode }) {
    const pathname = usePathname()

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
