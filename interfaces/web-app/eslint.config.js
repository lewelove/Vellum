import sveltePlugin from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import tsEslint from "typescript-eslint";

export default [
  ...sveltePlugin.configs["flat/recommended"],
  ...sveltePlugin.configs["flat/prettier"],
  {
    files: ["**/*.ts"],
    languageOptions: {
      parser: tsEslint.parser
    }
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsEslint.parser
      }
    },
    rules: {
      "svelte/button-has-type": "error",
      "svelte/no-unused-svelte-ignore": "error",
      "svelte/valid-compile": "error",
      "svelte/no-at-html-tags": "warn",
      "svelte/infinite-reactive-loop": "error",
      "svelte/no-dupe-use-directives": "error",
      "svelte/sort-attributes": [
        "error",
        {
          order: [
            "this",
            "bind:this",
            "slot",
            "definition",
            "PROPERTIES",
            "BINDINGS",
            "FUNCTIONS",
            "ACTIONS",
            "TRANSITIONS",
            "EVENTS"
          ]
        }
      ]
    }
  }
];
