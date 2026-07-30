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
      'inline-flex items-center justify-center gap-[7px] font-inherit font-extrabold cursor-pointer transition-[background-color,border-color,color,opacity] duration-[140ms] ease',
      'border border-border-control rounded-radius-md bg-surface-control text-text-body',
      'hover:not-disabled:bg-surface-control-hover',
      'disabled:cursor-not-allowed disabled:opacity-45',
      size === 'md' && 'min-h-[34px] px-3',
      size === 'sm' && 'min-h-[30px] px-2.5 text-xs',
      size === 'xs' && 'min-h-[26px] px-2 text-xs',
      variant === 'primary' && 'border-accent-border bg-accent-bg text-accent-text hover:not-disabled:bg-accent-bg-hover',
      variant === 'danger' && 'border-danger-border bg-danger-bg text-danger-text hover:not-disabled:bg-danger-bg-hover',
      block && 'w-full',
    ]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>