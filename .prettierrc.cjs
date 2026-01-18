// prettier.config.js, .prettierrc.js, prettier.config.mjs, or .prettierrc.mjs

/**
 * @see https://prettier.io/docs/en/configuration.html
 * @type {import("prettier").Config}
 */
const config = {
  arrowParens: 'always',
  bracketSpacing: true,
  endOfLine: 'lf',
  htmlWhitespaceSensitivity: 'strict',
  insertPragma: false,
  singleAttributePerLine: false,
  bracketSameLine: true,
  jsxBracketSameLine: true,
  jsxSingleQuote: true,
  printWidth: 160,
  proseWrap: 'preserve',
  quoteProps: 'as-needed',
  requirePragma: false,
  semi: true,
  singleQuote: true,
  tabWidth: 2,
  trailingComma: 'all',
  useTabs: false,
  embeddedLanguageFormatting: 'auto',
  vueIndentScriptAndStyle: false,
  experimentalTernaries: false,
  overrides: [
    {
      files: '*.test.js',
      options: {
        semi: false,
      },
    },
    {
      files: ['*.html', 'legacy/**/*.js'],
      options: {
        semi: false,
        tabWidth: 2,
      },
    },
  ],
};

module.exports = config;
