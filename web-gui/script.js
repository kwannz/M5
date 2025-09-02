// DeskAgent Web GUI JavaScript
class DeskAgentGUI {
    constructor() {
        this.currentView = 'dashboard';
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupKeyboardShortcuts();
        this.loadData();
        this.startAutoRefresh();
        console.log('🚀 DeskAgent Web GUI initialized');
    }

    setupEventListeners() {
        // Tab navigation
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const view = e.target.dataset.view;
                this.switchView(view);
            });
        });

        // Action buttons
        document.querySelectorAll('[data-action]').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const action = e.target.dataset.action;
                this.handleAction(action);
            });
        });

        // Tree toggles
        document.querySelectorAll('.tree-toggle').forEach(toggle => {
            toggle.addEventListener('click', (e) => {
                e.stopPropagation();
                this.toggleTreeItem(e.target);
            });
        });

        // Refresh button
        document.querySelectorAll('.action-btn').forEach(btn => {
            if (btn.textContent.includes('Refresh')) {
                btn.addEventListener('click', () => this.refreshData());
            }
        });

        // Copy buttons
        document.querySelectorAll('.rec-action').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const text = e.target.closest('.recommendation-item').querySelector('.rec-text').textContent;
                this.copyToClipboard(text);
            });
        });
    }

    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Only handle shortcuts if not typing in an input
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

            switch (e.key.toLowerCase()) {
                case 'p':
                    e.preventDefault();
                    this.handleAction('plan');
                    break;
                case 'r':
                    e.preventDefault();
                    this.switchView('review');
                    break;
                case 's':
                    e.preventDefault();
                    this.handleAction('status');
                    break;
                case 'f':
                    e.preventDefault();
                    this.handleAction('follow');
                    break;
                case 'a':
                    e.preventDefault();
                    this.handleAction('apply');
                    break;
                case 'n':
                    e.preventDefault();
                    this.handleAction('notify');
                    break;
                case '1':
                    e.preventDefault();
                    this.switchView('dashboard');
                    break;
                case '2':
                    e.preventDefault();
                    this.switchView('sprint');
                    break;
                case '3':
                    e.preventDefault();
                    this.switchView('review');
                    break;
            }
        });
    }

    switchView(viewName) {
        // Update current view
        this.currentView = viewName;

        // Update tab buttons
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('active');
            if (btn.dataset.view === viewName) {
                btn.classList.add('active');
            }
        });

        // Update view content
        document.querySelectorAll('.view').forEach(view => {
            view.classList.remove('active');
        });
        
        const targetView = document.getElementById(`${viewName}-view`);
        if (targetView) {
            targetView.classList.add('active');
        }

        console.log(`📋 Switched to ${viewName} view`);
        this.trackEvent('view_switch', { view: viewName });
    }

    handleAction(action) {
        const actionHandlers = {
            plan: () => {
                console.log('📋 Plan action triggered');
                this.showNotification('Plan workflow started', 'info');
                // Simulate plan workflow
                this.simulateProgress('Planning tasks...', 2000);
            },
            review: () => {
                console.log('🔍 Review action triggered');
                this.switchView('review');
            },
            status: () => {
                console.log('📊 Status action triggered');
                this.showNotification('Status report generated', 'success');
            },
            follow: () => {
                console.log('🔄 Follow-up action triggered');
                this.showNotification('Follow-up workflow started', 'info');
            },
            apply: () => {
                console.log('🛠️ Apply action triggered');
                this.showNotification('Applying changes...', 'info');
                this.simulateProgress('Applying changes...', 3000);
            },
            notify: () => {
                console.log('📢 Notify action triggered');
                this.showNotification('Notifications sent', 'success');
            },
            approve: () => {
                console.log('✅ Approve action triggered');
                this.showNotification('Review approved', 'success');
                this.updateReviewStatus('approved');
            },
            'request-changes': () => {
                console.log('❌ Request changes action triggered');
                this.showNotification('Changes requested', 'warning');
                this.updateReviewStatus('changes-requested');
            },
            export: () => {
                console.log('📄 Export action triggered');
                this.exportReport();
            },
            'view-diff': () => {
                console.log('🔍 View diff action triggered');
                this.showNotification('Opening diff viewer...', 'info');
            }
        };

        const handler = actionHandlers[action];
        if (handler) {
            handler();
            this.trackEvent('action_click', { action });
        } else {
            console.warn(`⚠️ Unknown action: ${action}`);
        }
    }

    toggleTreeItem(toggle) {
        const isExpanded = toggle.textContent === '▼';
        toggle.textContent = isExpanded ? '▶' : '▼';
        
        // In a real implementation, this would show/hide child items
        console.log(`📁 Tree item ${isExpanded ? 'collapsed' : 'expanded'}`);
    }

    loadData() {
        console.log('📊 Loading application data...');
        
        // Simulate loading data from files
        setTimeout(() => {
            this.updateDashboardData();
            this.updateSprintData();
            this.updateReviewData();
            console.log('✅ Data loaded successfully');
        }, 500);
    }

    updateDashboardData() {
        // Update progress bar animation
        const progressBar = document.querySelector('.progress-fill');
        if (progressBar) {
            progressBar.style.width = '75%';
        }

        // Update activity timestamps
        const timeElements = document.querySelectorAll('.activity-time');
        timeElements.forEach((el, index) => {
            const now = new Date();
            now.setMinutes(now.getMinutes() - (index * 30 + 20));
            el.textContent = now.toLocaleTimeString('en-US', { 
                hour: '2-digit', 
                minute: '2-digit',
                hour12: false 
            });
        });
    }

    updateSprintData() {
        // Update sprint statistics
        const stats = {
            total: 12,
            completed: 8,
            inProgress: 3,
            failed: 1
        };
        
        console.log('📋 Sprint stats updated:', stats);
    }

    updateReviewData() {
        // Update review quality score with animation
        const qualityScore = 92;
        console.log('🔍 Review quality score:', qualityScore);
    }

    refreshData() {
        console.log('🔄 Refreshing data...');
        this.showNotification('Refreshing data...', 'info');
        
        // Simulate refresh delay
        setTimeout(() => {
            this.loadData();
            this.showNotification('Data refreshed', 'success');
        }, 1000);
    }

    simulateProgress(message, duration) {
        this.showNotification(message, 'info');
        
        // Create a simple progress simulation
        let progress = 0;
        const interval = setInterval(() => {
            progress += 10;
            if (progress >= 100) {
                clearInterval(interval);
                this.showNotification('Operation completed', 'success');
            }
        }, duration / 10);
    }

    updateReviewStatus(status) {
        const statusElement = document.querySelector('.review-stats .stat .success');
        if (statusElement) {
            const statusMap = {
                approved: '✅ Approved',
                'changes-requested': '❌ Changes Requested',
                pending: '⏳ Pending'
            };
            statusElement.textContent = statusMap[status] || status;
        }
    }

    exportReport() {
        const reportData = {
            timestamp: new Date().toISOString(),
            sprint: 'sprint-01',
            quality_score: 92,
            files_changed: 5,
            recommendations: 3,
            status: 'approved'
        };
        
        const blob = new Blob([JSON.stringify(reportData, null, 2)], {
            type: 'application/json'
        });
        
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'deskagent-review-report.json';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        this.showNotification('Report exported', 'success');
    }

    copyToClipboard(text) {
        navigator.clipboard.writeText(text).then(() => {
            this.showNotification('Copied to clipboard', 'success');
        }).catch(() => {
            this.showNotification('Failed to copy', 'error');
        });
    }

    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 20px;
            border-radius: 6px;
            color: white;
            font-weight: 500;
            z-index: 1000;
            opacity: 0;
            transform: translateX(100px);
            transition: all 0.3s ease;
        `;
        
        // Set background color based on type
        const colors = {
            info: '#2563eb',
            success: '#059669',
            warning: '#d97706',
            error: '#dc2626'
        };
        notification.style.backgroundColor = colors[type] || colors.info;
        
        notification.textContent = message;
        document.body.appendChild(notification);
        
        // Animate in
        requestAnimationFrame(() => {
            notification.style.opacity = '1';
            notification.style.transform = 'translateX(0)';
        });
        
        // Auto remove
        setTimeout(() => {
            notification.style.opacity = '0';
            notification.style.transform = 'translateX(100px)';
            setTimeout(() => {
                if (document.body.contains(notification)) {
                    document.body.removeChild(notification);
                }
            }, 300);
        }, 3000);
    }

    startAutoRefresh() {
        // Refresh data every 30 seconds
        setInterval(() => {
            this.loadData();
        }, 30000);
        
        console.log('🔄 Auto-refresh started (30s interval)');
    }

    trackEvent(event, data) {
        // In a real implementation, this would send analytics
        console.log(`📊 Event tracked: ${event}`, data);
    }
}

// Initialize the GUI when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.deskAgentGUI = new DeskAgentGUI();
    
    // Add some interactive demo features
    setTimeout(() => {
        window.deskAgentGUI.showNotification('🤖 DeskAgent Web GUI loaded successfully!', 'success');
    }, 1000);
});

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DeskAgentGUI;
}