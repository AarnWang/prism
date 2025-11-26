<template>
  <div v-if="show" class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h3>{{ title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="16" height="16" viewBox="0 0 16 16">
            <line x1="2" y1="2" x2="14" y2="14" stroke="currentColor" stroke-width="2"/>
            <line x1="14" y1="2" x2="2" y2="14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
      
      <div class="modal-body">
        <div class="search-bar">
          <input 
            type="text" 
            v-model="searchQuery" 
            placeholder="搜索模型..." 
            class="search-input"
            ref="searchInput"
          >
        </div>

        <div v-if="loading" class="loading-state">
          <div class="spinner"></div>
          <p>正在获取模型列表...</p>
        </div>

        <div v-else-if="error" class="error-state">
          <p class="error-text">{{ error }}</p>
          <button class="retry-btn" @click="$emit('retry')">重试</button>
        </div>

        <div v-else class="model-list">
          <div 
            v-for="model in filteredModels" 
            :key="model.id"
            class="model-item"
            @click="selectModel(model.id)"
          >
            <span class="model-name">{{ model.id }}</span>
            <span v-if="model.owned_by" class="model-owner">{{ model.owned_by }}</span>
          </div>
          <div v-if="filteredModels.length === 0" class="empty-state">
            未找到匹配的模型
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue'

const props = defineProps({
  show: Boolean,
  title: {
    type: String,
    default: '选择模型'
  },
  models: {
    type: Array,
    default: () => []
  },
  loading: Boolean,
  error: String
})

const emit = defineEmits(['close', 'select', 'retry'])

const searchQuery = ref('')
const searchInput = ref(null)

const filteredModels = computed(() => {
  if (!searchQuery.value) return props.models
  const query = searchQuery.value.toLowerCase()
  return props.models.filter(model => 
    model.id.toLowerCase().includes(query) || 
    (model.label && model.label.toLowerCase().includes(query))
  )
})

const selectModel = (id) => {
  emit('select', id)
  emit('close')
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    searchQuery.value = ''
    nextTick(() => {
      if (searchInput.value) {
        searchInput.value.focus()
      }
    })
  }
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100; /* Higher than SettingsModal */
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.close-btn {
  background: none;
  border: none;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  color: #6b7280;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #374151;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
}

.search-bar {
  margin-bottom: 12px;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  box-sizing: border-box;
}

.search-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.model-list {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
}

.model-item {
  padding: 10px 12px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: background-color 0.2s;
}

.model-item:last-child {
  border-bottom: none;
}

.model-item:hover {
  background-color: #f9fafb;
}

.model-name {
  font-size: 14px;
  color: #374151;
  font-weight: 500;
}

.model-owner {
  font-size: 12px;
  color: #9ca3af;
}

.loading-state, .error-state, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: #6b7280;
  font-size: 14px;
}

.error-text {
  color: #dc2626;
  margin-bottom: 12px;
  text-align: center;
}

.retry-btn {
  padding: 6px 12px;
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: #374151;
}

.retry-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid #e5e7eb;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
