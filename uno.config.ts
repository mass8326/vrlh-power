import type { UserConfig } from "@unocss/core";
import wind, { type Theme } from "@unocss/preset-wind4";
import directives from "@unocss/transformer-directives";
import group from "@unocss/transformer-variant-group";

const config: UserConfig<Theme> = {
  theme: {
    font: {
      sans: "JetBrains Mono",
      mono: "JetBrains Mono",
    },
  },
  transformers: [directives(), group()],
  presets: [wind()],
};

export default config;
