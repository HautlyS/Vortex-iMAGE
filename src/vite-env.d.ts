/**
 * TypeScript Module - 0 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}