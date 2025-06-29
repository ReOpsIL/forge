import { useState, useRef } from 'react'
import { Dialog } from 'primereact/dialog'
import { Button } from 'primereact/button'
import { InputText } from 'primereact/inputtext'

// Custom hook for Git functionality
export const useTools = (toastRef, projectConfigured) => {
    // Git-related state
    const [activeView, setActiveView] = useState('home')
    const [loading, setLoading] = useState(false)


    // Import Spec menu item
    const getImportSpecMenuItem = () => {
        return {
            label: 'Import Spec',
            icon: 'pi pi-fw pi-file-import',
            disabled: loading || !projectConfigured,
            command: () => handleImportSpec()
        }
    }

    // Function to handle importing a specification
    const handleImportSpec = () => {
        // Create a file input element
        const fileInput = document.createElement('input');
        fileInput.type = 'file';
        fileInput.accept = '.md';
        fileInput.style.display = 'none';

        // Add event listener for file selection
        fileInput.addEventListener('change', async (event) => {
            const file = event.target.files[0];
            if (!file) return;

            // Read the file content
            const reader = new FileReader();
            reader.onload = async (e) => {
                const content = e.target.result;

                // Show loading state
                setLoading(true);

                try {
                    // Send the file content to the server
                    const response = await fetch('/api/blocks/process-spec', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({
                            markdown_content: content
                        }),
                    });

                    if (!response.ok) {
                        throw new Error('Failed to process specification');
                    }

                    const data = await response.json();

                    // Show success message
                    toastRef.current.show({
                        severity: 'success',
                        summary: 'Success',
                        detail: data.message,
                        life: 5000
                    });
                    setActiveView('blocks')

                } catch (error) {
                    console.error('Error processing specification:', error);
                    toastRef.current.show({
                        severity: 'error',
                        summary: 'Error',
                        detail: 'Failed to process specification',
                        life: 5000
                    });
                } finally {
                    setLoading(false);
                    // Remove the file input from the DOM
                    document.body.removeChild(fileInput);
                }
            };

            reader.readAsText(file);
        });

        // Add the file input to the DOM and trigger a click
        document.body.appendChild(fileInput);
        fileInput.click();
    }

    // Component to render Git-related dialogs

    return {
        // State
        loading,
        getImportSpecMenuItem,

    }
}

export default useTools
