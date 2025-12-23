# iMAGE Codebase Documentation

## Project Structure

### .

- **dock-template.vue** (.vue) - 78 lines
- **dock-styles.css** (.css) - 52 lines
- **playwright-ct.config.ts** (.ts) - 46 lines
- **vitest.config.ts** (.ts) - 21 lines
- **dock-integration.js** (.js) - 57 lines
- **vite.config.ts** (.ts) - 66 lines

### landing

- **app.js** (.js) - 153 lines
- **styles.css** (.css) - 623 lines

### playwright

- **index.ts** (.ts) - 3 lines

### playwright/.cache/assets

- **AuthButton-BevI0nZC.js** (.js) - 1461 lines
- **_plugin-vue_export-helper-pcqpp-6-.js** (.js) - 10 lines
- **PhotoGallery-BjV_KUpv.js** (.js) - 243 lines
- **PhotoGallery-D_10NACt.css** (.css) - 13 lines
- **index-Y-4AjvAo.css** (.css) - 1073 lines
- **index-CkkE85_b.js** (.js) - 19943 lines

### src

- **mobile.css** (.css) - 1177 lines
- **style.css** (.css) - 1446 lines
- **App.vue** (.vue) - 2478 lines
- **vite-env.d.ts** (.ts) - 7 lines
- **main.ts** (.ts) - 5 lines

### src-tauri

- **build.rs** (.rs) - 6 lines

### src-tauri/src

- **main.rs** (.rs) - 6 lines
- **compress.rs** (.rs) - 629 lines
- **security_verify.rs** (.rs) - 88 lines
- **crypto.rs** (.rs) - 1102 lines
- **github.rs** (.rs) - 1858 lines
- **lib.rs** (.rs) - 116 lines
- **pipeline.rs** (.rs) - 714 lines

### src/components

- **PrettyText.vue** (.vue) - 129 lines
- **AsyncState.vue** (.vue) - 141 lines
- **UploadToast.vue** (.vue) - 459 lines
- **SecuritySettings.vue** (.vue) - 770 lines
- **PhotoUploader.vue** (.vue) - 194 lines
- **CircularGallery.vue** (.vue) - 972 lines
- **PhotoCard.vue** (.vue) - 254 lines
- **AccentPicker.vue** (.vue) - 44 lines
- **AuthButton.vue** (.vue) - 258 lines
- **PrivacySettings.vue** (.vue) - 504 lines
- **FloatingTagPanel.vue** (.vue) - 215 lines
- **BackupSettings.vue** (.vue) - 819 lines
- **AmbientBackground.vue** (.vue) - 127 lines
- **GlowButton.vue** (.vue) - 139 lines
- **SpaceLoader.vue** (.vue) - 155 lines
- **PhotoDetail.vue** (.vue) - 752 lines
- **LocalImageBrowser.vue** (.vue) - 573 lines
- **TopBar.vue** (.vue) - 330 lines
- **PhotoGallery.vue** (.vue) - 868 lines
- **RainbowButton.vue** (.vue) - 90 lines
- **GlassSurface.vue** (.vue) - 377 lines
- **AlbumTree.vue** (.vue) - 242 lines
- **SettingsPanel.vue** (.vue) - 941 lines
- **HoverCard.vue** (.vue) - 69 lines
- **Masonry.vue** (.vue) - 364 lines
- **RepoCreator.vue** (.vue) - 448 lines
- **SyncStatusIndicator.vue** (.vue) - 315 lines
- **MacOSDock.vue** (.vue) - 211 lines
- **MobileNav.vue** (.vue) - 217 lines
- **FolderUploadDialog.vue** (.vue) - 407 lines
- **TopHeader.vue** (.vue) - 268 lines
- **PipelineEditor.vue** (.vue) - 923 lines
- **SecureImage.vue** (.vue) - 136 lines
- **ContextColorMenu.vue** (.vue) - 158 lines
- **ErrorBoundary.vue** (.vue) - 172 lines
- **GlitchText.vue** (.vue) - 123 lines
- **ContextMenu.vue** (.vue) - 172 lines
- **ThemeSettings.vue** (.vue) - 373 lines
- **ColorTagPanel.vue** (.vue) - 306 lines
- **DataDriverManager.vue** (.vue) - 661 lines
- **PhotoPreview.vue** (.vue) - 241 lines
- **HaloSearch.vue** (.vue) - 218 lines

### src/components/PhotoGallery

- **PhotoGalleryLoading.vue** (.vue) - 52 lines
- **PhotoGalleryEmpty.vue** (.vue) - 89 lines
- **PhotoGalleryList.vue** (.vue) - 247 lines
- **PhotoGalleryGrid.vue** (.vue) - 94 lines
- **PhotoGalleryErrorBoundary.vue** (.vue) - 95 lines
- **PhotoLightbox.vue** (.vue) - 373 lines

### src/components/optimized

- **useDynamicMasonry.ts** (.ts) - 135 lines
- **useOptimizedSearch.ts** (.ts) - 187 lines
- **index.ts** (.ts) - 11 lines
- **VirtualMasonry.vue** (.vue) - 687 lines
- **LazyImage.vue** (.vue) - 168 lines
- **OptimizedGallery.vue** (.vue) - 98 lines

### src/components/ui

- **ConfirmDialog.vue** (.vue) - 135 lines
- **Modal.vue** (.vue) - 144 lines

### src/components/ui/NeuralBackground

- **NeuralBackground.vue** (.vue) - 311 lines

### src/components/ui/cursor

- **Cursor.vue** (.vue) - 183 lines

### src/components/ui/vortex

- **Vortex.vue** (.vue) - 253 lines
- **index.ts** (.ts) - 1 lines

### src/composables

- **useMobileSearch.ts** (.ts) - 128 lines
- **useSyncStatus.ts** (.ts) - 174 lines
- **useColorTags.ts** (.ts) - 93 lines
- **useTimeout.ts** (.ts) - 60 lines
- **useTheme.ts** (.ts) - 401 lines
- **useUploadToast.ts** (.ts) - 154 lines
- **useErrorBoundary.ts** (.ts) - 75 lines
- **useBackupSettings.ts** (.ts) - 192 lines
- **useCrypto.ts** (.ts) - 251 lines
- **usePlatform.ts** (.ts) - 43 lines
- **usePhotoUpload.ts** (.ts) - 290 lines
- **useCompression.ts** (.ts) - 173 lines
- **useNavigation.ts** (.ts) - 145 lines
- **usePipeline.ts** (.ts) - 372 lines
- **useGitHubAuth.ts** (.ts) - 235 lines
- **usePhotoActions.ts** (.ts) - 49 lines
- **useKeyboardShortcuts.ts** (.ts) - 67 lines
- **useImageMetadata.ts** (.ts) - 473 lines
- **useFavorites.ts** (.ts) - 88 lines
- **useAccentColor.ts** (.ts) - 49 lines
- **useRepoManager.ts** (.ts) - 200 lines
- **useSelection.ts** (.ts) - 106 lines
- **useDataDriver.ts** (.ts) - 195 lines
- **useTags.ts** (.ts) - 83 lines
- **usePhotoPreviewSize.ts** (.ts) - 58 lines
- **useDockApps.ts** (.ts) - 54 lines

### src/composables/__tests__

- **uiProperties.spec.ts** (.ts) - 208 lines
- **useTheme.spec.ts** (.ts) - 188 lines
- **usePhotoPreviewSize.spec.ts** (.ts) - 110 lines
- **repoValidation.spec.ts** (.ts) - 209 lines
- **useSelection.spec.ts** (.ts) - 182 lines
- **useRepoManager.spec.ts** (.ts) - 259 lines
- **useFavorites.spec.ts** (.ts) - 220 lines
- **folderUpload.spec.ts** (.ts) - 407 lines
- **uploadProgress.spec.ts** (.ts) - 231 lines

### src/config

- **index.ts** (.ts) - 3 lines
- **constants.ts** (.ts) - 207 lines
- **css-vars.ts** (.ts) - 79 lines

### src/lib

- **colorTags.ts** (.ts) - 43 lines
- **utils.ts** (.ts) - 6 lines

### src/types

- **photo.ts** (.ts) - 41 lines

### tests

- **PhotoGallery.spec.ts** (.ts) - 36 lines
- **AuthButton.spec.ts** (.ts) - 37 lines

### tests/components

- **Masonry.spec.ts** (.ts) - 133 lines

### tests/mocks

- **usePhotoUpload.ts** (.ts) - 36 lines
- **useGitHubAuth.ts** (.ts) - 39 lines
- **useAccentColor.ts** (.ts) - 25 lines

## Summary

- Total files processed: 135
- Comments removed and replaced with structured headers
- Documentation generated: CODE_DOCUMENTATION.md
