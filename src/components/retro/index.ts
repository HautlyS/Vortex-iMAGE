// 8-Bit Pixel Art UI Components
// A complete retro gaming-inspired component library

// === CORE COMPONENTS ===
export { default as PixelCard } from './PixelCard.vue'
export { default as PixelButton } from './PixelButton.vue'
export { default as VoxelBackground } from './VoxelBackground.vue'

// === FORM COMPONENTS ===
export { default as PixelInput } from './PixelInput.vue'
export { default as PixelSlider } from './PixelSlider.vue'
export { default as PixelToggle } from './PixelToggle.vue'
export { default as PixelSearch } from './PixelSearch.vue'
export { default as PixelDropdown } from './PixelDropdown.vue'

// === FEEDBACK COMPONENTS ===
export { default as PixelProgress } from './PixelProgress.vue'
export { default as PixelLoader } from './PixelLoader.vue'
export { default as PixelBadge } from './PixelBadge.vue'
export { default as PixelTooltip } from './PixelTooltip.vue'
export { default as PixelModal } from './PixelModal.vue'

// === NAVIGATION COMPONENTS ===
export { default as PixelStepper } from './PixelStepper.vue'

// === EFFECT COMPONENTS ===
export { default as PixelBorder } from './PixelBorder.vue'
export { default as PixelSpark } from './PixelSpark.vue'
export { default as PixelText } from './PixelText.vue'
export { default as PixelMagnet } from './PixelMagnet.vue'
export { default as PixelNoise } from './PixelNoise.vue'
export { default as PixelLightning } from './PixelLightning.vue'

// === DECORATIVE COMPONENTS ===
export { default as PixelMascot } from './PixelMascot.vue'
export { default as PixelAvatar } from './PixelAvatar.vue'

// === TYPE EXPORTS ===
export type PixelButtonVariant = 'default' | 'primary' | 'success' | 'danger' | 'ghost' | 'retro'
export type PixelButtonSize = 'sm' | 'md' | 'lg'
export type PixelCardVariant = 'default' | 'success' | 'warning' | 'danger' | 'info'
export type PixelBadgeVariant = 'default' | 'success' | 'warning' | 'danger' | 'info' | 'legendary' | 'epic' | 'rare'
export type PixelLoaderVariant = 'spinner' | 'dots' | 'bars' | 'pacman' | 'blocks' | 'hearts'
export type PixelProgressVariant = 'default' | 'health' | 'mana' | 'exp' | 'rainbow'
export type PixelInputVariant = 'default' | 'terminal' | 'search'
export type PixelToggleVariant = 'default' | 'power' | 'switch'
export type PixelSliderVariant = 'default' | 'volume' | 'health' | 'mana'
export type PixelMascotMood = 'happy' | 'excited' | 'thinking' | 'sleeping'
