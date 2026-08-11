import defaultProfileJson from "../../docs/default-config.json";
import example68KeyboardProfileJson from "../../docs/example-68-keyboard-config.json";
import exampleLeftKeyboardProfileJson from "../../docs/example-left-keyboard-config.json";
import { parseConfigFile, type OverlayConfigFile } from "./configFile";

export type BuiltInProfileId = "default" | "left-keyboard" | "68-keyboard";

export const DEFAULT_BUILT_IN_PROFILE_ID: BuiltInProfileId = "default";

export type BuiltInProfileTemplate = {
  id: BuiltInProfileId;
  label: string;
  createProfile: () => OverlayConfigFile;
  exportJson: () => string;
};

const PROFILE_JSON_BY_ID: Record<BuiltInProfileId, unknown> = {
  default: defaultProfileJson,
  "left-keyboard": exampleLeftKeyboardProfileJson,
  "68-keyboard": example68KeyboardProfileJson,
};

export const BUILT_IN_PROFILE_TEMPLATES: Record<BuiltInProfileId, BuiltInProfileTemplate> = {
  default: {
    id: "default",
    label: "CS POV",
    createProfile: createDefaultProfileConfig,
    exportJson: exportDefaultProfileJson,
  },
  "left-keyboard": {
    id: "left-keyboard",
    label: "Left keyboard",
    createProfile: createLeftKeyboardProfileConfig,
    exportJson: exportLeftKeyboardProfileJson,
  },
  "68-keyboard": {
    id: "68-keyboard",
    label: "68 Keyboard",
    createProfile: create68KeyboardProfileConfig,
    exportJson: export68KeyboardProfileJson,
  },
};

export function createDefaultProfileConfig() {
  return createProfileConfig("default");
}

export function createLeftKeyboardProfileConfig() {
  return createProfileConfig("left-keyboard");
}

export function create68KeyboardProfileConfig() {
  return createProfileConfig("68-keyboard");
}

export function exportDefaultProfileJson() {
  return exportProfileJson("default");
}

export function exportLeftKeyboardProfileJson() {
  return exportProfileJson("left-keyboard");
}

export function export68KeyboardProfileJson() {
  return exportProfileJson("68-keyboard");
}

export function createProfileConfig(id: BuiltInProfileId) {
  return parseConfigFile(exportProfileJson(id));
}

export function exportProfileJson(id: BuiltInProfileId) {
  return `${JSON.stringify(PROFILE_JSON_BY_ID[id], null, 2)}\n`;
}
