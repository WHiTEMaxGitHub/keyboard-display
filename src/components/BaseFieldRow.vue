<script setup lang="ts">
defineProps<{
  label: string;
}>();
</script>

<template>
  <div class="field-row">
    <span>{{ label }}</span>
    <strong>
      <slot />
    </strong>
  </div>
</template>

<style scoped>
.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.field-row span {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}
</style>
