// Dashboard JavaScript
class NoRotDashboard {
    constructor() {
        this.init();
    }

    init() {
        this.loadStats();
        this.loadRecentActivity();
        this.loadConfig();
        
        // Refresh data every 30 seconds
        setInterval(() => {
            this.loadStats();
            this.loadRecentActivity();
        }, 30000);
    }

    async loadStats() {
        try {
            const response = await fetch('/api/stats');
            const stats = await response.json();
            
            document.getElementById('total-requests').textContent = stats.total_requests || 0;
            document.getElementById('blocked-requests').textContent = stats.blocked_requests || 0;
            document.getElementById('throttled-requests').textContent = stats.throttled_requests || 0;
            document.getElementById('allowed-requests').textContent = stats.allowed_requests || 0;
        } catch (error) {
            console.error('Failed to load stats:', error);
        }
    }

    async loadRecentActivity() {
        try {
            const response = await fetch('/api/recent');
            const records = await response.json();
            
            const activityList = document.getElementById('recent-activity');
            
            if (records.length === 0) {
                activityList.innerHTML = '<div class="loading">No recent activity</div>';
                return;
            }
            
            activityList.innerHTML = records.map(record => {
                const actionClass = this.getActionClass(record.action_taken);
                const actionText = this.formatAction(record.action_taken);
                const timeAgo = this.timeAgo(new Date(record.timestamp));
                
                return `
                    <div class="activity-item">
                        <div class="activity-details">
                            <div class="activity-url">${this.truncateUrl(record.url)}</div>
                            <div class="activity-meta">
                                ${record.classification || 'Unknown'} 
                                ${record.confidence ? `(${Math.round(record.confidence * 100)}%)` : ''} 
                                • ${timeAgo}
                            </div>
                        </div>
                        <div class="activity-action ${actionClass}">
                            ${actionText}
                        </div>
                    </div>
                `;
            }).join('');
            
        } catch (error) {
            console.error('Failed to load recent activity:', error);
            document.getElementById('recent-activity').innerHTML = 
                '<div class="loading">Failed to load recent activity</div>';
        }
    }

    async loadConfig() {
        try {
            const response = await fetch('/api/config');
            const config = await response.json();
            
            const configSection = document.getElementById('config-section');
            
            configSection.innerHTML = `
                <div class="config-item">
                    <h3>Filter Settings</h3>
                    <div class="config-toggle">
                        <span>Content Filtering</span>
                        <div class="toggle-switch ${config.filters.enabled ? 'active' : ''}" 
                             onclick="this.toggleConfig('filters.enabled')">
                        </div>
                    </div>
                    <div class="config-toggle">
                        <span>Content Classification</span>
                        <div class="toggle-switch ${config.classifier.enabled ? 'active' : ''}"
                             onclick="this.toggleConfig('classifier.enabled')">
                        </div>
                    </div>
                    <div class="config-toggle">
                        <span>Web UI</span>
                        <div class="toggle-switch ${config.ui.enabled ? 'active' : ''}"
                             onclick="this.toggleConfig('ui.enabled')">
                        </div>
                    </div>
                </div>
                
                <div class="config-item">
                    <h3>Filter Rules</h3>
                    ${Object.entries(config.filters.rules).map(([name, rule]) => `
                        <div class="config-toggle">
                            <span>${name.charAt(0).toUpperCase() + name.slice(1)} Rule</span>
                            <div class="toggle-switch ${rule.enabled ? 'active' : ''}"
                                 onclick="this.toggleRule('${name}')">
                            </div>
                        </div>
                        <div style="font-size: 0.9em; opacity: 0.7; margin: 5px 0;">
                            Threshold: ${rule.score_threshold}, Action: ${rule.action}
                        </div>
                    `).join('')}
                </div>
                
                <div class="config-item">
                    <h3>Classification Model</h3>
                    <p>Current model: <strong>${config.classifier.model_type}</strong></p>
                    <p>Confidence threshold: <strong>${config.classifier.confidence_threshold}</strong></p>
                </div>
            `;
            
        } catch (error) {
            console.error('Failed to load config:', error);
            document.getElementById('config-section').innerHTML = 
                '<div class="loading">Failed to load configuration</div>';
        }
    }

    getActionClass(action) {
        if (action.startsWith('blocked')) return 'action-blocked';
        if (action.startsWith('throttled')) return 'action-throttled';
        if (action.startsWith('allowed')) return 'action-allowed';
        return '';
    }

    formatAction(action) {
        if (action.startsWith('blocked')) return 'Blocked';
        if (action.startsWith('throttled')) {
            const seconds = action.split('_')[1];
            return `Throttled ${seconds}s`;
        }
        if (action.startsWith('allowed')) return 'Allowed';
        return action;
    }

    truncateUrl(url) {
        if (url.length > 60) {
            return url.substring(0, 57) + '...';
        }
        return url;
    }

    timeAgo(date) {
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMins / 60);
        const diffDays = Math.floor(diffHours / 24);

        if (diffMins < 1) return 'just now';
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        if (diffDays < 7) return `${diffDays}d ago`;
        return date.toLocaleDateString();
    }

    toggleConfig(path) {
        // This would send a request to update the configuration
        // For now, just show a message
        alert('Configuration updates will be available in a future version');
    }

    toggleRule(ruleName) {
        // This would send a request to update the rule
        // For now, just show a message
        alert(`Rule "${ruleName}" toggle will be available in a future version`);
    }
}

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new NoRotDashboard();
});