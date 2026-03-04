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
            <div style={{
                fontSize: 14, fontWeight: 600,
                letterSpacing: '2px', color: '#ababab',
                marginBottom: 28,
            }}>
                PI-ROUTER
            </div>

            <div style={{
                background: '#242426',
                border: '1px solid #3a3a3c',
                borderRadius: 8,
                padding: '36px 40px',
                width: 380,
                boxShadow: '0 8px 30px rgba(0,0,0,0.1)'
            }}>
                <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                        <label style={{ fontSize: 13, color: '#ababab', fontWeight: 500 }}>Login</label>
                        <input
                            type="text"
                            value={username}
                            onChange={e => setUsername(e.target.value)}
                            required
                            autoFocus
                            style={{
                                width: '100%', height: 38,
                                padding: '0 12px',
                                background: '#1c1c1e',
                                border: '1px solid #3a3a3c',
                                borderRadius: 6,
                                color: '#f0f0f0',
                                fontSize: 14,
                                fontFamily: 'inherit',
                                outline: 'none',
                                boxSizing: 'border-box',
                                transition: 'border-color 0.2s ease'
                            }}
                            onFocus={e => (e.currentTarget.style.borderColor = '#585b70')}
                            onBlur={e => (e.currentTarget.style.borderColor = '#3a3a3c')}
                        />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                        <label style={{ fontSize: 13, color: '#ababab', fontWeight: 500 }}>Password</label>
                        <input
                            type="password"
                            value={password}
                            onChange={e => setPassword(e.target.value)}
                            style={{
                                width: '100%', height: 38,
                                padding: '0 12px',
                                background: '#1c1c1e',
                                border: '1px solid #3a3a3c',
                                borderRadius: 6,
                                color: '#f0f0f0',
                                fontSize: 14,
                                fontFamily: 'inherit',
                                outline: 'none',
                                boxSizing: 'border-box',
                                transition: 'border-color 0.2s ease'
                            }}
                            onFocus={e => (e.currentTarget.style.borderColor = '#585b70')}
                            onBlur={e => (e.currentTarget.style.borderColor = '#3a3a3c')}
                        />
                    </div>
                    {error && (
                        <div style={{ color: '#f0a0a0', fontSize: 13, textAlign: 'center' }}>
                            {error}
                        </div>
                    )}
                    <div style={{ display: 'flex', justifyContent: 'center', marginTop: 8 }}>
                        <button
                            type="submit"
                            disabled={loading}
                            style={{
                                height: 38, padding: '0 40px',
                                background: '#e0e0e0',
                                border: '1px solid #e0e0e0',
                                borderRadius: 6,
                                color: '#1a1a1a',
                                fontSize: 14,
                                fontWeight: 600,
                                fontFamily: 'inherit',
                                cursor: loading ? 'not-allowed' : 'pointer',
                                opacity: loading ? 0.7 : 1,
                                transition: 'opacity 0.2s ease, transform 0.1s ease',
                                width: '100%'
                            }}
                            onMouseEnter={e => { if (!loading) e.currentTarget.style.opacity = '0.9' }}
                            onMouseLeave={e => { if (!loading) e.currentTarget.style.opacity = '1' }}
                            onMouseDown={e => { if (!loading) e.currentTarget.style.transform = 'scale(0.98)' }}
                            onMouseUp={e => { if (!loading) e.currentTarget.style.transform = 'scale(1)' }}
                        >
                            {loading ? '...' : 'Login'}
                        </button>
                    </div>
                </form>
            </div>
            <div style={{ marginTop: 32, fontSize: 12, color: '#666668' }}>
                © Pi-Router
            </div>
        </div>
    )
}
