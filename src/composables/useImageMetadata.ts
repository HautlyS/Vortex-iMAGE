/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref } from 'vue'

export interface ImageMetadata {
  
  fileName: string
  fileSize: number
  fileType: string
  dimensions?: { width: number; height: number }

  camera?: {
    make?: string
    model?: string
    lens?: string
  }
  settings?: {
    aperture?: string
    shutterSpeed?: string
    iso?: number
    focalLength?: string
    exposureMode?: string
    whiteBalance?: string
    flash?: string
  }

  dateTime?: {
    original?: Date
    digitized?: Date
    modified?: Date
  }
  location?: {
    latitude?: number
    longitude?: number
    altitude?: number
    address?: string
  }

  software?: string
  copyright?: string
  artist?: string
  description?: string
  orientation?: number
  colorSpace?: string
}

const loading = ref(false)
const error = ref<string | null>(null)

async function extractMetadata(source: File | string): Promise<ImageMetadata | null> {
  loading.value = true
  error.value = null
  
  try {
    let blob: Blob
    let fileName: string
    
    if (source instanceof File) {
      blob = source
      fileName = source.name
    } else {
      
      const response = await fetch(source)
      blob = await response.blob()
      fileName = source.split('/').pop() || 'unknown'
    }
    
    const metadata: ImageMetadata = {
      fileName,
      fileSize: blob.size,
      fileType: blob.type || 'image/unknown',
    }

    const dimensions = await getImageDimensions(blob)
    if (dimensions) {
      metadata.dimensions = dimensions
    }

    const exifData = await extractExifData(blob)
    if (exifData) {
      Object.assign(metadata, exifData)
    }
    
    return metadata
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to extract metadata'
    return null
  } finally {
    loading.value = false
  }
}

async function getImageDimensions(blob: Blob): Promise<{ width: number; height: number } | null> {
  return new Promise((resolve) => {
    const img = new Image()
    const url = URL.createObjectURL(blob)
    
    img.onload = () => {
      URL.revokeObjectURL(url)
      resolve({ width: img.naturalWidth, height: img.naturalHeight })
    }
    
    img.onerror = () => {
      URL.revokeObjectURL(url)
      resolve(null)
    }
    
    img.src = url
  })
}

async function extractExifData(blob: Blob): Promise<Partial<ImageMetadata> | null> {
  try {
    const buffer = await blob.arrayBuffer()
    const view = new DataView(buffer)

    if (view.getUint16(0) !== 0xFFD8) {
      return null 
    }
    
    let offset = 2
    const length = view.byteLength
    
    while (offset < length) {
      if (view.getUint8(offset) !== 0xFF) {
        offset++
        continue
      }
      
      const marker = view.getUint8(offset + 1)

      if (marker === 0xE1) {
        const segmentLength = view.getUint16(offset + 2)
        const exifData = parseExifSegment(view, offset + 4, segmentLength - 2)
        return exifData
      }

      if (marker >= 0xE0 && marker <= 0xEF) {
        const segmentLength = view.getUint16(offset + 2)
        offset += 2 + segmentLength
      } else {
        offset += 2
      }
    }
    
    return null
  } catch {
    return null
  }
}

function parseExifSegment(view: DataView, start: number, _length: number): Partial<ImageMetadata> | null {
  
  const exifHeader = String.fromCharCode(
    view.getUint8(start),
    view.getUint8(start + 1),
    view.getUint8(start + 2),
    view.getUint8(start + 3)
  )
  
  if (exifHeader !== 'Exif') {
    return null
  }
  
  const tiffStart = start + 6
  const littleEndian = view.getUint16(tiffStart) === 0x4949
  
  const result: Partial<ImageMetadata> = {
    camera: {},
    settings: {},
    dateTime: {},
  }
  
  try {
    
    const ifd0Offset = view.getUint32(tiffStart + 4, littleEndian)
    const ifd0Start = tiffStart + ifd0Offset

    const numEntries = view.getUint16(ifd0Start, littleEndian)
    
    for (let i = 0; i < numEntries; i++) {
      const entryOffset = ifd0Start + 2 + (i * 12)
      const tag = view.getUint16(entryOffset, littleEndian)
      const type = view.getUint16(entryOffset + 2, littleEndian)
      const count = view.getUint32(entryOffset + 4, littleEndian)
      const valueOffset = entryOffset + 8
      
      switch (tag) {
        case 0x010F: 
          result.camera!.make = readString(view, tiffStart, valueOffset, count, littleEndian)
          break
        case 0x0110: 
          result.camera!.model = readString(view, tiffStart, valueOffset, count, littleEndian)
          break
        case 0x0132: 
          const dateStr = readString(view, tiffStart, valueOffset, count, littleEndian)
          if (dateStr) {
            result.dateTime!.modified = parseExifDate(dateStr)
          }
          break
        case 0x0112: 
          result.orientation = type === 3 
            ? view.getUint16(valueOffset, littleEndian)
            : view.getUint32(valueOffset, littleEndian)
          break
        case 0x8769: 
          const exifOffset = view.getUint32(valueOffset, littleEndian)
          parseExifIFD(view, tiffStart + exifOffset, tiffStart, littleEndian, result)
          break
        case 0x8825: 
          const gpsOffset = view.getUint32(valueOffset, littleEndian)
          parseGpsIFD(view, tiffStart + gpsOffset, tiffStart, littleEndian, result)
          break
      }
    }
  } catch {
    
  }
  
  return result
}

function parseExifIFD(
  view: DataView, 
  ifdStart: number, 
  tiffStart: number, 
  littleEndian: boolean, 
  result: Partial<ImageMetadata>
): void {
  try {
    const numEntries = view.getUint16(ifdStart, littleEndian)
    
    for (let i = 0; i < numEntries; i++) {
      const entryOffset = ifdStart + 2 + (i * 12)
      const tag = view.getUint16(entryOffset, littleEndian)
      const type = view.getUint16(entryOffset + 2, littleEndian)
      const count = view.getUint32(entryOffset + 4, littleEndian)
      const valueOffset = entryOffset + 8
      
      switch (tag) {
        case 0x829A: 
          const expNum = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          const expDen = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian) + 4, littleEndian)
          result.settings!.shutterSpeed = expDen > expNum ? `1/${Math.round(expDen/expNum)}` : `${expNum/expDen}s`
          break
        case 0x829D: 
          const fNum = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          const fDen = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian) + 4, littleEndian)
          result.settings!.aperture = `f/${(fNum/fDen).toFixed(1)}`
          break
        case 0x8827: 
          result.settings!.iso = type === 3 
            ? view.getUint16(valueOffset, littleEndian)
            : view.getUint32(valueOffset, littleEndian)
          break
        case 0x920A: 
          const flNum = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          const flDen = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian) + 4, littleEndian)
          result.settings!.focalLength = `${Math.round(flNum/flDen)}mm`
          break
        case 0x9003: 
          const origDateStr = readString(view, tiffStart, valueOffset, count, littleEndian)
          if (origDateStr) {
            result.dateTime!.original = parseExifDate(origDateStr)
          }
          break
        case 0x9004: 
          const digDateStr = readString(view, tiffStart, valueOffset, count, littleEndian)
          if (digDateStr) {
            result.dateTime!.digitized = parseExifDate(digDateStr)
          }
          break
        case 0xA402: 
          const expMode = view.getUint16(valueOffset, littleEndian)
          result.settings!.exposureMode = ['Auto', 'Manual', 'Auto Bracket'][expMode] || 'Unknown'
          break
        case 0xA403: 
          const wb = view.getUint16(valueOffset, littleEndian)
          result.settings!.whiteBalance = wb === 0 ? 'Auto' : 'Manual'
          break
        case 0x9209: 
          const flash = view.getUint16(valueOffset, littleEndian)
          result.settings!.flash = (flash & 1) ? 'Fired' : 'Did not fire'
          break
        case 0xA434: 
          result.camera!.lens = readString(view, tiffStart, valueOffset, count, littleEndian)
          break
      }
    }
  } catch {
    
  }
}

function parseGpsIFD(
  view: DataView, 
  ifdStart: number, 
  tiffStart: number, 
  littleEndian: boolean, 
  result: Partial<ImageMetadata>
): void {
  try {
    result.location = {}
    const numEntries = view.getUint16(ifdStart, littleEndian)
    
    let latRef = 'N', lonRef = 'E'
    let lat: number[] = [], lon: number[] = []
    
    for (let i = 0; i < numEntries; i++) {
      const entryOffset = ifdStart + 2 + (i * 12)
      const tag = view.getUint16(entryOffset, littleEndian)
      const valueOffset = entryOffset + 8
      
      switch (tag) {
        case 0x0001: 
          latRef = String.fromCharCode(view.getUint8(valueOffset))
          break
        case 0x0002: 
          lat = readRational3(view, tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          break
        case 0x0003: 
          lonRef = String.fromCharCode(view.getUint8(valueOffset))
          break
        case 0x0004: 
          lon = readRational3(view, tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          break
        case 0x0006: 
          const altNum = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian), littleEndian)
          const altDen = view.getUint32(tiffStart + view.getUint32(valueOffset, littleEndian) + 4, littleEndian)
          result.location!.altitude = altNum / altDen
          break
      }
    }
    
    if (lat.length === 3) {
      result.location!.latitude = (lat[0] + lat[1]/60 + lat[2]/3600) * (latRef === 'S' ? -1 : 1)
    }
    if (lon.length === 3) {
      result.location!.longitude = (lon[0] + lon[1]/60 + lon[2]/3600) * (lonRef === 'W' ? -1 : 1)
    }
  } catch {
    
  }
}

function readString(
  view: DataView, 
  tiffStart: number, 
  valueOffset: number, 
  count: number, 
  littleEndian: boolean
): string {
  let offset = valueOffset
  if (count > 4) {
    offset = tiffStart + view.getUint32(valueOffset, littleEndian)
  }
  
  let str = ''
  for (let i = 0; i < count - 1; i++) {
    const char = view.getUint8(offset + i)
    if (char === 0) break
    str += String.fromCharCode(char)
  }
  return str.trim()
}

function readRational3(view: DataView, offset: number, littleEndian: boolean): number[] {
  const result: number[] = []
  for (let i = 0; i < 3; i++) {
    const num = view.getUint32(offset + i * 8, littleEndian)
    const den = view.getUint32(offset + i * 8 + 4, littleEndian)
    result.push(num / den)
  }
  return result
}

function parseExifDate(dateStr: string): Date | undefined {
  
  const match = dateStr.match(/(\d{4}):(\d{2}):(\d{2}) (\d{2}):(\d{2}):(\d{2})/)
  if (match) {
    return new Date(
      parseInt(match[1]),
      parseInt(match[2]) - 1,
      parseInt(match[3]),
      parseInt(match[4]),
      parseInt(match[5]),
      parseInt(match[6])
    )
  }
  return undefined
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

function formatDate(date: Date | undefined): string {
  if (!date) return '-'
  return date.toLocaleDateString('pt-BR', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function formatCoordinates(lat?: number, lon?: number): string {
  if (lat === undefined || lon === undefined) return '-'
  const latDir = lat >= 0 ? 'N' : 'S'
  const lonDir = lon >= 0 ? 'E' : 'W'
  return `${Math.abs(lat).toFixed(6)}° ${latDir}, ${Math.abs(lon).toFixed(6)}° ${lonDir}`
}

export function useImageMetadata() {
  return {
    loading,
    error,
    extractMetadata,
    formatFileSize,
    formatDate,
    formatCoordinates,
  }
}