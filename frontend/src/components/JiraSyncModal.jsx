import React from 'react';
import {Dialog} from 'primereact/dialog';
import {ProgressBar} from 'primereact/progressbar';
import {Button} from 'primereact/button';
import {Card} from 'primereact/card';
import {Message} from 'primereact/message';

const JiraSyncModal = ({visible, onHide, progress, status, isComplete}) => {
    const dialogFooter = isComplete ? (
        <div className="flex justify-content-end">
            <Button
                label="Close"
                icon="pi pi-check"
                className="p-button-primary"
                onClick={onHide}
            />
        </div>
    ) : null;

    const getProgressColor = () => {
        if (progress === 0) return 'info';
        if (progress === 100 && status.includes('failed')) return 'error';
        if (progress === 100) return 'success';
        return 'info';
    };

    const getMessageSeverity = () => {
        if (status.includes('failed') || status.includes('error')) return 'error';
        if (status.includes('completed') || status.includes('success')) return 'success';
        return 'info';
    };

    return (
        <Dialog
            header="Jira Sync Progress"
            visible={visible}
            style={{width: '500px'}}
            onHide={isComplete ? onHide : undefined}
            footer={dialogFooter}
            modal
            closable={isComplete}
            draggable={false}
            resizable={false}
        >
            <div className="p-fluid">
                <Card className="mb-4">
                    <div className="mb-3">
                        <h4 className="m-0 mb-2">Sync Progress</h4>
                        <ProgressBar 
                            value={progress} 
                            className="mb-2"
                            color={getProgressColor()}
                        />
                        <div className="text-center text-sm text-600">
                            {progress}% Complete
                        </div>
                    </div>

                    {status && (
                        <div className="mb-3">
                            <h4 className="m-0 mb-2">Status</h4>
                            <Message 
                                severity={getMessageSeverity()}
                                text={status}
                                className="w-full"
                            />
                        </div>
                    )}

                    {!isComplete && (
                        <div className="text-center">
                            <i className="pi pi-spin pi-spinner text-2xl text-primary"></i>
                            <p className="mt-2 mb-0 text-600">
                                Syncing with Jira... Please wait.
                            </p>
                        </div>
                    )}

                    {isComplete && progress === 100 && !status.includes('failed') && (
                        <div className="text-center">
                            <i className="pi pi-check-circle text-4xl text-green-500 mb-2"></i>
                            <p className="mt-2 mb-0 text-600">
                                Sync completed successfully!
                            </p>
                        </div>
                    )}

                    {isComplete && (status.includes('failed') || status.includes('error')) && (
                        <div className="text-center">
                            <i className="pi pi-times-circle text-4xl text-red-500 mb-2"></i>
                            <p className="mt-2 mb-0 text-600">
                                Sync encountered an error.
                            </p>
                        </div>
                    )}
                </Card>

                {!isComplete && (
                    <div className="text-xs text-500 text-center">
                        <p className="mb-1">This process may take a few moments depending on the amount of data.</p>
                        <p className="m-0">Please do not close this dialog or navigate away.</p>
                    </div>
                )}
            </div>
        </Dialog>
    );
};

export default JiraSyncModal;