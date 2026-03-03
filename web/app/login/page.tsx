"use client";
import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { login } from '@/lib/api'

export default function LoginPage() {
    const router = useRouter()
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
                window.location.href = '/' // Force hard reload to update state in layout
            }
        } catch (err: any) {
            setError(err.message.includes('401') ? 'Invalid login or password' : err.message)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100vh',
            background: '#e4e4e4',
            fontFamily: 'system-ui, -apple-system, sans-serif'
        }}>
            {/* Header Text */}
            <h1 style={{
                fontSize: 24,
                fontWeight: 700,
                color: '#333',
                letterSpacing: 2,
                marginBottom: 30,
                display: 'flex',
                alignItems: 'center',
                gap: 10
            }}>
                <span style={{ fontSize: 26 }}>⬢</span> PI-ROUTER
            </h1>

            {/* Login Box */}
            <div style={{
                background: '#f2f2f2',
                borderRadius: 8,
                padding: '30px 40px',
                width: 320,
                boxShadow: '0 4px 12px rgba(0,0,0,0.05)',
                border: '1px solid #d9d9d9',
            }}>
                <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 15 }}>

                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                        <label style={{ fontSize: 11, fontWeight: 600, width: 60, color: '#333' }}>Login</label>
                        <input
                            type="text"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            style={{
                                flex: 1,
                                padding: '6px 10px',
                                border: '1px solid #ccc',
                                borderRadius: 4,
                                fontSize: 13,
                                background: '#fff',
                                color: '#333'
                            }}
                            required
                        />
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                        <label style={{ fontSize: 11, fontWeight: 600, width: 60, color: '#333' }}>Password</label>
                        <input
                            type="password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            style={{
                                flex: 1,
                                padding: '6px 10px',
                                border: '1px solid #ccc',
                                borderRadius: 4,
                                fontSize: 13,
                                background: '#fff',
                                color: '#333'
                            }}
                        />
                    </div>

                    {error && (
                        <div style={{ color: '#d32f2f', fontSize: 12, textAlign: 'center', marginTop: 5 }}>
                            {error}
                        </div>
                    )}

                    <button
                        type="submit"
                        disabled={loading}
                        style={{
                            background: '#0d5c63',
                            color: '#fff',
                            border: 'none',
                            padding: '8px 24px',
                            borderRadius: 4,
                            fontSize: 12,
                            fontWeight: 600,
                            cursor: loading ? 'not-allowed' : 'pointer',
                            marginTop: 10,
                            alignSelf: 'center',
                            opacity: loading ? 0.7 : 1
                        }}
                    >
                        {loading ? '...' : 'Login'}
                    </button>
                </form>
            </div>

            <div style={{ marginTop: 40, fontSize: 11, color: '#888' }}>
                © Pi-Router Gateway
            </div>
        </div>
    )
}
