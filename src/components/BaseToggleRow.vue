<script setup lang="ts">
defineProps<{
  checked: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  change: [event: Event];
}>();
</script>

<template>
  <label :class="['flex items-center gap-2.5 text-text-secondary font-bold', compact ? 'm-0' : 'mb-4']">
    <input
      :checked="checked"
      type="checkbox"
      class="w-[18px] h-[18px]"
      style="accent-color: var(--color-accent)"
      @change="emit('change', $event)"
    />
    <slot />
  </label>
</template>