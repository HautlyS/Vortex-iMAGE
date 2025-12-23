<!-- Replace your existing template structure with this: -->
<template>
  <ErrorBoundary>
    <SpaceLoader v-if="loading" @complete="loading = false" />
    
    <div v-else class="app-dock" @dragover.prevent="isDragging = true" @dragleave.prevent="isDragging = false" @drop.prevent="onDrop">
      <!-- Neural Background -->
      <NeuralBg v-if="token" class="app-neural-bg" :hue="neuralHue" :saturation="0.7" :chroma="0.5" />
      
      <!-- Drag Overlay (keep existing) -->
      <Transition name="fade">
        <div v-if="isDragging && token && repo" class="drag-overlay">
          <!-- Your existing drag overlay content -->
        </div>
      </Transition>

      <!-- Main Content (no sidebar) -->
      <main class="main-dock">
        <!-- Simplified Header -->
        <header class="header-dock">
          <div class="header-left">
            <h1 class="view-title">{{ viewTitle }}</h1>
            <span v-if="filteredPhotos.length > 0" class="photo-count">{{ filteredPhotos.length }} fotos</span>
          </div>

          <!-- Keep search bar -->
          <HaloSearch v-model="searchQuery" placeholder="Pesquisar fotos..." class="flex-1 max-w-[520px]" />

          <!-- Minimal header actions -->
          <div class="header-actions">
            <div class="view-toggle">
              <button :class="{ active: viewMode === 'grid' }" @click="viewMode = 'grid'" title="Grade">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
              </button>
              <button :class="{ active: viewMode === 'list' }" @click="viewMode = 'list'" title="Lista">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
              </button>
            </div>
          </div>
        </header>

        <!-- Content -->
        <div class="content-dock">
          <!-- Your existing content sections -->
          <template v-if="!token">
            <!-- Login state -->
          </template>
          <template v-else-if="!repo">
            <!-- Repo setup state -->
          </template>
          <template v-else>
            <AsyncState :loading="loadingPhotos" :error="photoLoadError" :empty="false" @retry="retryLoadPhotos">
              <PhotoGallery 
                :photos="filteredPhotos" 
                :loading="false" 
                :albums="albums"
                :show-albums="currentView === 'photos' && !selectedAlbumPath"
                @photo-click="handlePhotoClick"
                @album-click="handleAlbumSelect"
              />
            </AsyncState>
          </template>
        </div>
      </main>

      <!-- macOS Dock -->
      <MacOSDock 
        :apps="dockAppsWithBadges" 
        :open-apps="activeApps"
        @app-click="handleDockAppClick" 
      />

      <!-- Keep all your existing modals -->
      <PrivacySettings v-if="showPrivacy" @close="showPrivacy = false" />
      <!-- ... other modals ... -->
    </div>
  </ErrorBoundary>
</template>
