'use client'
import { usePathname } from 'next/navigation'
import Sidebar from './Sidebar'
import Topbar from './Topbar'

export default function LayoutWrapper({ children }: { children: React.ReactNode }) {
    const pathname = usePathname()

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
