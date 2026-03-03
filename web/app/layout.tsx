import type { Metadata } from 'next'
import './globals.css'
import LayoutShell from './components/LayoutShell'

export const metadata: Metadata = {
  title: { default: 'Pi-Router', template: '%s — Pi-Router' },
  description: 'Pi-Router Management Console',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <LayoutShell>
          {children}
        </LayoutShell>
      </body>
    </html>
  )
}
