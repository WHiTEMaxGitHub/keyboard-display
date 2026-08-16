<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { normalizeHexColor } from "../domain/colorPicker";
import type { AppConfig, OverlayStyle } from "../domain/defaultConfig";
import BaseButton from "./BaseButton.vue";
import BaseControlGrid from "./BaseControlGrid.vue";
import BaseFormRow from "./BaseFormRow.vue";
import BasePanel from "./BasePanel.vue";
import BasePanelHeader from "./BasePanelHeader.vue";
import BaseRange from "./BaseRange.vue";
import BaseSelect from "./BaseSelect.vue";
import BaseToggleRow from "./BaseToggleRow.vue";
import ColorPicker from "./ColorPicker.vue";

type StyleColorField =
  | "idleColor"
  | "activeColor"
  | "idleTextColor"
  | "activeTextColor"
  | "backgroundColor";

const COLOR_FIELDS: Array<{
  field: StyleColorField;
  labelKey: string;
}> = [
  { field: "idleColor", labelKey: "appearance.colors.idleKey" },
  { field: "activeColor", labelKey: "appearance.colors.pressedKey" },
  { field: "idleTextColor", labelKey: "appearance.colors.idleText" },
  { field: "activeTextColor", labelKey: "appearance.colors.pressedText" },
  { field: "backgroundColor", labelKey: "appearance.colors.backplate" },
];

const props = defineProps<{
  config: AppConfig;
  recentColors: string[];
}>();

const emit = defineEmits<{
  "preview-overlay-style": [style: OverlayStyle];
  "update-overlay-style": [style: OverlayStyle];
  "remember-color": [color: string];
  "refresh-pov": [];
}>();

const { t } = useI18n();
const rememberedBackplateAlpha = ref(alphaFromHex(props.config.style.backgroundColor) || 255);
const idleKeyVisibilityOptions = [
  { value: "visible", labelKey: "appearance.idleVisibility.visible" },
  { value: "hidden", labelKey: "appearance.idleVisibility.hidden" },
];

watch(
  () => props.config.style.backgroundColor,
  (color) => {
    const alpha = alphaFromHex(color);
    if (alpha > 0) {
      rememberedBackplateAlpha.value = alpha;
    }
  },
);

function updateScale(scale: number) {
  emit("update-overlay-style", { ...props.config.style, scale });
}

function formatScale(scale: number) {
  return `${scale.toFixed(2)}x`;
}

function effectiveUnitPx() {
  return Math.round(props.config.layout.unitPx * props.config.style.scale);
}

function updateOpacity(opacity: number) {
  emit("update-overlay-style", { ...props.config.style, opacity });
}

function updateBackgroundRadius(backgroundRadius: number) {
  emit("update-overlay-style", {
    ...props.config.style,
    backgroundRadius,
  });
}

function updateIdleKeyVisibility(value: string) {
  const idleKeyVisibility = value as OverlayStyle["idleKeyVisibility"];
  emit("update-overlay-style", { ...props.config.style, idleKeyVisibility });
}

function updateBackplateTransparent(event: Event) {
  const transparent = (event.target as HTMLInputElement).checked;
  const currentAlpha = alphaFromHex(props.config.style.backgroundColor);
  if (transparent && currentAlpha > 0) {
    rememberedBackplateAlpha.value = currentAlpha;
  }

  emit("update-overlay-style", {
    ...props.config.style,
    backgroundColor: setHexAlpha(
      props.config.style.backgroundColor,
      transparent ? 0 : rememberedBackplateAlpha.value,
    ),
  });
}

function updateStyleColor(
  field: StyleColorField,
  color: string,
) {
  const nextColor = normalizeHexColor(color, props.config.style[field]);
  emit("update-overlay-style", {
    ...props.config.style,
    [field]: nextColor,
  });
}

function previewStyleColor(
  field: StyleColorField,
  color: string,
) {
  const nextColor = normalizeHexColor(color, props.config.style[field]);
  emit("preview-overlay-style", {
    ...props.config.style,
    [field]: nextColor,
  });
}

function isBackplateTransparent() {
  const normalizedColor = normalizeHexColor(props.config.style.backgroundColor);
  return normalizedColor.length === 9 && normalizedColor.endsWith("00");
}

function setHexAlpha(color: string, alpha: number) {
  const normalizedColor = normalizeHexColor(color);
  const rgb = normalizedColor.slice(0, 7);
  return `${rgb}${Math.min(255, Math.max(0, Math.round(alpha)))
    .toString(16)
    .padStart(2, "0")}`;
}

function alphaFromHex(color: string) {
  const normalizedColor = normalizeHexColor(color);
  return normalizedColor.length === 9 ? Number.parseInt(normalizedColor.slice(7, 9), 16) : 255;
}
</script>

<template>
  <BasePanel wide>
    <BasePanelHeader :title="t('appearance.title')">
      <template #actions>
        <BaseButton @click="emit('refresh-pov')">
          {{ t("appearance.refreshPov") }}
        </BaseButton>
      </template>
    </BasePanelHeader>

    <BaseRange
      :model-value="config.style.scale"
      :label="t('appearance.scale')"
      :value-label="`${formatScale(config.style.scale)} · ${t('appearance.unitPx', { px: effectiveUnitPx() })}`"
      :min="0.5"
      :max="2.5"
      :step="0.01"
      @update:model-value="updateScale"
    />

    <BaseRange
      :model-value="config.style.opacity"
      :label="t('appearance.overlayTransparency')"
      :description="t('appearance.overlayTransparencyDescription')"
      :min="0.35"
      :max="1"
      :step="0.01"
      @update:model-value="updateOpacity"
    />

    <BaseControlGrid class="mb-4" min-column-width="260px">
      <BaseRange
        compact
        :model-value="config.style.backgroundRadius"
        :label="t('appearance.backplateRadius')"
        :min="0"
        :max="24"
        :step="1"
        @update:model-value="updateBackgroundRadius"
      />
    </BaseControlGrid>

    <BaseFormRow :label="t('appearance.idleKeys')">
      <BaseSelect
        class="w-full"
        compact
        :model-value="config.style.idleKeyVisibility"
        :options="idleKeyVisibilityOptions.map((option) => ({ value: option.value, label: t(option.labelKey) }))"
        @update:model-value="updateIdleKeyVisibility"
      />
    </BaseFormRow>

    <BaseControlGrid :aria-label="t('appearance.colors.group')">
      <ColorPicker
        v-for="colorField in COLOR_FIELDS"
        :key="colorField.field"
        :label="t(colorField.labelKey)"
        :value="config.style[colorField.field]"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor(colorField.field, $event)"
        @update:value="updateStyleColor(colorField.field, $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <div class="flex min-h-[38px] items-center">
        <BaseToggleRow
          compact
          :checked="isBackplateTransparent()"
          @change="updateBackplateTransparent"
        >
          {{ t("appearance.colors.transparentBackplate") }}
        </BaseToggleRow>
      </div>
    </BaseControlGrid>
  </BasePanel>
</template>
