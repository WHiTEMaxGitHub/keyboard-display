<script setup lang="ts">
withDefaults(
  defineProps<{
    type?: "text" | "number";
    modelValue: string | number;
    placeholder?: string;
    disabled?: boolean;
    block?: boolean;
  }>(),
  {
    type: "text",
    placeholder: "",
    disabled: false,
    block: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();
</script>

<template>
  <input
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :class="[
      'min-h-[34px] border border-border-dim rounded-lg bg-surface-control text-text-body font-inherit text-sm font-bold leading-none px-2.5 py-0',
      'focus:border-accent-focus-border focus:outline-2 focus:outline-accent-focus-ring focus:outline-offset-0',
      'disabled:cursor-not-allowed disabled:opacity-55',
      block ? 'w-full' : '',
    ]"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>