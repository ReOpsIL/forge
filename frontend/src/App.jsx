import {useState, useEffect, useRef} from 'react'
import {Toast} from 'primereact/toast'
import {Dialog} from 'primereact/dialog'
import {Button} from 'primereact/button'
import {Menubar} from 'primereact/menubar'
import {InputText} from 'primereact/inputtext'
import BlocksView from './components/BlocksView'
import FlowView from './components/FlowView'
import ProjectView from './components/ProjectView'
import PromptSettingsView from './components/PromptSettingsView'
import LoggerView from './components/LoggerView'
import ChatView from './components/ChatView'
import useGit from './Git'
import './App.css'

function App() {
    const [activeView, setActiveView] = useState('home')
    const [projectConfigured, setProjectConfigured] = useState(true)
    const [showConfigDialog, setShowConfigDialog] = useState(false)
    const [configMessage, setConfigMessage] = useState('')
    const toastRef = useRef(null)

    // Use the Git hook to get all git-related functionality
    const git = useGit(toastRef, projectConfigured, setShowConfigDialog)

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
        },
        {
            label: 'Logger',
            icon: 'pi pi-fw pi-list',
            disabled: !projectConfigured,
            command: () => {
                if (projectConfigured) {
                    setActiveView('logger')
                } else {
                    setShowConfigDialog(true)
                }
            }
        },
        {
            label: 'Chat',
            icon: 'pi pi-fw pi-comments',
            command: () => {
                setActiveView('chat')
            }
        },
        git.getBuildMenuItem(),
        git.getImportSpecMenuItem(),
        git.getGitMenuItems()
    ]

    const renderContent = () => {
        switch (activeView) {
            case 'blocks':
                return <BlocksView/>
            case 'flow':
                return <FlowView/>
            case 'project':
                return <ProjectView setActiveView={setActiveView}/>
            case 'promptSettings':
                return <PromptSettingsView setActiveView={setActiveView}/>
            case 'logger':
                return <LoggerView/>
            case 'chat':
                return <ChatView/>
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

    return (
        <div className="app-container">
            <Toast ref={toastRef} />

            {/* Project Configuration Dialog */}
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

            {/* Git-related dialogs */}
            <git.GitDialogs />

            <Menubar model={items} className="mb-4"/>

            {renderContent()}
        </div>
    )
}

export default App
