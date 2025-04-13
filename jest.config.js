module.exports = {
  transform: {
    '^.+\\.svelte$': ['svelte-jester', { preprocess: true }],
    '^.+\\.js$': 'babel-jest',
    '^.+\\.ts$': 'ts-jest',
  },
  moduleFileExtensions: ['js', 'ts', 'svelte'],
  testPathIgnorePatterns: ['node_modules', 'dist', 'build'],
  setupFilesAfterEnv: ['@testing-library/jest-dom/extend-expect'],
};
