<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { AppNotification } from "../composables/useNotifications";

defineProps<{
  notifications: AppNotification[];
}>();

const emit = defineEmits<{
  dismiss: [id: number];
}>();

const { t } = useI18n();
</script>

<template>
  <TransitionGroup
    tag="div"
    name="toast"
    class="fixed top-[18px] right-[18px] z-20 grid w-[min(420px,calc(100vw-32px))] gap-2 pointer-events-none"
    aria-live="polite"
  >
    <div
      v-for="notification in notifications"
      :key="notification.id"
      :class="[
        'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border border-border-dim rounded-lg bg-surface-panel text-text-body text-[13px] font-extrabold px-3 py-2.5 pointer-events-auto shadow-[0_10px_24px_rgba(0,0,0,0.22)]',
        notification.tone === 'success' && 'border-success-border bg-success-bg text-success-text',
        notification.tone === 'error' && 'border-danger-border bg-danger-dark-bg text-danger-text',
        notification.tone === 'info' && 'border-info-border bg-info-bg text-info-text',
      ]"
    >
      <span class="min-w-0 overflow-wrap-anywhere">{{ notification.message }}</span>
      <button
        type="button"
        :aria-label="t('common.dismissNotification')"
        class="grid w-[26px] h-[26px] place-items-center border-0 rounded-full bg-white/8 text-inherit cursor-pointer font-inherit leading-none hover:bg-white/14"
        @click="emit('dismiss', notification.id)"
      >
        ×
      </button>
    </div>
  </TransitionGroup>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 180ms ease,
    transform 220ms cubic-bezier(0.2, 0.9, 0.2, 1);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(32px) scale(0.98);
}

.toast-move {
  transition: transform 180ms ease;
}
</style>
