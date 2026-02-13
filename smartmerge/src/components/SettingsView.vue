<!--
  SettingsView Component - Configuration UI for SmartMerge
  
  Provides a comprehensive settings interface for:
  - Diff algorithm selection
  - Highlighting themes
  - Diff behavior options
  - Merge resolution preferences
  - UI preferences
-->

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { DiffAlgorithm, Theme, ResolutionStrategy } from '@/api/smartmerge';
import ColorSchemeEditor from './ColorSchemeEditor.vue';

// Settings interface
interface SmartMergeSettings {
  diff: {
    algorithm: DiffAlgorithm;
    ignoreWhitespace: boolean;
    ignoreCase: boolean;
    detectMovedBlocks: boolean;
    contextLines: number;
  };
  highlighting: {
    theme: Theme;
    enableInlineHighlights: boolean;
    showLineNumbers: boolean;
    syntaxHighlighting: boolean;
  };
  merge: {
    defaultStrategy: ResolutionStrategy;
    autoResolveSimple: boolean;
    includeConflictMarkers: boolean;
    maxHistorySize: number;
  };
  ui: {
    splitViewRatio: number;
    fontSize: number;
    tabSize: number;
    wrapLines: boolean;
  };
}

// Default settings
const defaultSettings: SmartMergeSettings = {
  diff: {
    algorithm: 'myers',
    ignoreWhitespace: false,
    ignoreCase: false,
    detectMovedBlocks: true,
    contextLines: 3,
  },
  highlighting: {
    theme: 'auto',
    enableInlineHighlights: true,
    showLineNumbers: true,
    syntaxHighlighting: true,
  },
  merge: {
    defaultStrategy: 'prefer_left',
    autoResolveSimple: false,
    includeConflictMarkers: true,
    maxHistorySize: 100,
  },
  ui: {
    splitViewRatio: 50,
    fontSize: 13,
    tabSize: 4,
    wrapLines: false,
  },
};

// Settings state
const settings = ref<SmartMergeSettings>(JSON.parse(JSON.stringify(defaultSettings)));
const hasChanges = ref(false);
const savedNotification = ref(false);

// Active tab
const activeTab = ref<'diff' | 'highlighting' | 'merge' | 'ui' | 'colors'>('diff');

// Emits
const emit = defineEmits<{
  'settings-changed': [settings: SmartMergeSettings];
}>();

// Load settings on mount
onMounted(() => {
  loadSettings();
});

// Watch for changes
watch(
  settings,
  () => {
    hasChanges.value = true;
  },
  { deep: true }
);

// Storage key
const STORAGE_KEY = 'smartmerge_settings';

// Load settings from localStorage
function loadSettings() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      settings.value = { ...defaultSettings, ...parsed };
    }
  } catch (err) {
    console.error('Failed to load settings:', err);
  }
}

// Save settings to localStorage
function saveSettings() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    hasChanges.value = false;
    savedNotification.value = true;
    
    // Emit settings change event
    emit('settings-changed', settings.value);
    
    // Hide notification after 2 seconds
    setTimeout(() => {
      savedNotification.value = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to save settings:', err);
  }
}

// Reset to defaults
function resetToDefaults() {
  if (confirm('Are you sure you want to reset all settings to defaults?')) {
    settings.value = JSON.parse(JSON.stringify(defaultSettings));
    saveSettings();
  }
}

// Export settings
function exportSettings() {
  const json = JSON.stringify(settings.value, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'smartmerge-settings.json';
  a.click();
  URL.revokeObjectURL(url);
}

// Import settings
function importSettings(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  
  const reader = new FileReader();
  reader.onload = (e) => {
    try {
      const imported = JSON.parse(e.target?.result as string);
      settings.value = { ...defaultSettings, ...imported };
      saveSettings();
    } catch (err) {
      alert('Failed to import settings: Invalid JSON file');
    }
  };
  reader.readAsText(file);
}

// Get current settings
function getCurrentSettings(): SmartMergeSettings {
  return settings.value;
}

// Expose for parent
defineExpose({
  getCurrentSettings,
  saveSettings,
});
</script>

<template>
  <div class="settings-view">
    <!-- Header -->
    <div class="settings-header">
      <h2>Settings</h2>
      <div class="header-actions">
        <button @click="exportSettings" class="action-btn">
          Export
        </button>
        <label class="action-btn">
          Import
          <input 
            type="file" 
            accept=".json" 
            @change="importSettings"
            style="display: none"
          />
        </label>
        <button @click="resetToDefaults" class="action-btn action-btn--danger">
          Reset to Defaults
        </button>
        <button 
          @click="saveSettings" 
          :disabled="!hasChanges"
          class="action-btn action-btn--primary"
        >
          Save Changes
        </button>
      </div>
    </div>

    <!-- Saved notification -->
    <div v-if="savedNotification" class="notification notification--success">
      ✓ Settings saved successfully!
    </div>

    <!-- Tabs -->
    <div class="settings-tabs">
      <button 
        @click="activeTab = 'diff'"
        :class="{ active: activeTab === 'diff' }"
        class="tab-btn"
      >
        Diff Settings
      </button>
      <button 
        @click="activeTab = 'highlighting'"
        :class="{ active: activeTab === 'highlighting' }"
        class="tab-btn"
      >
        Highlighting
      </button>
      <button 
        @click="activeTab = 'merge'"
        :class="{ active: activeTab === 'merge' }"
        class="tab-btn"
      >
        Merge Settings
      </button>
      <button 
        @click="activeTab = 'ui'"
        :class="{ active: activeTab === 'ui' }"
        class="tab-btn"
      >
        UI Preferences
      </button>
      <button 
        @click="activeTab = 'colors'"
        :class="{ active: activeTab === 'colors' }"
        class="tab-btn"
      >
        Color Scheme
      </button>
    </div>

    <!-- Content -->
    <div class="settings-content">
      <!-- Diff Settings -->
      <div v-show="activeTab === 'diff'" class="settings-section">
        <h3>Diff Algorithm & Behavior</h3>
        
        <div class="setting-group">
          <label class="setting-label">
            Diff Algorithm
            <span class="setting-description">
              Choose the algorithm for computing differences
            </span>
          </label>
          <select v-model="settings.diff.algorithm" class="setting-select">
            <option value="myers">Myers (Default, balanced)</option>
            <option value="smart">Smart (Context-aware, better for code)</option>
            <option value="patience">Patience (Best for moved code)</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-label">
            Context Lines
            <span class="setting-description">
              Number of unchanged lines to show around changes
            </span>
          </label>
          <input 
            v-model.number="settings.diff.contextLines"
            type="number"
            min="0"
            max="10"
            class="setting-input"
          />
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.diff.ignoreWhitespace"
              type="checkbox"
            />
            <span>
              Ignore Whitespace
              <span class="setting-description">
                Ignore changes in whitespace when comparing
              </span>
            </span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.diff.ignoreCase"
              type="checkbox"
            />
            <span>
              Ignore Case
              <span class="setting-description">
                Treat uppercase and lowercase as identical
              </span>
            </span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.diff.detectMovedBlocks"
              type="checkbox"
            />
            <span>
              Detect Moved Blocks
              <span class="setting-description">
                Identify code that was moved (not added/deleted)
              </span>
            </span>
          </label>
        </div>
      </div>

      <!-- Highlighting Settings -->
      <div v-show="activeTab === 'highlighting'" class="settings-section">
        <h3>Highlighting & Display</h3>
        
        <div class="setting-group">
          <label class="setting-label">
            Color Theme
            <span class="setting-description">
              Choose the highlighting color scheme
            </span>
          </label>
          <select v-model="settings.highlighting.theme" class="setting-select">
            <option value="auto">Auto (Follow system)</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.highlighting.enableInlineHighlights"
              type="checkbox"
            />
            <span>
              Enable Inline Highlights
              <span class="setting-description">
                Highlight character-level differences within lines
              </span>
            </span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.highlighting.showLineNumbers"
              type="checkbox"
            />
            <span>
              Show Line Numbers
              <span class="setting-description">
                Display line numbers in the diff view
              </span>
            </span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.highlighting.syntaxHighlighting"
              type="checkbox"
            />
            <span>
              Syntax Highlighting
              <span class="setting-description">
                Apply syntax highlighting to code (future feature)
              </span>
            </span>
          </label>
        </div>
      </div>

      <!-- Merge Settings -->
      <div v-show="activeTab === 'merge'" class="settings-section">
        <h3>Merge & Conflict Resolution</h3>
        
        <div class="setting-group">
          <label class="setting-label">
            Default Resolution Strategy
            <span class="setting-description">
              Default strategy for auto-resolving conflicts
            </span>
          </label>
          <select v-model="settings.merge.defaultStrategy" class="setting-select">
            <option value="prefer_left">Prefer Left Version</option>
            <option value="prefer_right">Prefer Right Version</option>
            <option value="prefer_both">Prefer Both (Combine)</option>
            <option value="prefer_base">Prefer Base Version</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-label">
            Max Undo History Size
            <span class="setting-description">
              Maximum number of undo/redo operations to keep
            </span>
          </label>
          <input 
            v-model.number="settings.merge.maxHistorySize"
            type="number"
            min="10"
            max="500"
            class="setting-input"
          />
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.merge.autoResolveSimple"
              type="checkbox"
            />
            <span>
              Auto-resolve Simple Conflicts
              <span class="setting-description">
                Automatically resolve non-overlapping conflicts
              </span>
            </span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.merge.includeConflictMarkers"
              type="checkbox"
            />
            <span>
              Include Conflict Markers
              <span class="setting-description">
                Add &lt;&lt;&lt;, ===, &gt;&gt;&gt; markers in merge output
              </span>
            </span>
          </label>
        </div>
      </div>

      <!-- UI Settings -->
      <div v-show="activeTab === 'ui'" class="settings-section">
        <h3>User Interface</h3>
        
        <div class="setting-group">
          <label class="setting-label">
            Split View Ratio
            <span class="setting-description">
              Left/right panel width ratio ({{ settings.ui.splitViewRatio }}% / {{ 100 - settings.ui.splitViewRatio }}%)
            </span>
          </label>
          <input 
            v-model.number="settings.ui.splitViewRatio"
            type="range"
            min="30"
            max="70"
            class="setting-range"
          />
        </div>

        <div class="setting-group">
          <label class="setting-label">
            Font Size
            <span class="setting-description">
              Editor font size in pixels
            </span>
          </label>
          <input 
            v-model.number="settings.ui.fontSize"
            type="number"
            min="10"
            max="24"
            class="setting-input"
          />
        </div>

        <div class="setting-group">
          <label class="setting-label">
            Tab Size
            <span class="setting-description">
              Number of spaces per tab
            </span>
          </label>
          <input 
            v-model.number="settings.ui.tabSize"
            type="number"
            min="2"
            max="8"
            class="setting-input"
          />
        </div>

        <div class="setting-group">
          <label class="setting-checkbox">
            <input 
              v-model="settings.ui.wrapLines"
              type="checkbox"
            />
            <span>
              Wrap Long Lines
              <span class="setting-description">
                Wrap lines that exceed viewport width
              </span>
            </span>
          </label>
        </div>
      </div>

      <!-- Color Scheme -->
      <div v-show="activeTab === 'colors'" class="settings-section">
        <ColorSchemeEditor />
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Header */
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  background: var(--color-background-soft);
  border-bottom: 2px solid var(--color-border);
}

.settings-header h2 {
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  padding: 0.5rem 1rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn--primary {
  background: var(--color-primary, #3b82f6);
  color: white;
  border-color: var(--color-primary, #3b82f6);
}

.action-btn--danger {
  background: var(--color-danger, #dc2626);
  color: white;
  border-color: var(--color-danger, #dc2626);
}

/* Notification */
.notification {
  padding: 0.75rem 1.5rem;
  text-align: center;
  font-weight: 500;
}

.notification--success {
  background: var(--color-success-bg, #d1fae5);
  color: var(--color-success, #16a34a);
}

/* Tabs */
.settings-tabs {
  display: flex;
  gap: 0;
  background: var(--color-background-mute);
  border-bottom: 1px solid var(--color-border);
}

.tab-btn {
  flex: 1;
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: var(--color-background-soft);
}

.tab-btn.active {
  background: var(--color-background);
  border-bottom-color: var(--color-primary, #3b82f6);
  color: var(--color-primary, #3b82f6);
}

/* Content */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 2rem;
}

.settings-section {
  max-width: 800px;
  margin: 0 auto;
}

.settings-section h3 {
  margin: 0 0 1.5rem 0;
  padding-bottom: 0.5rem;
  border-bottom: 2px solid var(--color-border);
}

/* Setting groups */
.setting-group {
  margin-bottom: 1.5rem;
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.setting-description {
  font-weight: 400;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.setting-select,
.setting-input {
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-background);
  width: 100%;
  max-width: 400px;
}

.setting-range {
  width: 100%;
  max-width: 400px;
}

.setting-checkbox {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  cursor: pointer;
  user-select: none;
}

.setting-checkbox input[type='checkbox'] {
  margin-top: 0.25rem;
  cursor: pointer;
}

.setting-checkbox span {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
</style>
