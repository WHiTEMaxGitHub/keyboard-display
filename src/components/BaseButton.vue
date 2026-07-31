<script setup lang="ts">
withDefaults(
  defineProps<{
    type?: "button" | "submit";
    variant?: "primary" | "secondary" | "danger";
    size?: "xs" | "sm" | "md";
    block?: boolean;
    disabled?: boolean;
  }>(),
  {
    type: "button",
    variant: "secondary",
    size: "md",
    block: false,
    disabled: false,
  },
);
</script>

<template>
  <button
    :type="type"
    :class="[
      'inline-flex items-center justify-center gap-[7px] font-inherit font-extrabold cursor-pointer btn-press rounded-md border border-border-control bg-surface-control text-text-body',
      'hover:not-disabled:bg-surface-control-hover',
      'disabled:cursor-not-allowed disabled:opacity-45',
      size === 'md' && 'min-h-[34px] px-3',
      size === 'sm' && 'min-h-[30px] px-2.5 text-xs',
      size === 'xs' && 'min-h-[26px] px-2 text-xs',
      variant === 'primary' && 'border-transparent bg-gradient-to-br from-accent to-accent-blue text-[#04111c] shadow-[0_8px_28px_var(--color-accent-glow)] hover:not-disabled:shadow-[0_12px_36px_var(--color-accent-glow)]',
      variant === 'danger' && 'border-danger-border bg-danger-bg text-danger-text hover:not-disabled:bg-danger-bg-hover',
      block && 'w-full',
    ]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>