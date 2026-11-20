<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ repo: string; commits: number; ci: 'passing' | 'failing' | 'running' }>()

const label = computed(() => `${props.commits} commit${props.commits === 1 ? '' : 's'} this week`)
</script>

<template>
  <div class="activity-widget" :data-ci="ci">
    <strong>{{ repo }}</strong>
    <span>{{ label }}</span>
    <span class="ci">CI {{ ci }}</span>
  </div>
</template>

<style scoped>
.activity-widget {
  display: inline-flex;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-family: system-ui, sans-serif;
}
[data-ci='passing'] .ci { color: #2f855a; }
[data-ci='failing'] .ci { color: #c53030; }
</style>
