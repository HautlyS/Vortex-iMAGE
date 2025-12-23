export interface ColorTag {
  id: string
  name: string
  color: string
}

export const COLORS: ColorTag[] = [
  { id: 'red', name: 'Vermelho', color: '#ef4444' },
  { id: 'orange', name: 'Laranja', color: '#f97316' },
  { id: 'yellow', name: 'Amarelo', color: '#eab308' },
  { id: 'green', name: 'Verde', color: '#22c55e' },
  { id: 'blue', name: 'Azul', color: '#3b82f6' },
  { id: 'purple', name: 'Roxo', color: '#a855f7' },
  { id: 'pink', name: 'Rosa', color: '#ec4899' },
  { id: 'gray', name: 'Cinza', color: '#6b7280' },
]

class ColorTagManager {
  private tags: Record<string, string> = {}

  setTag(photoId: string, colorId: string) {
    this.tags[photoId] = colorId
  }

  getTag(photoId: string): ColorTag | null {
    const colorId = this.tags[photoId]
    return colorId ? COLORS.find(c => c.id === colorId) || null : null
  }

  removeTag(photoId: string) {
    delete this.tags[photoId]
  }

  getAllTags() {
    return { ...this.tags }
  }

  setAllTags(tags: Record<string, string>) {
    this.tags = { ...tags }
  }
}

export const colorTagManager = new ColorTagManager()
