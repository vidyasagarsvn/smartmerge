<!--
  ColorSchemeEditor Component - Advanced color customization UI
  
  Provides comprehensive color scheme management with:
  - Preset color schemes (GitHub, Bitbucket, Solarized, VS Code)
  - Per-category color customization
  - Live preview
  - Import/export custom schemes
-->

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Type definitions
interface ChangeTypeColors {
  background: string;
  text: string;
  selected_background?: string;
  selected_text?: string;
  border?: string;
}

interface ColorScheme {
  name: string;
  version: string;
  additions: ChangeTypeColors;
  deletions: ChangeTypeColors;
  modifications: ChangeTypeColors;
  moved_blocks: ChangeTypeColors;
  trivial_changes: ChangeTypeColors;
  equal: ChangeTypeColors;
  conflicts: ChangeTypeColors;
  inline_highlight_bg: string;
  inline_highlight_text: string;
  border_color: string;
  background: string;
  text_color: string;
}

// Props
const props = defineProps<{
  theme: 'light' | 'dark';
}>();

// Emits
const emit = defineEmits<{
  'scheme-changed': [scheme: ColorScheme];
}>();

// State
const availableSchemes = ref<Array<[string, string]>>([]);
const selectedSchemeId = ref<string>('github-light');
const currentScheme = ref<ColorScheme | null>(null);
const editingColor = ref<string | null>(null);
const editingValue = ref<string>('');
const showColorPicker = ref(false);

// Categories for display
const categories = [
  { key: 'additions', label: 'Additions (Green)', icon: '➕' },
  { key: 'deletions', label: 'Deletions (Red)', icon: '➖' },
  { key: 'modifications', label: 'Modifications (Yellow)', icon: '📝' },
  { key: 'moved_blocks', label: 'Moved Blocks (Blue)', icon: '↔️' },
  { key: 'trivial_changes', label: 'Trivial Changes (Gray)', icon: '⚪' },
  { key: 'equal', label: 'Equal/Unchanged (White)', icon: '✓' },
  { key: 'conflicts', label: 'Conflicts (Red)', icon: '⚠️' },
];

// Computed
const selectedSchemeName = computed(() => {
  return availableSchemes.value.find(([id]) => id === selectedSchemeId.value)?.[1] || 'Custom';
});

// Load available schemes
onMounted(async () => {
  try {
    const schemes = await invoke<Array<[string, string]>>('get_available_color_schemes');
    availableSchemes.value = schemes;
    
    // Load initial scheme
    await loadScheme(selectedSchemeId.value);
  } catch (error) {
    console.error('Failed to load color schemes:', error);
  }
});

// Load a color scheme
async function loadScheme(schemeId: string) {
  try {
    const scheme = await invoke<ColorScheme>('get_color_scheme', { schemeId });
    currentScheme.value = scheme;
    selectedSchemeId.value = schemeId;
    emit('scheme-changed', scheme);
  } catch (error) {
    console.error('Failed to load scheme:', error);
  }
}

// Handle color change
function updateColor(colorPath: string, value: string) {
  if (!currentScheme.value) return;
  
  // Parse path like "additions.background"
  const parts = colorPath.split('.');
  if (parts.length === 2) {
    const [category, property] = parts;
    (currentScheme.value as any)[category][property] = value;
    emit('scheme-changed', currentScheme.value);
  } else if (parts.length === 1) {
    // Top-level properties
    (currentScheme.value as any)[colorPath] = value;
    emit('scheme-changed', currentScheme.value);
  }
}

// Start editing a color
function startEditColor(path: string, currentValue: string) {
  editingColor.value = path;
  editingValue.value = currentValue;
  showColorPicker.value = true;
}

// Save color edit
function saveColorEdit() {
  if (editingColor.value && editingValue.value) {
    updateColor(editingColor.value, editingValue.value);
  }
  closeColorPicker();
}

// Close color picker
function closeColorPicker() {
  editingColor.value = null;
  editingValue.value = '';
  showColorPicker.value = false;
}

// Export scheme
function exportScheme() {
  if (!currentScheme.value) return;
  const json = JSON.stringify(currentScheme.value, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${currentScheme.value.name.toLowerCase().replace(/\s+/g, '-')}.json`;
  link.click();
  URL.revokeObjectURL(url);
}

// Import scheme
function importScheme() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.json';
  input.onchange = async (e: any) => {
    const file = e.target.files[0];
    if (!file) return;
    
    const text = await file.text();
    try {
      const scheme = JSON.parse(text) as ColorScheme;
      currentScheme.value = scheme;
      selectedSchemeId.value = 'custom';
      emit('scheme-changed', scheme);
    } catch (error) {
      console.error('Failed to parse scheme file:', error);
    }
  };
  input.click();
}

// Reset to selected preset
function reset() {
  loadScheme(selectedSchemeId.value);
}
</script>

<template>
  <div class="color-scheme-editor">
    <!-- Header -->
    <div class="editor-header">
      <h3>Color Scheme: {{ selectedSchemeName }}</h3>
      <div class="header-actions">
        <button @click="exportScheme" class="action-btn" title="Export color scheme">
          📥 Export
        </button>
        <button @click="importScheme" class="action-btn" title="Import color scheme">
          📤 Import
        </button>
        <button v-if="selectedSchemeId !== 'custom'" @click="reset" class="action-btn" title="Reset to preset">
          🔄 Reset
        </button>
      </div>
    </div>

    <!-- Preset Selection -->
    <div class="preset-section">
      <label for="scheme-select">Preset Schemes:</label>
      <select
        id="scheme-select"
        v-model="selectedSchemeId"
        @change="loadScheme(selectedSchemeId)"
        class="scheme-select"
      >
        <option value="custom" disabled>Custom</option>
        <optgroup label="Light Themes">
          <option value="github-light">GitHub Light</option>
          <option value="bitbucket">Bitbucket</option>
          <option value="solarized-light">Solarized Light</option>
          <option value="vscode-light">VS Code Light</option>
        </optgroup>
        <optgroup label="Dark Themes">
          <option value="github-dark">GitHub Dark</option>
          <option value="solarized-dark">Solarized Dark</option>
          <option value="vscode-dark">VS Code Dark</option>
        </optgroup>
      </select>
    </div>

    <!-- Categories -->
    <div class="categories-grid">
      <div v-for="category in categories" :key="category.key" class="color-category">
        <div class="category-header">
          <span class="category-icon">{{ category.icon }}</span>
          <span class="category-name">{{ category.label }}</span>
        </div>

        <div v-if="currentScheme" class="color-group">
          <!-- Background Color -->
          <div class="color-row">
            <label>Background:</label>
            <div class="color-input-group">
              <div
                class="color-swatch"
                :style="{ backgroundColor: (currentScheme as any)[category.key]?.background || '#fff' }"
                @click="startEditColor(`${category.key}.background`, (currentScheme as any)[category.key]?.background)"
              ></div>
              <input
                type="text"
                :value="(currentScheme as any)[category.key]?.background || '#'"
                @input="(e: any) => updateColor(`${category.key}.background`, e.target.value)"
                class="color-text-input"
                placeholder="#ffffff"
              />
            </div>
          </div>

          <!-- Text Color -->
          <div class="color-row">
            <label>Text:</label>
            <div class="color-input-group">
              <div
                class="color-swatch"
                :style="{ backgroundColor: (currentScheme as any)[category.key]?.text || '#000' }"
                @click="startEditColor(`${category.key}.text`, (currentScheme as any)[category.key]?.text)"
              ></div>
              <input
                type="text"
                :value="(currentScheme as any)[category.key]?.text || '#'"
                @input="(e: any) => updateColor(`${category.key}.text`, e.target.value)"
                class="color-text-input"
                placeholder="#000000"
              />
            </div>
          </div>

          <!-- Selected Background (if exists) -->
          <div v-if="(currentScheme as any)[category.key]?.selected_background" class="color-row">
            <label>Selected (Bg):</label>
            <div class="color-input-group">
              <div
                class="color-swatch"
                :style="{ backgroundColor: (currentScheme as any)[category.key]?.selected_background || '#fff' }"
              ></div>
              <input
                type="text"
                :value="(currentScheme as any)[category.key]?.selected_background || '#'"
                @input="(e: any) => updateColor(`${category.key}.selected_background`, e.target.value)"
                class="color-text-input"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Global Colors -->
    <div class="global-colors">
      <h4>Global Colors</h4>
      <div v-if="currentScheme" class="global-grid">
        <div class="global-color-row">
          <label>Inline Highlight BG:</label>
          <div class="color-input-group">
            <div
              class="color-swatch"
              :style="{ backgroundColor: currentScheme.inline_highlight_bg }"
            ></div>
            <input
              type="text"
              :value="currentScheme.inline_highlight_bg"
              @input="(e: any) => updateColor('inline_highlight_bg', e.target.value)"
              class="color-text-input"
            />
          </div>
        </div>
        <div class="global-color-row">
          <label>Border Color:</label>
          <div class="color-input-group">
            <div
              class="color-swatch"
              :style="{ backgroundColor: currentScheme.border_color }"
            ></div>
            <input
              type="text"
              :value="currentScheme.border_color"
              @input="(e: any) => updateColor('border_color', e.target.value)"
              class="color-text-input"
            />
          </div>
        </div>
        <div class="global-color-row">
          <label>Background:</label>
          <div class="color-input-group">
            <div
              class="color-swatch"
              :style="{ backgroundColor: currentScheme.background }"
            ></div>
            <input
              type="text"
              :value="currentScheme.background"
              @input="(e: any) => updateColor('background', e.target.value)"
              class="color-text-input"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.color-scheme-editor {
  padding: 16px;
  background: var(--bg-color, #ffffff);
  border-radius: 8px;
  border: 1px solid var(--border-color, #e0e0e0);
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 2px solid var(--border-color, #e0e0e0);
}

.editor-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: background 0.2s;
}

.action-btn:hover {
  background: #0056b3;
}

.preset-section {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.preset-section label {
  font-weight: 600;
  white-space: nowrap;
}

.scheme-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  background: white;
  cursor: pointer;
  flex: 1;
  max-width: 300px;
}

.categories-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.color-category {
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  padding: 12px;
  background: var(--bg-alt, #f9f9f9);
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-weight: 600;
  font-size: 14px;
}

.category-icon {
  font-size: 18px;
}

.color-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.color-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.color-row label {
  min-width: 70px;
  font-weight: 600;
}

.color-input-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.color-swatch {
  width: 32px;
  height: 32px;
  border: 1px solid var(--border-color, #999);
  border-radius: 4px;
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 0.1s;
}

.color-swatch:hover {
  transform: scale(1.05);
}

.color-text-input {
  padding: 4px 8px;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  flex: 1;
  min-width: 80px;
}

.global-colors {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 2px solid var(--border-color, #e0e0e0);
}

.global-colors h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
}

.global-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 12px;
}

.global-color-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.global-color-row label {
  font-weight: 600;
  min-width: 120px;
}
</style>
