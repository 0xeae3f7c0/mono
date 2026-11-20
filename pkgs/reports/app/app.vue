<script setup lang="ts">
// Lists archived weekly reports (PDFs uploaded by `archive`).
const weeks = ['2026-W47', '2026-W46', '2026-W45']
</script>

<template>
  <main>
    <h1>Weekly reports</h1>
    <ul>
      <li v-for="week in weeks" :key="week">
        <a :href="`/reports/${week}.pdf`">{{ week }}</a>
      </li>
    </ul>
  </main>
</template>
