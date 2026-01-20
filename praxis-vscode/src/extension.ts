/**
 * Praxis VS Code Extension
 * 
 * Spawns the Praxis AI coding agent in a terminal window.
 * Similar to Claude Code and OpenCode extensions.
 */

import * as vscode from 'vscode';

let praxisTerminal: vscode.Terminal | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('Praxis extension activated');

    // Command: Start Praxis in terminal
    const startCommand = vscode.commands.registerCommand('praxis.start', () => {
        startPraxis(false);
    });

    // Command: Start Praxis in split terminal
    const startSplitCommand = vscode.commands.registerCommand('praxis.startInSplit', () => {
        startPraxis(true);
    });

    // Clean up terminal reference when it's closed
    context.subscriptions.push(
        vscode.window.onDidCloseTerminal(terminal => {
            if (terminal === praxisTerminal) {
                praxisTerminal = undefined;
            }
        })
    );

    context.subscriptions.push(startCommand, startSplitCommand);
}

function startPraxis(splitTerminal: boolean): void {
    // If terminal exists and is still open, just show it
    if (praxisTerminal) {
        praxisTerminal.show();
        return;
    }

    const config = vscode.workspace.getConfiguration('praxis');
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

    // Build command with options
    const executablePath = config.get<string>('executablePath') || 'praxis';
    const args: string[] = [];

    // Add optional arguments based on settings
    const orchestratorModel = config.get<string>('orchestratorModel');
    if (orchestratorModel) {
        args.push(`-o "${orchestratorModel}"`);
    }

    const executorModel = config.get<string>('executorModel');
    if (executorModel) {
        args.push(`-e "${executorModel}"`);
    }

    if (config.get<boolean>('debug')) {
        args.push('-d');
    }

    if (config.get<boolean>('disableBrowser')) {
        args.push('--no-browser');
    }

    const command = args.length > 0
        ? `${executablePath} ${args.join(' ')}`
        : executablePath;

    // Create terminal options - open in editor area (side panel like OpenCode)
    const terminalOptions: vscode.TerminalOptions = {
        name: '🛰️ Praxis',
        cwd: workspaceFolder,
        iconPath: new vscode.ThemeIcon('rocket'),
        message: 'Starting Praxis AI Coding Agent...',
        location: vscode.TerminalLocation.Editor, // Opens in editor area (side panel)
    };

    // Create and show terminal
    praxisTerminal = vscode.window.createTerminal(terminalOptions);
    praxisTerminal.show();

    // Send the praxis command
    praxisTerminal.sendText(command);
}

export function deactivate() {
    // Clean up terminal if extension is deactivated
    if (praxisTerminal) {
        praxisTerminal.dispose();
        praxisTerminal = undefined;
    }
}
