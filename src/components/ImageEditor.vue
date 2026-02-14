<template>
  <div 
    class="w-full h-full relative bg-slate-200 overflow-hidden select-none cursor-default"
    ref="containerRef"
    @wheel.prevent="handleWheel"
    @mousedown.prevent="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <canvas 
      ref="canvasRef"
      class="block shadow-lg"
    ></canvas>
    
    <!-- Confirm / Cancel Actions for Pending Operation (Crop) -->
    <div 
      v-if="pendingAction"
      class="absolute top-4 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-slate-800/80 backdrop-blur text-white rounded-lg shadow-xl z-50 transform transition-all"
      @mousedown.stop
    >
      <span>{{ pendingActionText }}</span>
      <button @click="confirmAction" class="bg-blue-600 hover:bg-blue-500 rounded px-2 py-1 text-xs">确认 (Enter)</button>
      <button @click="cancelAction" class="bg-slate-600 hover:bg-slate-500 rounded px-2 py-1 text-xs">取消 (Esc)</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, nextTick, onUnmounted } from 'vue'

const props = defineProps({
  src: { type: String, default: '' },
  tool: { type: String, default: 'move' } // 'move', 'crop', 'blur'
})

const emit = defineEmits(['update:canUndo', 'update:imageInfo'])

const containerRef = ref(null)
const canvasRef = ref(null)
const ctx = ref(null)

// Image State
let image = null
let originalWidth = 0
let originalHeight = 0

// Viewport State
const scale = ref(1)
const offset = ref({ x: 0, y: 0 })

// Interaction State
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const startOffset = ref({ x: 0, y: 0 })

// Selection State (Image Coordinates)
const selection = ref(null) // { x, y, w, h } or null
const isSelecting = ref(false)
const selectionStart = ref({ x: 0, y: 0 })

// History Stack (ImageData or base64)
const history = ref([])
const historyIndex = ref(-1)

// Pending Action (e.g., confirm crop)
const pendingAction = ref(null) // 'crop'
const pendingActionText = ref('')

// Resize Observer
let resizeObserver = null

// Initialize




const clearCanvas = () => {
    console.log('Clearing canvas')
    image = null
    offscreenCanvas = null
    offscreenCtx = null
    originalWidth = 0
    originalHeight = 0
    history.value = []
    historyIndex.value = -1
    selection.value = null
    render()
    updateImageInfo()
}

const loadImage = (src) => {
  console.log('Loading image:', src)
  image = new Image()
  image.crossOrigin = 'Anonymous'
  image.onload = () => {
    console.log('Image loaded:', image.naturalWidth, image.naturalHeight)
    originalWidth = image.naturalWidth
    originalHeight = image.naturalHeight
    
    // Clear buffers to ensure fresh start
    offscreenCanvas = null
    offscreenCtx = null
    history.value = []
    historyIndex.value = -1
    selection.value = null
    
    fitToContainer()
    saveState()
    
    render()
    updateImageInfo()
  }
  image.onerror = (err) => {
    console.error('Failed to load image object', err)
  }
  image.src = src
}

const zoomIn = () => {
  if (!image) return
  scale.value *= 1.2
  render()
}

const zoomOut = () => {
  if (!image) return
  scale.value /= 1.2
  render()
}


const fitToContainer = () => {
  if (!containerRef.value || !image) return
  
  const { clientWidth, clientHeight } = containerRef.value
  const imageAspect = originalWidth / originalHeight
  const containerAspect = clientWidth / clientHeight
  
  if (imageAspect > containerAspect) {
    scale.value = (clientWidth * 0.9) / originalWidth
  } else {
    scale.value = (clientHeight * 0.9) / originalHeight
  }
  
  offset.value = {
    x: (clientWidth - originalWidth * scale.value) / 2,
    y: (clientHeight - originalHeight * scale.value) / 2
  }
}

const render = () => {
  if (!ctx.value || !containerRef.value) return
  const { clientWidth, clientHeight } = containerRef.value
  
  // Update canvas logic size to match container for crisp drawing
  if (canvasRef.value.width !== clientWidth || canvasRef.value.height !== clientHeight) {
    canvasRef.value.width = clientWidth
    canvasRef.value.height = clientHeight
  }
  
  const context = ctx.value
  context.clearRect(0, 0, clientWidth, clientHeight)
  
  if (!image || originalWidth === 0 || originalHeight === 0) return
  
  context.save()
  context.translate(offset.value.x, offset.value.y)
  context.scale(scale.value, scale.value)
  
  // Draw current image state
  // Typically we draw the *current state* which might be modified.
  // Ideally, 'image' holds the base image, and history holds modifications?
  // Or 'image' is always the current modified image.
  // The 'image' object source is static. But if we edit, we update 'image.src'?
  // That is slow.
  // Better: Draw 'image', then draw overlay? No, operations modify the pixels.
  // So 'image' should point to an offscreen canvas or similar that holds current state.
  // Let's use an offscreen canvas as the source of truth if we edit.
  // Actually, keeping history as ImageData allows restoring.
  // But drawing ImageData every frame when panning/zooming is slow if image is huge (4k).
  // Strategy:
  // 1. 'currentOffscreen' canvas holds the full-res edited image.
  // 2. We draw 'currentOffscreen' to the display canvas with transform.
  // 3. History saves 'currentOffscreen' content (or Blob/DataURL).
  
  context.drawImage(getCurrentImageSource(), 0, 0)
  
  // Draw selection if any
  if (selection.value) {
    const { x, y, w, h } = selection.value
    // Draw Overlay
    context.fillStyle = 'rgba(0, 0, 0, 0.5)'
    // Outer areas (Mask)
    // Actually easier: fill whole rect with semi-transparent, then clearRect the selection?
    // But we are inside transformed context.
    
    // Draw Selection Border
    context.strokeStyle = '#fff'
    context.lineWidth = 1 / scale.value
    context.setLineDash([5 / scale.value, 5 / scale.value])
    context.strokeRect(x, y, w, h)
    
    // Draw Selection Border (Black contrast)
    context.strokeStyle = '#000'
    context.setLineDash([5 / scale.value, 5 / scale.value])
    context.lineDashOffset = 5 / scale.value
    context.strokeRect(x, y, w, h)
  }
  
  context.restore()
  
  // Update Image Info
  updateImageInfo()
}

// Offscreen buffer for edits
let offscreenCanvas = null
let offscreenCtx = null

const getCurrentImageSource = () => {
  if (!offscreenCanvas && image) {
    offscreenCanvas = document.createElement('canvas')
    offscreenCanvas.width = originalWidth
    offscreenCanvas.height = originalHeight
    offscreenCtx = offscreenCanvas.getContext('2d')
    offscreenCtx.drawImage(image, 0, 0)
  }
  return offscreenCanvas || image
}

const saveState = () => {
  // Save current offscreen canvas state to history
  if (!offscreenCanvas) getCurrentImageSource()
  
  // Limit history size
  if (historyIndex.value < history.value.length - 1) {
    // Truncate future
    history.value = history.value.slice(0, historyIndex.value + 1)
  }
  
  const data = offscreenCtx.getImageData(0, 0, originalWidth, originalHeight)
  history.value.push(data)
  historyIndex.value++
  
  emit('update:canUndo', historyIndex.value > 0)
}

const undo = () => {
  if (historyIndex.value > 0) {
    historyIndex.value--
    const data = history.value[historyIndex.value]
    
    // Restore size if changed (crop)
    if (data.width !== originalWidth || data.height !== originalHeight) {
      originalWidth = data.width
      originalHeight = data.height
      offscreenCanvas.width = originalWidth
      offscreenCanvas.height = originalHeight
      // Re-fit view? Maybe.
    }
    
    offscreenCtx.putImageData(data, 0, 0)
    selection.value = null
    render()
    emit('update:canUndo', historyIndex.value > 0)
  }
}

const reset = () => {
  if (history.value.length > 0) {
    const initial = history.value[0]
    history.value = [initial]
    historyIndex.value = 0
    
    originalWidth = initial.width
    originalHeight = initial.height
    offscreenCanvas.width = originalWidth
    offscreenCanvas.height = originalHeight
    
    offscreenCtx.putImageData(initial, 0, 0)
    selection.value = null
    fitToContainer()
    render()
    emit('update:canUndo', false)
  }
}

// Handling Inputs
const getPointInImage = (e) => {
  const rect = canvasRef.value.getBoundingClientRect()
  const displayX = e.clientX - rect.left
  const displayY = e.clientY - rect.top
  
  // Inverse Transform
  // screen = (image * scale) + offset
  // image = (screen - offset) / scale
  const imgX = (displayX - offset.value.x) / scale.value
  const imgY = (displayY - offset.value.y) / scale.value
  return { x: imgX, y: imgY }
}

const handleWheel = (e) => {
  const zoomFactor = 1.1
  const direction = e.deltaY > 0 ? 1 / zoomFactor : zoomFactor
  
  const rect = canvasRef.value.getBoundingClientRect()
  const mouseX = e.clientX - rect.left
  const mouseY = e.clientY - rect.top
  
  // Zoom around mouse point
  // newScale = scale * direction
  // newOffset = mouse - (mouse - offset) * direction
  
  const newScale = Math.max(0.01, Math.min(50, scale.value * direction))
  const newOffset = {
    x: mouseX - (mouseX - offset.value.x) * direction,
    y: mouseY - (mouseY - offset.value.y) * direction
  }
  
  scale.value = newScale
  offset.value = newOffset
  render()
}

const handleMouseDown = (e) => {
  if (!image) return
  
  if (e.button === 1 || props.tool === 'move' || e.ctrlKey) {
    // Pan
    isDragging.value = true
    dragStart.value = { x: e.clientX, y: e.clientY }
    startOffset.value = { ...offset.value }
    return
  }
  
  if (props.tool === 'crop' || props.tool === 'blur') {
    isSelecting.value = true
    const pt = getPointInImage(e)
    selectionStart.value = pt
    selection.value = { x: pt.x, y: pt.y, w: 0, h: 0 }
  }
}

const handleMouseMove = (e) => {
  if (isDragging.value) {
    const dx = e.clientX - dragStart.value.x
    const dy = e.clientY - dragStart.value.y
    offset.value = {
      x: startOffset.value.x + dx,
      y: startOffset.value.y + dy
    }
    render()
    return
  }
  
  if (isSelecting.value) {
    const pt = getPointInImage(e)
    const w = pt.x - selectionStart.value.x
    const h = pt.y - selectionStart.value.y
    selection.value = {
      x: w > 0 ? selectionStart.value.x : pt.x,
      y: h > 0 ? selectionStart.value.y : pt.y,
      w: Math.abs(w),
      h: Math.abs(h)
    }
    render()
  }
}

const handleMouseUp = () => {
  isDragging.value = false
  
  if (isSelecting.value) {
    isSelecting.value = false
    handleSelectionComplete()
  }
}

const handleSelectionComplete = () => {
  if (!selection.value || selection.value.w < 2 || selection.value.h < 2) {
    selection.value = null
    render()
    return
  }
  
  if (props.tool === 'crop') {
    pendingAction.value = 'crop'
    pendingActionText.value = '确认裁剪？'
  } else if (props.tool === 'blur') {
    applyBlur()
  }
}

const confirmAction = () => {
  if (pendingAction.value === 'crop') {
    applyCrop()
  }
  cancelAction()
}

const cancelAction = () => {
  pendingAction.value = null
  selection.value = null
  render()
}

const applyCrop = () => {
  if (!selection.value) return
  const { x, y, w, h } = selection.value
  
  // Crop offscreen canvas
  const cropped = offscreenCtx.getImageData(x, y, w, h)
  
  // Resize offscreen canvas
  originalWidth = w
  originalHeight = h
  offscreenCanvas.width = w
  offscreenCanvas.height = h
  offscreenCtx.putImageData(cropped, 0, 0)
  
  selection.value = null
  saveState()
  fitToContainer() // Re-center
  render()
}

const applyBlur = () => {
  if (!selection.value) return
  const { x, y, w, h } = selection.value
  
  // Get data of selection
  // Note: getImageData snaps to device pixels, so the resulting imageData.width/height might differ slightly from w/h
  const imageData = offscreenCtx.getImageData(x, y, w, h)
  const data = imageData.data
  const { width: realW, height: realH } = imageData
  
  // Simple Mosaic Blur
  const blockSize = Math.max(4, Math.min(realW, realH) / 20)
  
  // Process mosaic
  for (let by = 0; by < realH; by += blockSize) {
    for (let bx = 0; bx < realW; bx += blockSize) {
      // Get color of block center
      let r = 0, g = 0, b = 0, count = 0
      
      // Calculate average color of block
      for (let iy = 0; iy < blockSize && by + iy < realH; iy++) {
        for (let ix = 0; ix < blockSize && bx + ix < realW; ix++) {
          const i = ((by + iy) * realW + (bx + ix)) * 4
          r += data[i]
          g += data[i+1]
          b += data[i+2]
          count++
        }
      }
      
      r = Math.round(r / count)
      g = Math.round(g / count)
      b = Math.round(b / count)
      
      // Fill block
      for (let iy = 0; iy < blockSize && by + iy < realH; iy++) {
        for (let ix = 0; ix < blockSize && bx + ix < realW; ix++) {
          const i = ((by + iy) * realW + (bx + ix)) * 4
          data[i] = r
          data[i+1] = g
          data[i+2] = b
          data[i+3] = 255 // Alpha full
        }
      }
    }
  }
  
  offscreenCtx.putImageData(imageData, x, y)
  selection.value = null
  saveState()
  render()
}

const handleGlobalKey = (e) => {
  if (e.key === 'Enter' && pendingAction.value) {
    confirmAction()
  } else if (e.key === 'Escape') {
    cancelAction()
  } else if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
    undo()
  }
}



const updateImageInfo = () => {
  if (!image) {
    emit('update:imageInfo', '')
    return
  }
  const zoom = Math.round(scale.value * 100)
  emit('update:imageInfo', `${Math.round(originalWidth)} x ${Math.round(originalHeight)} px | ${zoom}%`)
}

const exportImage = async () => {
  if (!offscreenCanvas) return null
  return offscreenCanvas.toDataURL('image/png')
}

watch(() => props.src, (newSrc) => {
  if (newSrc) {
    loadImage(newSrc)
  } else {
    clearCanvas()
  }
}, { immediate: true })

watch(() => props.tool, (newTool) => {
  cancelAction()
  selection.value = null
  render()
})

onMounted(() => {
  ctx.value = canvasRef.value.getContext('2d')
  window.addEventListener('keydown', handleGlobalKey)
  
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
        // Debounce render if needed, but for canvas resize usually direct sync is okay if handled carefully
        // Here we just re-render which checks canvas size vs client size
        requestAnimationFrame(render);
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKey)
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

defineExpose({
  undo,
  reset,
  zoomIn,
  zoomOut,
  exportImage
})
</script>

<style scoped>
/* Optional cursors */
</style>
