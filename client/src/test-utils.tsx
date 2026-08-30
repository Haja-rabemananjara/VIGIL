import type { ReactNode } from "react";
import { render, type RenderOptions } from "@testing-library/react";
import { AuthProvider } from "@/stores/auth";

function TestWrapper({ children }: { children: ReactNode }) {
  return <AuthProvider>{children}</AuthProvider>;
}

export function renderWithAuth(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, "wrapper">,
) {
  return render(ui, { wrapper: TestWrapper, ...options });
}
