# Requirements Document

## Introduction

This document specifies the requirements for a comprehensive enhancement of the iMAGE photo management application. The enhancement transforms the current basic photo upload/view app into a production-ready, open-source photo management system with advanced folder/album handling, GitHub repository management, responsive UI with resizable previews, favorites system, color tagging, and a modern Matrix/Apple-inspired dark theme.

## Glossary

- **Photo_Manager**: The main application system for managing photos
- **Album**: A logical grouping of photos, represented as a folder in the GitHub repository
- **Sub_Album**: A nested album within a parent album, represented as a subfolder
- **Color_Tag**: A user-defined color label that can be applied to photos or albums for organization
- **Favorites_System**: The subsystem managing user-marked favorite photos and albums
- **Repository_Manager**: The subsystem handling GitHub repository creation and configuration
- **Privacy_Sync**: The subsystem ensuring local privacy settings match GitHub repository visibility
- **Photo_Preview**: The UI component displaying photo thumbnails with interactive controls
- **Resize_Handle**: A draggable UI element for adjusting photo preview size
- **Context_Menu**: A right-click menu providing actions for selected items
- **Theme_Engine**: The subsystem managing application visual customization
- **Upload_Toast**: A persistent notification component showing file transfer progress
- **Sync_Status_Indicator**: A visual element showing the synchronization state of a photo between local storage and GitHub

## Requirements

### Requirement 1: Folder Upload with Album Options

**User Story:** As a user, I want to upload folders and choose whether they become albums or just import all images recursively, so that I can organize my photos according to my preference.

#### Acceptance Criteria

1. WHEN a user uploads a folder, THE Photo_Manager SHALL display a dialog asking whether to treat the folder as an album or import images recursively
2. WHEN the user selects "Create Album" option, THE Photo_Manager SHALL create a corresponding folder structure in the GitHub repository
3. WHEN the user selects "Import Recursively" option, THE Photo_Manager SHALL extract all images from nested folders and upload them to the root photos folder
4. WHEN a folder contains subfolders and "Create Album" is selected, THE Photo_Manager SHALL create sub-albums matching the subfolder structure
5. IF the folder upload fails partially, THEN THE Photo_Manager SHALL report which files succeeded and which failed with error details

### Requirement 2: GitHub Repository Creation

**User Story:** As a user, I want to create a new GitHub repository directly from the app, so that I can start using the app without leaving it.

#### Acceptance Criteria

1. WHEN a user clicks "Create New Repository", THE Repository_Manager SHALL display a repository creation form
2. THE Repository_Manager SHALL allow configuration of repository name, description, and visibility (public/private)
3. WHEN the user submits the form, THE Repository_Manager SHALL create the repository via GitHub API
4. IF repository creation succeeds, THEN THE Repository_Manager SHALL automatically configure the app to use the new repository
5. IF repository creation fails, THEN THE Repository_Manager SHALL display a descriptive error message
6. THE Repository_Manager SHALL validate repository name format before submission

### Requirement 3: Privacy Settings Synchronization

**User Story:** As a user, I want my app privacy settings to actually sync with GitHub repository visibility, so that my photos have the privacy level I expect.

#### Acceptance Criteria

1. WHEN the user changes privacy level in settings, THE Privacy_Sync SHALL update the GitHub repository visibility accordingly
2. WHEN the app loads, THE Privacy_Sync SHALL fetch current repository visibility and update local settings to match
3. IF privacy sync fails, THEN THE Privacy_Sync SHALL notify the user and show the actual repository state
4. THE Privacy_Sync SHALL require user confirmation before changing repository from private to public
5. WHILE syncing privacy settings, THE Privacy_Sync SHALL display a loading indicator

### Requirement 4: Resizable Photo Previews

**User Story:** As a user, I want to resize photo previews by dragging a corner handle, so that I can customize the gallery view to my preference.

#### Acceptance Criteria

1. THE Photo_Preview SHALL display a resize handle in the bottom-right corner of each photo
2. WHEN the user drags the resize handle, THE Photo_Preview SHALL resize proportionally
3. THE Photo_Preview SHALL persist the user's preferred preview size across sessions
4. THE Photo_Preview SHALL enforce minimum and maximum size constraints (80px to 400px)
5. WHEN multiple photos are selected, THE Photo_Preview SHALL apply resize to all selected photos simultaneously

### Requirement 5: Favorites System

**User Story:** As a user, I want to mark photos and albums as favorites, so that I can quickly access my preferred content.

#### Acceptance Criteria

1. THE Photo_Preview SHALL display a heart icon in the top-right corner for favoriting
2. WHEN the user clicks the heart icon, THE Favorites_System SHALL toggle the favorite status
3. THE Favorites_System SHALL persist favorites to local storage
4. THE Favorites_System SHALL display favorited items in a dedicated "Favorites" view
5. THE Favorites_System SHALL support favoriting entire albums/folders
6. WHEN viewing favorites, THE Favorites_System SHALL group items by type (photos, albums)

### Requirement 6: Color Tagging System

**User Story:** As a user, I want to assign color tags to photos and albums, so that I can visually organize and filter my content.

#### Acceptance Criteria

1. WHEN the user right-clicks selected items, THE Context_Menu SHALL display a color selection submenu
2. THE Color_Tag system SHALL provide at least 8 predefined colors (red, orange, yellow, green, blue, purple, pink, gray)
3. WHEN a color is selected, THE Color_Tag system SHALL apply the tag to all selected items
4. THE Color_Tag system SHALL automatically add used colors to the sidebar with a default name
5. WHEN the user double-clicks a color tag name in sidebar, THE Color_Tag system SHALL allow renaming
6. THE Color_Tag system SHALL persist tags and names to local storage
7. WHEN clicking a color tag in sidebar, THE Photo_Manager SHALL filter to show only items with that tag

### Requirement 7: Multi-Selection and Batch Operations

**User Story:** As a user, I want to select multiple photos and albums to perform batch operations, so that I can efficiently manage large collections.

#### Acceptance Criteria

1. WHEN the user holds Ctrl/Cmd and clicks items, THE Photo_Manager SHALL add items to selection
2. WHEN the user holds Shift and clicks, THE Photo_Manager SHALL select range of items
3. THE Photo_Manager SHALL visually indicate selected items with a highlight
4. WHEN right-clicking a selection, THE Context_Menu SHALL show batch operation options
5. THE Context_Menu SHALL include options for: color tag, favorite, delete, move to album

### Requirement 8: Album Management in Sidebar

**User Story:** As a user, I want to see and navigate my albums in the sidebar, so that I can easily browse my photo organization.

#### Acceptance Criteria

1. THE Photo_Manager SHALL display albums as expandable tree items in the sidebar
2. WHEN an album contains sub-albums, THE Photo_Manager SHALL show expand/collapse controls
3. WHEN the user clicks an album, THE Photo_Manager SHALL display only photos in that album
4. THE Photo_Manager SHALL show photo count next to each album name
5. THE Photo_Manager SHALL support drag-and-drop of photos into albums in sidebar

### Requirement 9: Modern Theme System

**User Story:** As a user, I want a modern, customizable dark theme with Matrix/Apple 2030 aesthetics, so that the app feels premium and matches my style.

#### Acceptance Criteria

1. THE Theme_Engine SHALL provide a base dark theme with black-gray color palette
2. THE Theme_Engine SHALL support accent color customization
3. THE Theme_Engine SHALL include subtle Matrix-style visual effects (optional scanlines, glow effects)
4. THE Theme_Engine SHALL provide smooth animations and transitions throughout the UI
5. THE Theme_Engine SHALL persist theme preferences across sessions
6. THE Theme_Engine SHALL ensure all UI elements maintain accessibility contrast ratios

### Requirement 10: Responsive Gallery Layout

**User Story:** As a user, I want the photo gallery to be fully responsive and adapt to different window sizes, so that I can use the app on any screen.

#### Acceptance Criteria

1. THE Photo_Manager SHALL use CSS Grid with auto-fill for responsive photo layout
2. WHEN the window is resized, THE Photo_Manager SHALL reflow photos without layout jumps
3. THE Photo_Manager SHALL support both grid and list view modes
4. THE Photo_Manager SHALL remember the user's preferred view mode
5. WHEN in list view, THE Photo_Manager SHALL show photo metadata (name, size, date)

### Requirement 11: Enhanced Upload Progress

**User Story:** As a user, I want detailed upload progress feedback, so that I know exactly what's happening with my uploads.

#### Acceptance Criteria

1. WHEN uploading multiple files, THE Photo_Manager SHALL show individual progress for each file
2. THE Photo_Manager SHALL display overall batch progress percentage
3. THE Photo_Manager SHALL show upload speed and estimated time remaining
4. IF an upload fails, THEN THE Photo_Manager SHALL allow retry for individual files
5. THE Photo_Manager SHALL support canceling uploads in progress

### Requirement 12: Photo Metadata Display

**User Story:** As a user, I want to see photo metadata in the lightbox view, so that I can understand details about my photos.

#### Acceptance Criteria

1. WHEN viewing a photo in lightbox, THE Photo_Manager SHALL display file name, size, and dimensions
2. THE Photo_Manager SHALL show upload date and any color tags applied
3. THE Photo_Manager SHALL provide a button to copy the direct image URL
4. THE Photo_Manager SHALL show favorite status with toggle capability in lightbox

### Requirement 13: Persistent Upload Progress Toast

**User Story:** As a user, I want to see a persistent toast notification showing upload/download progress, so that I can monitor file transfers without interrupting my workflow.

#### Acceptance Criteria

1. WHEN files are being uploaded or downloaded, THE Photo_Manager SHALL display a toast notification in the bottom-right corner
2. THE Toast SHALL remain visible until all transfers reach 100% completion
3. THE Toast SHALL display a summary showing "(N) files being uploaded/downloaded"
4. WHEN the user clicks the toast summary, THE Photo_Manager SHALL expand a dropdown showing individual file progress
5. THE expanded dropdown SHALL show each file name with its individual percentage progress
6. WHEN all transfers complete successfully, THE Toast SHALL auto-dismiss after a brief delay (3 seconds)
7. IF any transfer fails, THEN THE Toast SHALL remain visible and highlight failed files
8. THE Toast SHALL support minimizing back to summary view while transfers continue

### Requirement 14: Photo Sync Status Indicator

**User Story:** As a user, I want to see the sync status of each photo, so that I know which photos are local-only, remote-only, or fully synced.

#### Acceptance Criteria

1. THE Photo_Preview SHALL display a sync status indicator icon on each photo
2. THE Sync_Status_Indicator SHALL show distinct states: "local only", "remote not downloaded", "synced"
3. WHEN the user hovers over the sync status indicator, THE Photo_Manager SHALL display a tooltip explaining the current status
4. WHEN the user clicks the sync status indicator, THE Photo_Manager SHALL display a menu with sync options
5. THE sync options menu SHALL include: "Upload to GitHub", "Download from GitHub", "Remove local copy", "Delete from GitHub"
6. THE sync options menu SHALL only show relevant options based on current sync state
7. WHEN the user selects a sync action, THE Photo_Manager SHALL execute the action and update the indicator accordingly
8. THE Sync_Status_Indicator SHALL use distinct visual styling (icons/colors) for each state to be easily distinguishable

