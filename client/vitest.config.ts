import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      reportsDirectory: "../docs/client-coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "**/node_modules/**",
        "**/.next/**",
        "**/*.config.*",
        "**/src-tauri/**",
        "src/app/**/layout.tsx",
        "src/components/ui/**",
        "**/*.d.ts",
        "src/app/page.tsx",
        "src/**/*.test.{ts,tsx}",
        "src/stores/socket.tsx",
        "src/lib/useNotifications.ts",
        "src/app/teams/[teamId]/releases/client.tsx",
        "src/app/teams/[teamId]/releases/[releaseId]/client.tsx",
        "src/app/onboarding/page.tsx",
        "src/app/messages/[userId]/page.tsx",
        "src/app/settings/services/page.tsx",
        "src/app/teams/[teamId]/page.tsx",
        "src/app/teams/[teamId]/incidents/page.tsx",
        "src/app/teams/[teamId]/incidents/[incidentId]/page.tsx",
        "src/app/teams/[teamId]/members/page.tsx",
        "src/app/teams/[teamId]/releases/page.tsx",
        "src/app/teams/[teamId]/releases/[releaseId]/page.tsx",
        "src/app/teams/[teamId]/rules/page.tsx",
      ],
    },
  },
});
