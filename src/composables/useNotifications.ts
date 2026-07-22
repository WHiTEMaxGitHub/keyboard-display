import { ref } from "vue";

export type NotificationTone = "success" | "error" | "info";

export type AppNotification = {
  id: number;
  tone: NotificationTone;
  message: string;
};

let nextNotificationId = 1;

export function useNotifications() {
  const notifications = ref<AppNotification[]>([]);

  function notify(tone: NotificationTone, message: string) {
    const id = nextNotificationId++;
    notifications.value = [
      ...notifications.value,
      {
        id,
        tone,
        message,
      },
    ];

    window.setTimeout(() => {
      dismissNotification(id);
    }, 4200);
  }

  function dismissNotification(id: number) {
    notifications.value = notifications.value.filter((notification) => notification.id !== id);
  }

  return {
    notifications,
    notify,
    dismissNotification,
  };
}
