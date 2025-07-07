/** @type {import("prettier").Config} */
export default {
  plugins: ["prettier-plugin-svelte"],
  braceStyle: "1tbs",
  trailingComma: "all",
  overrides: [{ files: "*.json", options: { trailingComma: "none" } }],
};
