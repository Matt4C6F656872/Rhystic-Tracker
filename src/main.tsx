import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import LiveDeckOverlayWindow from './components/LiveDeckOverlayWindow'
import './index.css'
import 'mana-font/css/mana.css'
import './assets/keyrune/keyrune.css'

const isDeckOverlayWindow =
  new URLSearchParams(window.location.search).get('window') === 'deck-overlay'

if (isDeckOverlayWindow) {
  document.documentElement.classList.add('overlay-window')
  document.body.classList.add('overlay-window')
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isDeckOverlayWindow ? <LiveDeckOverlayWindow /> : <App />}
  </React.StrictMode>,
)
