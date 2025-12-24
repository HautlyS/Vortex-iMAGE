# Initial Analysis

This is a brownfield project. Here is a summary of the analysis.

## Project Goal

The project is a photo storage application named "Vortex - iMAGE" that uses GitHub as a backend. It allows users to upload photos to a GitHub repository.

## Tech Stack

- **Backend:** Rust with Tauri (v2)
- **Frontend:** Vue.js (v3) with TypeScript and Vite
- **Styling:** Tailwind CSS (v4)
- **Authentication:** GitHub OAuth Device Flow
- **Platform:** Desktop (Windows, macOS, Linux) and Mobile (Android, iOS)

## Key Features

- Photo upload to a selected GitHub repository.
- Drag-and-drop support for uploads.
- Git LFS support for large files.
- AMOLED-friendly dark theme with customizable accent colors.
- Offline token persistence for GitHub authentication.
- Mobile support for both Android and iOS.

---

## Product Definition

### Target Audience

The application is intended for a worldwide community of users who need a simple and effective way to store their photos.

### Main Goals

The primary goal is to provide a secure and private photo storage solution, leveraging GitHub for storage.

### Key Features for Initial Version

- **End-to-end encryption:** All photos will be encrypted to ensure user privacy.

### Non-Functional Requirements

- **High performance:** The application should be fast and responsive, with a focus on quick image loading and upload speeds.