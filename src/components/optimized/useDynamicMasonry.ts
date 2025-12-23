

export interface DynamicSize {
  width: number
  height: number
  scale: number
  type: 'small' | 'medium' | 'large' | 'hero' | 'wide' | 'tall'
  highlight: boolean
  group?: string
}

export function useDynamicMasonry(baseWidth = 280, baseHeight = 320) {
  const sizeVariants = {
    small: { scale: 0.7, weight: 0.4 },
    medium: { scale: 1.0, weight: 0.3 },
    large: { scale: 1.4, weight: 0.2 },
    hero: { scale: 2.0, weight: 0.05 },
    wide: { scale: 1.2, weight: 0.03 },
    tall: { scale: 1.3, weight: 0.02 }
  }

  const calculateDynamicSize = (item: any, index: number): DynamicSize => {
    const hash = item.id.split('').reduce((a: number, c: string) => a + c.charCodeAt(0), 0)
    const random = (hash + index) % 1000 / 1000

    // Determine size type based on weighted probability
    let cumulative = 0
    let sizeType: keyof typeof sizeVariants = 'medium'

    for (const [type, config] of Object.entries(sizeVariants)) {
      cumulative += config.weight
      if (random <= cumulative) {
        sizeType = type as keyof typeof sizeVariants
        break
      }
    }

    // Special patterns for visual interest
    const isHighlight = index % 7 === 0 || sizeType === 'hero'
    const isGrouped = index % 5 === 0 && sizeType === 'small'

    // Calculate dimensions based on original aspect ratio
    const originalAspect = item.width && item.height ? item.width / item.height : 1
    const scale = sizeVariants[sizeType].scale

    let width = baseWidth * scale
    let height = baseHeight * scale

    // Adjust for aspect ratio while maintaining size category
    if (sizeType === 'wide') {
      width = baseWidth * 1.8
      height = baseHeight * 0.8
    } else if (sizeType === 'tall') {
      width = baseWidth * 0.8
      height = baseHeight * 1.6
    } else {
      // Maintain aspect ratio within size bounds
      if (originalAspect > 1.5) { // Wide image
        height = width / originalAspect
      } else if (originalAspect < 0.7) { // Tall image
        width = height * originalAspect
      }
    }

    return {
      width: Math.round(width),
      height: Math.round(height),
      scale,
      type: sizeType,
      highlight: isHighlight,
      group: isGrouped ? `group-${Math.floor(index / 5)}` : undefined
    }
  }

  const generateMasonryLayout = (items: any[], containerWidth: number) => {
    const gap = 16
    const columns = Math.max(2, Math.floor(containerWidth / (baseWidth + gap)))
    const columnHeights = new Array(columns).fill(0)
    const positions: Array<{ x: number; y: number; item: any; size: DynamicSize }> = []

    items.forEach((item, index) => {
      const size = calculateDynamicSize(item, index)

      // Find best column (shortest for normal items, specific logic for special items)
      let targetColumn = 0

      if (size.type === 'hero') {
        // Hero images prefer left columns
        targetColumn = columnHeights.slice(0, Math.ceil(columns / 2))
          .reduce((minIdx, height, idx, arr) => height < arr[minIdx] ? idx : minIdx, 0)
      } else if (size.group) {
        // Grouped items try to stay together
        const groupColumn = index % columns
        targetColumn = groupColumn
      } else {
        // Normal items go to shortest column
        targetColumn = columnHeights.reduce((minIdx, height, idx) =>
          height < columnHeights[minIdx] ? idx : minIdx, 0)
      }

      // Handle column overflow for wide items
      if (size.type === 'wide' && targetColumn + 1 >= columns) {
        targetColumn = Math.max(0, columns - 2)
      }

      const x = targetColumn * (baseWidth + gap)
      const y = columnHeights[targetColumn]

      positions.push({ x, y, item, size })

      // Update column heights
      const itemWidth = size.width
      const affectedColumns = Math.ceil(itemWidth / (baseWidth + gap))

      for (let i = 0; i < affectedColumns && targetColumn + i < columns; i++) {
        columnHeights[targetColumn + i] = Math.max(
          columnHeights[targetColumn + i],
          y + size.height + gap
        )
      }
    })

    return {
      positions,
      totalHeight: Math.max(...columnHeights),
      columns
    }
  }

  return {
    calculateDynamicSize,
    generateMasonryLayout,
    sizeVariants
  }
}
