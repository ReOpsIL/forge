import { useState, useRef } from 'react'
import { Dialog } from 'primereact/dialog'
import { Button } from 'primereact/button'
import { InputText } from 'primereact/inputtext'

// Custom hook for Git functionality
export const useGit = (toastRef, projectConfigured, setShowConfigDialog) => {
    // Git-related state
    const [showBranchDialog, setShowBranchDialog] = useState(false)
    const [showCommitDialog, setShowCommitDialog] = useState(false)
    const [showMergeDialog, setShowMergeDialog] = useState(false)
    const [branchName, setBranchName] = useState('')
    const [commitMessage, setCommitMessage] = useState('')
    const [sourceBranch, setSourceBranch] = useState('')
    const [targetBranch, setTargetBranch] = useState('main')
    const [showBuildOutputDialog, setShowBuildOutputDialog] = useState(false)
    const [buildOutput, setBuildOutput] = useState('')
    const [loading, setLoading] = useState(false)

    // Function to handle building the project
    const handleBuild = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/build', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })

            const data = await response.json()
            if (response.ok) {
                // Store the build output and show the dialog
                setBuildOutput(data.output)
                setShowBuildOutputDialog(true)

                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
            } else {
                // Store the error output and show the dialog
                setBuildOutput(data.output)
                setShowBuildOutputDialog(true)

                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error building project:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to build project',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    // Function to handle pushing changes to remote repository
    const handlePush = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/push', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })

            const data = await response.json()
            if (response.ok) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error pushing changes:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to push changes',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    const handlePull = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/pull', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })

            const data = await response.json()
            if (response.ok) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error pulling changes:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to pull changes',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    // Function to handle creating a new branch
    const handleCreateBranch = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        if (!branchName.trim()) {
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Branch name cannot be empty',
                life: 5000
            })
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/branch', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ branch_name: branchName }),
            })

            const data = await response.json()
            if (response.ok) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
                setShowBranchDialog(false)
                setBranchName('')
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error creating branch:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to create branch',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    // Function to handle committing changes
    const handleCommit = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        if (!commitMessage.trim()) {
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Commit message cannot be empty',
                life: 5000
            })
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/commit', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ commit_message: commitMessage }),
            })

            const data = await response.json()
            if (response.ok) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
                setShowCommitDialog(false)
                setCommitMessage('')
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error committing changes:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to commit changes',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    // Function to handle merging branches
    const handleMerge = async () => {
        if (!projectConfigured) {
            setShowConfigDialog(true)
            return
        }

        if (!sourceBranch.trim() || !targetBranch.trim()) {
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Source and target branch names cannot be empty',
                life: 5000
            })
            return
        }

        setLoading(true)
        try {
            const response = await fetch('/api/git/merge', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    source_branch: sourceBranch,
                    target_branch: targetBranch
                }),
            })

            const data = await response.json()
            if (response.ok) {
                toastRef.current.show({
                    severity: 'success',
                    summary: 'Success',
                    detail: data.message,
                    life: 5000
                })
                setShowMergeDialog(false)
                setSourceBranch('')
                setTargetBranch('main')
            } else {
                toastRef.current.show({
                    severity: 'error',
                    summary: 'Error',
                    detail: data.message,
                    life: 5000
                })
            }
        } catch (error) {
            console.error('Error merging branches:', error)
            toastRef.current.show({
                severity: 'error',
                summary: 'Error',
                detail: 'Failed to merge branches',
                life: 5000
            })
        } finally {
            setLoading(false)
        }
    }

    // Git menu items
    const getGitMenuItems = () => {
        return {
            label: 'Git',
            icon: 'pi pi-fw pi-github',
            disabled: loading || !projectConfigured,
            items: [
                {
                    label: 'Create Branch',
                    icon: 'pi pi-fw pi-copy',
                    command: () => setShowBranchDialog(true)
                },
                {
                    label: 'Commit',
                    icon: 'pi pi-fw pi-file-arrow-up',
                    command: () => setShowCommitDialog(true)
                },
                {
                    label: 'Merge',
                    icon: 'pi pi-fw pi-code',
                    command: () => setShowMergeDialog(true)
                },
                {
                    label: 'Pull',
                    icon: 'pi pi-fw pi-cloud-download',
                    disabled: loading || !projectConfigured,
                    command: () => handlePull()
                },
                {
                    label: 'Push',
                    icon: 'pi pi-fw pi-cloud-upload',
                    disabled: loading || !projectConfigured,
                    command: () => handlePush()
                },
            ]
        }
    }

    // Build menu item
    const getBuildMenuItem = () => {
        return {
            label: 'Build',
            icon: 'pi pi-fw pi-hammer',
            disabled: loading || !projectConfigured,
            command: () => handleBuild()
        }
    }

    // Component to render Git-related dialogs
    const GitDialogs = () => (
        <>
            {/* Create Branch Dialog */}
            <Dialog
                header="Create New Branch"
                visible={showBranchDialog}
                style={{ width: '30vw' }}
                onHide={() => setShowBranchDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Create"
                            icon="pi pi-check"
                            onClick={handleCreateBranch}
                            disabled={loading || !branchName.trim()}
                        />
                        <Button
                            label="Cancel"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowBranchDialog(false)}
                            disabled={loading}
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <label htmlFor="branchName">Branch Name</label>
                        <InputText
                            id="branchName"
                            value={branchName}
                            onChange={(e) => setBranchName(e.target.value)}
                            disabled={loading}
                        />
                    </div>
                </div>
            </Dialog>

            {/* Commit Dialog */}
            <Dialog
                header="Commit Changes"
                visible={showCommitDialog}
                style={{ width: '30vw' }}
                onHide={() => setShowCommitDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Commit"
                            icon="pi pi-check"
                            onClick={handleCommit}
                            disabled={loading || !commitMessage.trim()}
                        />
                        <Button
                            label="Cancel"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowCommitDialog(false)}
                            disabled={loading}
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <label htmlFor="commitMessage">Commit Message</label>
                        <InputText
                            id="commitMessage"
                            value={commitMessage}
                            onChange={(e) => setCommitMessage(e.target.value)}
                            disabled={loading}
                        />
                    </div>
                </div>
            </Dialog>

            {/* Merge Dialog */}
            <Dialog
                header="Merge Branches"
                visible={showMergeDialog}
                style={{ width: '30vw' }}
                onHide={() => setShowMergeDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Merge"
                            icon="pi pi-check"
                            onClick={handleMerge}
                            disabled={loading || !sourceBranch.trim() || !targetBranch.trim()}
                        />
                        <Button
                            label="Cancel"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowMergeDialog(false)}
                            disabled={loading}
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <label htmlFor="sourceBranch">Source Branch</label>
                        <InputText
                            id="sourceBranch"
                            value={sourceBranch}
                            onChange={(e) => setSourceBranch(e.target.value)}
                            disabled={loading}
                        />
                    </div>
                    <div className="field">
                        <label htmlFor="targetBranch">Target Branch</label>
                        <InputText
                            id="targetBranch"
                            value={targetBranch}
                            onChange={(e) => setTargetBranch(e.target.value)}
                            disabled={loading}
                        />
                    </div>
                </div>
            </Dialog>

            {/* Build Output Dialog */}
            <Dialog
                header="Build Output"
                visible={showBuildOutputDialog}
                style={{ width: '60vw' }}
                onHide={() => setShowBuildOutputDialog(false)}
                footer={
                    <div>
                        <Button
                            label="Close"
                            icon="pi pi-times"
                            className="p-button-text"
                            onClick={() => setShowBuildOutputDialog(false)}
                        />
                    </div>
                }
            >
                <div className="p-fluid">
                    <div className="field">
                        <div className="p-2 border-1 surface-border border-round mt-2" 
                             style={{
                                 backgroundColor: '#FFFFFF19', 
                                 maxHeight: '400px', 
                                 overflow: 'auto',
                                 whiteSpace: 'pre-wrap',
                                 fontFamily: 'monospace'
                             }}>
                            {buildOutput || 'No output available.'}
                        </div>
                    </div>
                </div>
            </Dialog>
        </>
    )

    return {
        // State
        showBranchDialog,
        showCommitDialog,
        showMergeDialog,
        branchName,
        commitMessage,
        sourceBranch,
        targetBranch,
        showBuildOutputDialog,
        buildOutput,
        loading,
        
        // Setters
        setShowBranchDialog,
        setShowCommitDialog,
        setShowMergeDialog,
        setBranchName,
        setCommitMessage,
        setSourceBranch,
        setTargetBranch,
        setShowBuildOutputDialog,
        setBuildOutput,
        
        // Functions
        handleBuild,
        handlePush,
        handlePull,
        handleCreateBranch,
        handleCommit,
        handleMerge,
        
        // Menu items
        getGitMenuItems,
        getBuildMenuItem,
        
        // Components
        GitDialogs
    }
}

export default useGit