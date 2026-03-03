import type { Metadata } from 'next'
import './globals.css'
import Sidebar from './components/Sidebar'
import Topbar from './components/Topbar'

export const metadata: Metadata = {
  title: { default: 'Pi-Router', template: '%s — Pi-Router' },
  description: 'Pi-Router Management Console',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div className="shell">
          <Topbar />
          <div className="shell-body">
            <Sidebar />
            <div className="content-pane">
              {children}
            </div>
          </div>
        </div>
      </body>
    </html>
  )
}
