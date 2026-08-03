export default {
  ignoreFiles: ["dist/**", "node_modules/**", "**/dist/**"],
  extends: ["stylelint-config-standard", "stylelint-config-html"],
  rules: {
    "custom-property-empty-line-before": null,
    "rule-empty-line-before": null,
    "comment-empty-line-before": null,
    "lightness-notation": null,
    "hue-degree-notation": null,
    "alpha-value-notation": null,
    "color-function-alias-fix": null,
    "declaration-property-value-no-unknown": null
  }
};
