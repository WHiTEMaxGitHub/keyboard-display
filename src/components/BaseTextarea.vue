<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    disabled?: boolean;
    block?: boolean;
    rows?: number;
  }>(),
  {
    placeholder: "",
    disabled: false,
    block: false,
    rows: 3,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <textarea
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :rows="rows"
    :class="[
      'min-h-[34px] border border-border-control rounded-radius-md bg-surface-control text-text-body font-inherit text-sm font-bold leading-[1.5] px-2.5 py-2 resize-y',
      'focus:border-accent-focus-border focus:outline-2 focus:outline-accent-focus-ring focus:outline-offset-0',
      'disabled:cursor-not-allowed disabled:opacity-55',
      block ? 'w-full' : '',
    ]"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>