import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.jsx'
import './index.css'

// PrimeReact imports
import 'primereact/resources/themes/lara-dark-teal/theme.css'  // theme
import 'primereact/resources/primereact.min.css'                  // core css
import 'primeicons/primeicons.css'                                // icons
import 'primeflex/primeflex.css'                                  // primeflex

ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
        <App/>
    </React.StrictMode>,
)
