import { useState } from 'react'
import { Menubar } from 'primereact/menubar'
import BlocksView from './components/BlocksView'
import FlowView from './components/FlowView'
import './App.css'

function App() {
  const [activeView, setActiveView] = useState('home')

  const items = [
    {
      label: 'Blocks',
      icon: 'pi pi-fw pi-th-large',
      command: () => {
        setActiveView('blocks')
      }
    },
    {
      label: 'Flow',
      icon: 'pi pi-fw pi-sitemap',
      command: () => {
        setActiveView('flow')
        console.log('Flow clicked')
      }
    },
    {
      label: 'Help',
      icon: 'pi pi-fw pi-question-circle',
      command: () => {
        setActiveView('help')
        console.log('Help clicked')
      }
    },
    {
      label: 'Home',
      icon: 'pi pi-fw pi-home',
      command: () => {
        setActiveView('home')
      }
    }
  ]

  const renderContent = () => {
    switch (activeView) {
      case 'blocks':
        return <BlocksView />
      case 'flow':
        return <FlowView />
      case 'help':
        return (
          <div className="content">
            <h1>Help</h1>
            <p>Help content is not implemented yet.</p>
          </div>
        )
      default:
        return (
          <div className="content">
            <h1>Welcome to Forge</h1>
            <p>Select an option from the menu above to get started.</p>
          </div>
        )
    }
  }

  return (
    <div className="app-container">
      <Menubar model={items} />
      {renderContent()}
    </div>
  )
}

export default App
