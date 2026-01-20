"use strict";
/**
 * Praxis VS Code Extension
 *
 * Spawns the Praxis AI coding agent in a terminal window.
 * Similar to Claude Code and OpenCode extensions.
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
let praxisTerminal;
function activate(context) {
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
    context.subscriptions.push(vscode.window.onDidCloseTerminal(terminal => {
        if (terminal === praxisTerminal) {
            praxisTerminal = undefined;
        }
    }));
    context.subscriptions.push(startCommand, startSplitCommand);
}
function startPraxis(splitTerminal) {
    // If terminal exists and is still open, just show it
    if (praxisTerminal) {
        praxisTerminal.show();
        return;
    }
    const config = vscode.workspace.getConfiguration('praxis');
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    // Build command with options
    const executablePath = config.get('executablePath') || 'praxis';
    const args = [];
    // Add optional arguments based on settings
    const orchestratorModel = config.get('orchestratorModel');
    if (orchestratorModel) {
        args.push(`-o "${orchestratorModel}"`);
    }
    const executorModel = config.get('executorModel');
    if (executorModel) {
        args.push(`-e "${executorModel}"`);
    }
    if (config.get('debug')) {
        args.push('-d');
    }
    if (config.get('disableBrowser')) {
        args.push('--no-browser');
    }
    const command = args.length > 0
        ? `${executablePath} ${args.join(' ')}`
        : executablePath;
    // Create terminal options - open in editor area (side panel like OpenCode)
    const terminalOptions = {
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
function deactivate() {
    // Clean up terminal if extension is deactivated
    if (praxisTerminal) {
        praxisTerminal.dispose();
        praxisTerminal = undefined;
    }
}
//# sourceMappingURL=extension.js.map