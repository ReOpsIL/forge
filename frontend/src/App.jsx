import { useState } from 'react'
import { Menubar } from 'primereact/menubar'
import './App.css'

function App() {
  const items = [
    {
      label: 'Blocks',
      icon: 'pi pi-fw pi-th-large',
      command: () => {
        console.log('Blocks clicked')
      }
    },
    {
      label: 'Flow',
      icon: 'pi pi-fw pi-sitemap',
      command: () => {
        console.log('Flow clicked')
      }
    },
    {
      label: 'Help',
      icon: 'pi pi-fw pi-question-circle',
      command: () => {
        console.log('Help clicked')
      }
    }
  ]

  return (
    <div className="app-container">
      <Menubar model={items} />
      <div className="content">
        <h1>Welcome to Forge</h1>
        <p>Select an option from the menu above to get started.</p>
      </div>
    </div>
  )
}

export default App