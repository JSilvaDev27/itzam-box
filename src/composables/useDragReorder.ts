/* ItzamBox — Drag-to-Reorder Composable
   Copyright (C) 2026 SodigTech — GPL-3.0
   HTML5 Drag and Drop API for reordering a list of items.
   Provides drag handlers and drop zone indicators. */

import { ref, type Ref } from 'vue'

export interface DragReorderOptions<T> {
  /** The reactive list of items (will be mutated on reorder). */
  items: Ref<T[]>
  /** Callback fired after reorder completes. Receives the new ordered array. */
  onReorder: (items: T[]) => void
  /** Unique key accessor (default: identity). */
  key?: (item: T) => string | number
}

export interface DragReorderHandlers {
  /** Attach to each draggable element. */
  dragHandlers: {
    draggable: true
    onDragstart: (e: DragEvent, index: number) => void
    onDragend: (e: DragEvent) => void
  }
  /** Attach to each drop zone wrapper. */
  dropHandlers: (index: number) => {
    onDragOver: (e: DragEvent) => void
    onDragEnter: (e: DragEvent) => void
    onDragLeave: (e: DragEvent) => void
    onDrop: (e: DragEvent) => void
  }
  /** Reactive index of the item being dragged (-1 if none). */
  draggingIndex: Ref<number>
  /** Reactive index of the drop target (-1 if none). */
  dropTargetIndex: Ref<number>
}

/**
 * useDragReorder — HTML5 Drag and Drop for reorderable lists.
 *
 * Usage:
 *   const { dragHandlers, dropHandlers, draggingIndex } = useDragReorder({
 *     items: myList,
 *     onReorder: (newOrder) => { /* persist *\/ },
 *   })
 *
 * Template:
 *   <div v-for="(item, i) in items" :key="item.id"
 *     v-bind="dragHandlers(item, i)"
 *     ... >
 *     <div v-bind="dropHandlers(i)" />
 *   </div>
 */
export function useDragReorder<T>(
  options: DragReorderOptions<T>,
): DragReorderHandlers {
  const { items, onReorder } = options

  const draggingIndex = ref(-1)
  const dropTargetIndex = ref(-1)

  let dragImage: HTMLElement | null = null

  function onDragstart(e: DragEvent, index: number) {
    draggingIndex.value = index
    dropTargetIndex.value = -1

    // Custom drag image: semi-transparent clone
    const target = e.target as HTMLElement
    const clone = target.cloneNode(true) as HTMLElement
    clone.style.opacity = '0.7'
    clone.style.position = 'absolute'
    clone.style.top = '-1000px'
    clone.style.left = '-1000px'
    clone.style.width = target.offsetWidth + 'px'
    clone.style.background = 'var(--bg-secondary, #12161f)'
    clone.style.border = '1px solid var(--border-color, #262f45)'
    clone.style.borderRadius = '8px'
    clone.style.boxShadow = '0 8px 24px rgba(0,0,0,0.4)'
    clone.style.padding = '8px 12px'
    clone.style.pointerEvents = 'none'
    document.body.appendChild(clone)
    dragImage = clone

    e.dataTransfer?.setDragImage(clone, 20, 20)
    e.dataTransfer!.effectAllowed = 'move'

    // Add dragging class after a tick (so the browser captures the style)
    requestAnimationFrame(() => {
      target.classList.add('dragging')
    })
  }

  function onDragend(_e: DragEvent) {
    draggingIndex.value = -1
    dropTargetIndex.value = -1

    // Remove drag image
    if (dragImage && document.body.contains(dragImage)) {
      document.body.removeChild(dragImage)
      dragImage = null
    }

    // Remove dragging class from all items
    document.querySelectorAll('.dragging').forEach((el) => {
      el.classList.remove('dragging')
    })
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault()
    e.dataTransfer!.dropEffect = 'move'
  }

  function onDragEnter(e: DragEvent, index: number) {
    e.preventDefault()
    if (index !== draggingIndex.value) {
      dropTargetIndex.value = index
    }
  }

  function onDragLeave(_e: DragEvent) {
    dropTargetIndex.value = -1
  }

  function onDrop(e: DragEvent, index: number) {
    e.preventDefault()
    const from = draggingIndex.value
    if (from < 0 || from === index) {
      draggingIndex.value = -1
      dropTargetIndex.value = -1
      return
    }

    const newItems = [...items.value]
    const [moved] = newItems.splice(from, 1)
    newItems.splice(index, 0, moved)

    items.value = newItems
    onReorder(newItems)

    draggingIndex.value = -1
    dropTargetIndex.value = -1
  }

  const dragHandlers = {
    draggable: true as const,
    onDragstart,
    onDragend,
  }

  function dropHandlers(index: number) {
    return {
      onDragOver,
      onDragEnter: (e: DragEvent) => onDragEnter(e, index),
      onDragLeave,
      onDrop: (e: DragEvent) => onDrop(e, index),
    }
  }

  return {
    dragHandlers,
    dropHandlers,
    draggingIndex,
    dropTargetIndex,
  }
}
