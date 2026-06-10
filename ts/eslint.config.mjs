import tseslint from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";

/** @type {import('eslint').Linter.FlatConfig[]} */
export default [
  {
    files: ["packages/**/*.ts"],
    languageOptions: {
      parser: tsParser,
      parserOptions: { project: true },
    },
    plugins: { "@typescript-eslint": tseslint },
    rules: {
      ...tseslint.configs.recommended.rules,
    },
  },
  // Policy-package sandbox: forbid dangerous globals in policy/catalog packages
  {
    files: ["packages/policy-*/**/*.ts", "packages/catalog-*/**/*.ts"],
    rules: {
      "no-restricted-globals": [
        "error",
        { name: "Date",         message: "Policy may not use wall-clock time." },
        { name: "document",     message: "Policy may not access the DOM." },
        { name: "window",       message: "Policy may not access window." },
        { name: "localStorage", message: "Policy may not access localStorage." },
        { name: "fetch",        message: "Policy may not make network calls." },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: "MemberExpression[object.name='Math'][property.name='random']",
          message: "Policy may not use Math.random; use deterministic RNG from script-sdk.",
        },
      ],
    },
  },
];
