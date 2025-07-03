import {useEffect, useRef, useState} from 'react'
import {Toast} from 'primereact/toast'
import {Dialog} from 'primereact/dialog'
import {Button} from 'primereact/button'
import {Menubar} from 'primereact/menubar'
import BlocksView from './components/BlocksView'
import ProjectView from './components/ProjectView'
import PromptSettingsView from './components/PromptSettingsView'
import Terminal from './components/Terminal'
import useTools from './Tools'
import './App.css'

function App() {
    const [activeView, setActiveView] = useState('home')
    const [projectConfigured, setProjectConfigured] = useState(true)
    const [showConfigDialog, setShowConfigDialog] = useState(false)
    const [configMessage, setConfigMessage] = useState('')
    const [blocksRefreshTrigger, setBlocksRefreshTrigger] = useState(0)
    const [terminalPanelWidth, setTerminalPanelWidth] = useState(400)
    const [terminalPanelCollapsed, setTerminalPanelCollapsed] = useState(false)
    const [isResizing, setIsResizing] = useState(false)
    const toastRef = useRef(null)
    const resizerRef = useRef(null)

    // Use the Git hook to get all git-related functionality
    const tools = useTools(toastRef, projectConfigured)

    useEffect(() => {
        checkProjectConfig()
    }, [])

    // Handle panel resizing
    useEffect(() => {
        const handleMouseMove = (e) => {
            if (!isResizing) return

            const containerWidth = window.innerWidth
            const newWidth = containerWidth - e.clientX

            // Constrain width between 200px and 80% of window width
            const minWidth = 200
            const maxWidth = containerWidth * 0.8
            const constrainedWidth = Math.max(minWidth, Math.min(maxWidth, newWidth))

            setTerminalPanelWidth(constrainedWidth)
        }

        const handleMouseUp = () => {
            setIsResizing(false)
        }

        if (isResizing) {
            document.addEventListener('mousemove', handleMouseMove)
            document.addEventListener('mouseup', handleMouseUp)
            document.body.style.cursor = 'ew-resize'
            document.body.style.userSelect = 'none'
        }

        return () => {
            document.removeEventListener('mousemove', handleMouseMove)
            document.removeEventListener('mouseup', handleMouseUp)
            document.body.style.cursor = ''
            document.body.style.userSelect = ''
        }
    }, [isResizing])

    // Trigger terminal resize when panel width changes
    useEffect(() => {
        const timer = setTimeout(() => {
            // Import terminalManager from Terminal component
            if (window.terminalManager) {
                window.terminalManager.fit()
            }
        }, 100)

        return () => clearTimeout(timer)
    }, [terminalPanelWidth, terminalPanelCollapsed])

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
                    setBlocksRefreshTrigger(prev => prev + 1)
                } else {
                    setShowConfigDialog(true)
                }
            }
        },
        tools.getImportSpecMenuItem()
    ]

    const renderContent = () => {
        switch (activeView) {
            case 'blocks':
                return <BlocksView refreshTrigger={blocksRefreshTrigger}/>
            case 'project':
                return <ProjectView setActiveView={setActiveView}/>
            case 'promptSettings':
                return <PromptSettingsView setActiveView={setActiveView}/>
            case 'help':
                return (
                    <div className="content">
                        <h1>Help</h1>
                        <p>Help content is not implemented yet.</p>
                    </div>
                )
            default:
                return (
                    <div className="welcome-content"></div>
                )
        }
    }

    const handleToggleTerminal = () => {
        setTerminalPanelCollapsed(!terminalPanelCollapsed)
    }

    const handleStartResize = (e) => {
        e.preventDefault()
        setIsResizing(true)
    }

    return (
        <div className="app-container">
            <Toast ref={toastRef}/>

            {/* Project Configuration Dialog */}
            <Dialog
                header="Project Configuration Required"
                visible={showConfigDialog}
                style={{width: '50vw'}}
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
                <p>Please configure your project settings before accessing Blocks view.</p>
            </Dialog>

            <Menubar model={items} className="mb-4"/>

            <div className="main-content">
                <div
                    className="left-panel"
                    style={{
                        width: terminalPanelCollapsed
                            ? 'calc(100% - 60px)'
                            : `calc(100% - ${terminalPanelWidth}px)`
                    }}
                >
                    {renderContent()}
                </div>

                {!terminalPanelCollapsed && (
                    <div
                        className="panel-resizer"
                        onMouseDown={handleStartResize}
                        ref={resizerRef}
                    />
                )}

                <div
                    className={`right-panel ${terminalPanelCollapsed ? 'collapsed' : ''}`}
                    style={{
                        width: terminalPanelCollapsed ? '60px' : `${terminalPanelWidth}px`
                    }}
                >
                    <div className="terminal-panel-header">
                        <span className={`terminal-panel-title ${terminalPanelCollapsed ? 'collapsed' : ''}`}>
                            {!terminalPanelCollapsed && 'Terminal'}
                        </span>
                        <Button
                            icon={terminalPanelCollapsed ? 'pi pi-angle-left' : 'pi pi-angle-right'}
                            className="p-button-text p-button-sm terminal-toggle-btn"
                            onClick={handleToggleTerminal}
                            tooltip={terminalPanelCollapsed ? 'Show Terminal' : 'Hide Terminal'}
                        />
                    </div>
                    {!terminalPanelCollapsed && (
                        <div className="terminal-panel-content">
                            <Terminal/>
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}

export default App
