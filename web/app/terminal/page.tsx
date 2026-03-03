'use client'
import { useEffect, useRef } from 'react'
import { API } from '@/lib/api'

export default function TerminalPage() {
    const containerRef = useRef<HTMLDivElement>(null)

    useEffect(() => {
        let term: any
        let ws: WebSocket

        const init = async () => {
            const { Terminal } = await import('@xterm/xterm')
            const { FitAddon } = await import('@xterm/addon-fit')
            await import('@xterm/xterm/css/xterm.css' as any)

            term = new Terminal({
                cursorBlink: true,
                fontFamily: '"JetBrains Mono","Fira Code",monospace',
                fontSize: 13,
                theme: {
                    background: '#1a1a1f',
                    foreground: '#cdd6f4',
                    cursor: '#cba6f7',
                    selectionBackground: '#45475a',
                    black: '#45475a', red: '#f38ba8', green: '#a6e3a1',
                    yellow: '#f9e2af', blue: '#89b4fa', magenta: '#cba6f7',
                    cyan: '#89dceb', white: '#bac2de',
                    brightBlack: '#585b70', brightRed: '#f38ba8', brightGreen: '#a6e3a1',
                    brightYellow: '#f9e2af', brightBlue: '#89b4fa', brightMagenta: '#cba6f7',
                    brightCyan: '#89dceb', brightWhite: '#a6adc8',
                },
            })

            const fitAddon = new FitAddon()
            term.loadAddon(fitAddon)
            term.open(containerRef.current!)
            fitAddon.fit()

            const token = localStorage.getItem('token')
            if (!token) {
                window.location.href = '/login'
                return
            }
            const wsUrl = API.replace('http', 'ws') + '/api/terminal?token=' + token
            ws = new WebSocket(wsUrl)

            ws.onopen = () => term.writeln('\x1b[32mConnected to RPi shell.\x1b[0m')
            ws.onmessage = (e) => term.write(e.data)
            ws.onerror = () => term.writeln('\x1b[31m[ERROR] WebSocket connection failed.\x1b[0m')
            ws.onclose = () => term.writeln('\x1b[33m[Disconnected]\x1b[0m')

            term.onData((data: string) => {
                if (ws.readyState === WebSocket.OPEN) ws.send(data)
            })

            const resizeObserver = new ResizeObserver(() => fitAddon.fit())
            if (containerRef.current) resizeObserver.observe(containerRef.current)

            return () => {
                resizeObserver.disconnect()
            }
        }

        init()

        return () => {
            ws?.close()
            term?.dispose()
        }
    }, [])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div className="toolbar">
                <span style={{ fontSize: 13, color: 'var(--text-muted)' }}>
                    RPi Terminal — root shell over WebSocket
                </span>
            </div>
            <div
                ref={containerRef}
                style={{
                    flex: 1,
                    background: '#1a1a1f',
                    padding: 8,
                    overflow: 'hidden',
                }}
            />
        </div>
    )
}
