import path from "node:path";
import { fileURLToPath } from "node:url";
import pluginImport from "eslint-plugin-import";
import configPrettier from "eslint-plugin-prettier/recommended";
import pluginSvelte from "eslint-plugin-svelte";
import pluginUnicorn from "eslint-plugin-unicorn";
import { globifyGitIgnoreFile } from "globify-gitignore";
import tseslint from "typescript-eslint";

/** @type {import("eslint").Linter.Config[]} */
const config = [
  await getIgnoreConfig(),
  configPrettier,
  getImportConfig(),
  ...getStandardConfigs(),
  ...getSvelteConfigs(),
  ...getTsConfigs(),
];

export default config;

async function getIgnoreConfig() {
  const filename = fileURLToPath(import.meta.url);
  const dirname = path.dirname(filename);
  const globs = await globifyGitIgnoreFile(dirname);
  const ignores = globs.map(({ glob, included }) => {
    if (!glob.startsWith("**/")) glob = "**/" + glob;
    return (included ? "!" : "") + glob;
  });
  return {
    ignores: [...ignores, "**/vendor/**", "**/(payload)/admin/importMap.js"],
  };
}

function getImportConfig() {
  const groups = [
    "type",
    "builtin",
    "external",
    "internal",
    "parent",
    "sibling",
    "index",
    "object",
  ];
  return {
    plugins: { import: pluginImport },
    settings: {
      "import/parsers": { "@typescript-eslint/parser": [".ts", ".tsx"] },
    },
    rules: {
      "import/order": ["warn", { alphabetize: { order: "asc" }, groups }],
      "sort-imports": ["warn", { ignoreDeclarationSort: true }],
    },
  };
}

function getStandardConfigs() {
  return [
    ...tseslint.configs.recommended,
    {
      plugins: { unicorn: pluginUnicorn },
      rules: {
        // Handled by typescript
        "no-undef": "off",
        // Personal opinion
        "no-restricted-syntax": [
          "error",
          {
            selector:
              "CallExpression[callee.object.name='console'][callee.property.name='log']",
            message: "console.log() is for temporary development use only",
          },
        ],
        "unicorn/catch-error-name": "off",
        "unicorn/no-null": "off",
        "unicorn/prevent-abbreviations": "off",
        "unicorn/switch-case-braces": ["error", "avoid"],
      },
    },
    {
      files: ["**/*.{cjs,cts}"],
      rules: { "@typescript-eslint/no-var-requires": "off" },
    },
  ];
}

function getTsConfigs() {
  const files = ["**/*.svelte", "**/*.ts", "**/*.cts", "**/*.mts", "**/*.tsx"];
  return [
    ...[
      ...tseslint.configs.strictTypeChecked,
      {
        rules: {
          "@typescript-eslint/ban-ts-comment": "off",
          "@typescript-eslint/consistent-type-imports": "error",
          "@typescript-eslint/explicit-member-accessibility": "error",
          "@typescript-eslint/explicit-module-boundary-types": "error",
          "@typescript-eslint/no-confusing-void-expression": [
            "off",
            { ignoreVoidOperator: true },
          ],
          "@typescript-eslint/no-explicit-any": "off",
          "@typescript-eslint/no-non-null-assertion": "off",
          "@typescript-eslint/no-unsafe-argument": "off",
          "@typescript-eslint/no-unsafe-assignment": "off",
          "@typescript-eslint/no-unsafe-call": "off",
          "@typescript-eslint/no-unsafe-member-access": "off",
          "@typescript-eslint/no-unused-vars": [
            "error",
            { argsIgnorePattern: "^_" },
          ],
          "@typescript-eslint/restrict-template-expressions": [
            "error",
            { allowNullish: true, allowNumber: true },
          ],
          "@typescript-eslint/unbound-method": "warn",
          "import/no-duplicates": "error",
          // Might silently delete important code if types are inferred incorrectly
          "@typescript-eslint/no-unnecessary-condition": "off",
          // Too many false positives with complex types
          "@typescript-eslint/no-unsafe-return": "off",
          // Conflicts with options for @typescript-eslint/no-confusing-void-expression
          "@typescript-eslint/no-meaningless-void-operator": "off",
          // Conflicts with fix for @typescript-eslint/unbound-method
          "@typescript-eslint/no-invalid-void-type": [
            "error",
            { allowAsThisParameter: true },
          ],
        },
      },
    ].map((val) => ({
      ...val,
      files,
      languageOptions: {
        parserOptions: {
          parser: tseslint.parser,
          projectService: true,
          extraFileExtensions: [".svelte"],
        },
      },
    })),
    {
      files: ["*.d.ts"],
      rules: { "@typescript-eslint/consistent-type-imports": "off" },
    },
  ];
}

function getSvelteConfigs() {
  return [
    ...pluginSvelte.configs["flat/recommended"],
    ...pluginSvelte.configs["flat/prettier"],
    {
      files: ["**/*.svelte"],
      rules: {
        "@typescript-eslint/no-confusing-void-expression": "off",
        // Necessary evil
        "svelte/no-at-html-tags": "warn",
        // Type detection struggles
        "@typescript-eslint/no-unsafe-return": "off",
        // Prop destructuring
        "prefer-const": ["error", { destructuring: "all" }],
      },
    },
  ];
}
