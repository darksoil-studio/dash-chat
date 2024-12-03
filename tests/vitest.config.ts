import { defineConfig } from "vitest/config";
// @ts-ignore
import pkg from "./package.json";

export default defineConfig({
  test: {
    poolOptions: {
      threads: {
        singleThread: true,
      },
    },
    testTimeout: 60 * 1000 * 3, // 3  mins
    deps: {
      optimizer: {
        ssr: {
          enabled: true,
          // @ts-ignore
          include: Object.keys(pkg.dependencies),
          exclude: ["@holochain/client"],
        },
      },
    },
  },
});
