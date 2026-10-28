import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import LiveDeckOverlayWindow from './components/LiveDeckOverlayWindow'
import { DeckTrackerOverlayView } from './components/deck_tracker/DeckTrackerOverlayView'
import './index.css'
import 'mana-font/css/mana.css'
import './assets/keyrune/keyrune.css'

const isDeckOverlayWindow =
  new URLSearchParams(window.location.search).get('window') === 'deck-overlay'

const isDeckTrackerOverlay =
  window.location.hash.includes('overlay/deck-tracker') ||
  window.location.pathname.includes('overlay/deck-tracker')

if (isDeckOverlayWindow || isDeckTrackerOverlay) {
  document.documentElement.classList.add('overlay-window')
  document.body.classList.add('overlay-window')
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isDeckOverlayWindow ? (
      <LiveDeckOverlayWindow />
    ) : isDeckTrackerOverlay ? (
      <DeckTrackerOverlayView />
    ) : (
      <App />
    )}
  </React.StrictMode>,
)
