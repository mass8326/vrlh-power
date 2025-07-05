import type { UserConfig } from "@unocss/core";
import fonts from "@unocss/preset-web-fonts";
import wind from "@unocss/preset-wind4";
import directives from "@unocss/transformer-directives";
import group from "@unocss/transformer-variant-group";

const config: UserConfig = {
  transformers: [directives(), group()],
  presets: [wind(), fonts({ fonts: { sans: ["Montserrat"] } })],
};

export default config;
