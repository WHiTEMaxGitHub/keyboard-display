<script setup lang="ts">
import type { AppNotification } from "../composables/useNotifications";

defineProps<{
  notifications: AppNotification[];
}>();

const emit = defineEmits<{
  dismiss: [id: number];
}>();
</script>

<template>
  <TransitionGroup
    tag="div"
    name="toast"
    class="notification-stack"
    aria-live="polite"
  >
    <div
      v-for="notification in notifications"
      :key="notification.id"
      :class="['notification-item', notification.tone]"
    >
      <span>{{ notification.message }}</span>
      <button type="button" aria-label="Dismiss notification" @click="emit('dismiss', notification.id)">
        ×
      </button>
    </div>
  </TransitionGroup>
</template>

<style scoped>
.notification-stack {
  position: fixed;
  top: 18px;
  right: 18px;
  z-index: 20;
  display: grid;
  width: min(420px, calc(100vw - 32px));
  gap: 8px;
  pointer-events: none;
}

.notification-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  background: #171b22;
  color: #dfe5ec;
  font-size: 13px;
  font-weight: 800;
  padding: 10px 12px;
  pointer-events: auto;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.22);
}

.notification-item.success {
  border-color: rgba(159, 240, 185, 0.28);
  background: #122019;
  color: #dfffe9;
}

.notification-item.error {
  border-color: rgba(255, 143, 143, 0.28);
  background: #241516;
  color: #ffd9d9;
}

.notification-item.info {
  border-color: rgba(155, 190, 255, 0.28);
  background: #141b27;
  color: #dbe7ff;
}

.notification-item span {
  min-width: 0;
  overflow-wrap: anywhere;
}

.notification-item button {
  display: grid;
  width: 26px;
  height: 26px;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  color: inherit;
  cursor: pointer;
  font: inherit;
  line-height: 1;
}

.notification-item button:hover {
  background: rgba(255, 255, 255, 0.14);
}

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
