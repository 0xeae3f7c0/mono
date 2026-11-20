<script setup lang="ts">
import ActivityWidget from '../components/ActivityWidget.vue'
</script>

<template>
  <main>
    <h1>Embeddable activity widget</h1>
    <p>Preview of the snippet sites embed (built by Vite, minified by <code>minify</code>).</p>
    <ActivityWidget repo="commit/commit" :commits="42" ci="passing" />
  </main>
</template>
