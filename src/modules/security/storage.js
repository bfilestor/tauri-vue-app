const fallbackState = new Map()

function createMapStorage(map) {
  return {
    getItem(key) {
      return map.has(key) ? map.get(key) : null
    },
    setItem(key, value) {
      map.set(key, String(value))
    },
    removeItem(key) {
      map.delete(key)
    },
  }
}

function getDefaultStorageBackend() {
  if (typeof globalThis.localStorage !== 'undefined') {
    return globalThis.localStorage
  }

  return createMapStorage(fallbackState)
}

export function createMemoryStorage(seed = {}) {
  const map = new Map(Object.entries(seed).map(([key, value]) => [key, String(value)]))
  return createMapStorage(map)
}

export function createStorageAdapter(storage = getDefaultStorageBackend()) {
  if (storage && typeof storage.getString === 'function' && typeof storage.setString === 'function') {
    return storage
  }

  return {
    raw: storage,
    getString(key, fallback = '') {
      const value = storage.getItem(key)
      return value == null ? fallback : value
    },
    setString(key, value) {
      if (value == null || value === '') {
        storage.removeItem(key)
        return
      }

      storage.setItem(key, String(value))
    },
    getNumber(key, fallback = 0) {
      const rawValue = storage.getItem(key)
      if (rawValue == null || rawValue === '') {
        return fallback
      }

      const nextValue = Number(rawValue)
      return Number.isFinite(nextValue) ? nextValue : fallback
    },
    setNumber(key, value) {
      if (!Number.isFinite(value)) {
        storage.removeItem(key)
        return
      }

      storage.setItem(key, String(value))
    },
    remove(key) {
      storage.removeItem(key)
    },
  }
}
