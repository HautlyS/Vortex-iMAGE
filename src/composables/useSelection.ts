/**
 * TypeScript Module - 3 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'

const selected = ref<Set<string>>(new Set())
const lastSelected = ref<string | null>(null)

export function selectAdditive(id: string, currentSelection: Set<string>): Set<string> {
  const newSelection = new Set(currentSelection)
  if (newSelection.has(id)) {
    newSelection.delete(id)
  } else {
    newSelection.add(id)
  }
  return newSelection
}

export function selectRange(
  anchorId: string | null,
  targetId: string,
  allIds: string[],
  currentSelection: Set<string>
): Set<string> {
  if (!anchorId) {
    
    return new Set([targetId])
  }

  const anchorIndex = allIds.indexOf(anchorId)
  const targetIndex = allIds.indexOf(targetId)

  if (anchorIndex === -1 || targetIndex === -1) {
    
    return new Set([targetId])
  }

  const start = Math.min(anchorIndex, targetIndex)
  const end = Math.max(anchorIndex, targetIndex)

  const newSelection = new Set(currentSelection)
  for (let i = start; i <= end; i++) {
    newSelection.add(allIds[i])
  }

  return newSelection
}

export function useSelection() {
  const selectedCount = computed(() => selected.value.size)

  function select(
    id: string,
    options: { additive?: boolean; range?: boolean } = {},
    allIds: string[] = []
  ): void {
    const { additive = false, range = false } = options

    if (range && lastSelected.value) {
      selected.value = selectRange(lastSelected.value, id, allIds, selected.value)
    } else if (additive) {
      selected.value = selectAdditive(id, selected.value)
    } else {
      
      selected.value = new Set([id])
    }

    lastSelected.value = id
  }

  function clearSelection(): void {
    selected.value = new Set()
    lastSelected.value = null
  }

  function isSelected(id: string): boolean {
    return selected.value.has(id)
  }

  function getSelected(): string[] {
    return Array.from(selected.value)
  }

  function selectAll(ids: string[]): void {
    selected.value = new Set(ids)
    if (ids.length > 0) {
      lastSelected.value = ids[ids.length - 1]
    }
  }

  return {
    selected,
    lastSelected,
    selectedCount,
    select,
    clearSelection,
    isSelected,
    getSelected,
    selectAll,
  }
}