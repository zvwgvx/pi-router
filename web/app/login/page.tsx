'use client'
import { useState } from 'react'
import { login } from '@/lib/api'

export default function LoginPage() {
    const [username, setUsername] = useState('admin')
    const [password, setPassword] = useState('')
    const [error, setError] = useState('')
    const [loading, setLoading] = useState(false)

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setError('')
        setLoading(true)
        try {
            const res = await login({ username, password })
            if (res.token) {
                localStorage.setItem('token', res.token)
                window.location.href = '/'
            }
        } catch (err: any) {
            setError(err.message.includes('401') ? 'Invalid login or password' : err.message)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div style={{
            position: 'fixed', inset: 0,
            display: 'flex', flexDirection: 'column',
            alignItems: 'center', justifyContent: 'center',
            background: '#1c1c1e',
            fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
            fontSize: 14,
            color: '#f0f0f0',
        }}>
            {/* Logo */}
            <div style={{
                fontSize: 13, fontWeight: 600,
                letterSpacing: '2px', color: '#ababab',
                marginBottom: 24,
            }}>
                PI-ROUTER
            </div>

            {/* Card */}
            <div style={{
                background: '#242426',
                border: '1px solid #3a3a3c',
                borderRadius: 6,
                padding: '24px 28px',
                width: 300,
            }}>
                <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>

                    {/* Login row */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                        <label style={{ fontSize: 12, color: '#ababab', width: 65, flexShrink: 0 }}>Login</label>
                        <input
                            type="text"
                            value={username}
                            onChange={e => setUsername(e.target.value)}
                            required
                            autoFocus
                            style={{
                                flex: 1, height: 28,
                                padding: '0 8px',
                                background: '#1c1c1e',
                                border: '1px solid #3a3a3c',
                                borderRadius: 4,
                                color: '#f0f0f0',
                                fontSize: 12,
                                fontFamily: 'inherit',
                                outline: 'none',
                                boxSizing: 'border-box',
                            }}
                        />
                    </div>

                    {/* Password row */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                        <label style={{ fontSize: 12, color: '#ababab', width: 65, flexShrink: 0 }}>Password</label>
                        <input
                            type="password"
                            value={password}
                            onChange={e => setPassword(e.target.value)}
                            style={{
                                flex: 1, height: 28,
                                padding: '0 8px',
                                background: '#1c1c1e',
                                border: '1px solid #3a3a3c',
                                borderRadius: 4,
                                color: '#f0f0f0',
                                fontSize: 12,
                                fontFamily: 'inherit',
                                outline: 'none',
                                boxSizing: 'border-box',
                            }}
                        />
                    </div>

                    {error && (
                        <div style={{ color: '#f0a0a0', fontSize: 11, textAlign: 'center' }}>
                            {error}
                        </div>
                    )}

                    {/* Button */}
                    <div style={{ display: 'flex', justifyContent: 'center', marginTop: 4 }}>
                        <button
                            type="submit"
                            disabled={loading}
                            style={{
                                height: 28, padding: '0 28px',
                                background: '#e0e0e0',
                                border: '1px solid #e0e0e0',
                                borderRadius: 4,
                                color: '#1a1a1a',
                                fontSize: 12,
                                fontWeight: 600,
                                fontFamily: 'inherit',
                                cursor: loading ? 'not-allowed' : 'pointer',
                                opacity: loading ? 0.5 : 1,
                            }}
                        >
                            {loading ? '...' : 'Login'}
                        </button>
                    </div>
                </form>
            </div>

            {/* Footer */}
            <div style={{ marginTop: 28, fontSize: 11, color: '#666668' }}>
                © Pi-Router
            </div>
        </div>
    )
}
