import {useState, useEffect, useRef} from 'react'
import {Toast} from 'primereact/toast'
import {Dialog} from 'primereact/dialog'
import {Button} from 'primereact/button'
import {Menubar} from 'primereact/menubar'
import BlocksView from './components/BlocksView'
import FlowView from './components/FlowView'
import ProjectView from './components/ProjectView'
import './App.css'

function App() {
    const [activeView, setActiveView] = useState('home')
    const [projectConfigured, setProjectConfigured] = useState(true)
    const [showConfigDialog, setShowConfigDialog] = useState(false)
    const [configMessage, setConfigMessage] = useState('')
    const toastRef = useRef(null)

    useEffect(() => {
        checkProjectConfig()
    }, [])

    const checkProjectConfig = async () => {
        try {
            const response = await fetch('/api/project/check-config')
            if (!response.ok) {
                throw new Error('Failed to check project configuration')
            }

            const data = await response.json()
            setProjectConfigured(data.configured)
            setConfigMessage(data.message)

            if (!data.configured) {
                setShowConfigDialog(true)
            }
        } catch (error) {
            console.error('Error checking project configuration:', error)
            setProjectConfigured(false)
            setConfigMessage('Error checking project configuration')
            setShowConfigDialog(true)
        }
    }

    const items = [
        {
            label: 'Project',
            icon: 'pi pi-fw pi-cog',
            command: () => {
                setActiveView('project')
            }
        },
        {
            label: 'Blocks',
            icon: 'pi pi-fw pi-th-large',
            disabled: !projectConfigured,
            command: () => {
                if (projectConfigured) {
                    setActiveView('blocks')
                } else {
                    setShowConfigDialog(true)
                }
            }
        },
        {
            label: 'Flow',
            icon: 'pi pi-fw pi-sitemap',
            disabled: !projectConfigured,
            command: () => {
                if (projectConfigured) {
                    setActiveView('flow')
                } else {
                    setShowConfigDialog(true)
                }
            }
        }
    ]

    const renderContent = () => {
        switch (activeView) {
            case 'blocks':
                return <BlocksView/>
            case 'flow':
                return <FlowView/>
            case 'project':
                return <ProjectView/>
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
            <Toast ref={toastRef} />
            <Dialog
                header="Project Configuration Required"
                visible={showConfigDialog}
                style={{ width: '50vw' }}
                onHide={() => setShowConfigDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Configure Project"
                            icon="pi pi-cog"
                            onClick={() => {
                                setActiveView('project')
                                setShowConfigDialog(false)
                            }}
                        />
                        <Button
                            label="Close"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowConfigDialog(false)}
                        />
                    </div>
                }
            >
                <p>{configMessage}</p>
                <p>Please configure your project settings before accessing Blocks and Flow views.</p>
            </Dialog>
            <Menubar model={items}/>
            {renderContent()}
        </div>
    )
}

export default App
